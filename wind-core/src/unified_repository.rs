use crate::merge::{MergeEngine, MergeResult};
use crate::model::{Branch, BranchId, Changeset, FileChange as ModelFileChange, Manifest, NodeId};
use crate::working_copy::{FileChange, WorkingCopy};
use anyhow::{anyhow, Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;
use wind_bridge::{GitExporter, GitImporter};
use wind_storage::{FileSystemStore, SyncObjectStore};

pub struct UnifiedRepository {
    storage: Arc<FileSystemStore>,
    working_copy: WorkingCopy,
    merge_engine: MergeEngine,
    wind_dir: PathBuf,
    root_path: PathBuf,
    current_branch: Option<BranchId>,
}

impl UnifiedRepository {
    pub fn init(path: PathBuf) -> Result<Self> {
        let wind_dir = path.join(".wind");
        fs::create_dir_all(&wind_dir).context("Failed to create .wind directory")?;
        fs::create_dir_all(wind_dir.join("objects"))?;
        fs::create_dir_all(wind_dir.join("refs/heads"))?;

        let gitignore_path = path.join(".gitignore");
        let windignore_path = path.join(".windignore");
        
        if !gitignore_path.exists() && !windignore_path.exists() {
            let default_content = "# Wind VCS ignore file
.wind/
.git/
target/
node_modules/
*.swp
*.tmp
.DS_Store
";
            fs::write(windignore_path, default_content)?;
        }

        let storage_path = wind_dir.join("storage");
        let storage = Arc::new(FileSystemStore::new(&storage_path)?);

        let working_copy = WorkingCopy::new(path.clone(), &wind_dir, storage.clone() as Arc<dyn wind_storage::SyncObjectStore>)?;
        let merge_engine = MergeEngine::new(storage.clone() as Arc<dyn wind_storage::SyncObjectStore>);

        let main_branch = Branch {
            id: Uuid::new_v4().to_string(),
            name: "main".to_string(),
            head: String::new(),
        };

        let repo = Self {
            storage,
            working_copy,
            merge_engine,
            wind_dir: wind_dir.clone(),
            root_path: path,
            current_branch: Some(main_branch.id.clone()),
        };

        repo.write_branch(&main_branch)?;
        
        let head_path = wind_dir.join("HEAD");
        fs::write(head_path, &main_branch.id)?;

        Ok(repo)
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        let wind_dir = path.join(".wind");
        if !wind_dir.exists() {
            return Err(anyhow!("Not a Wind repository: .wind directory not found"));
        }

        let storage_path = wind_dir.join("storage");
        let storage = Arc::new(FileSystemStore::new(&storage_path)?);

        let working_copy = WorkingCopy::new(path.clone(), &wind_dir, storage.clone() as Arc<dyn wind_storage::SyncObjectStore>)?;
        let merge_engine = MergeEngine::new(storage.clone() as Arc<dyn wind_storage::SyncObjectStore>);

        let head_path = wind_dir.join("HEAD");
        let current_branch = if head_path.exists() {
            let content = fs::read_to_string(&head_path)?;
            Some(content.trim().to_string())
        } else {
            None
        };

        Ok(Self {
            storage,
            working_copy,
            merge_engine,
            wind_dir,
            root_path: path,
            current_branch,
        })
    }

    pub fn status(&self) -> Result<Vec<FileChange>> {
        self.working_copy.scan_working_tree()
    }

    pub fn add(&mut self, paths: Vec<PathBuf>) -> Result<()> {
        for path in paths {
            self.working_copy.add_file(&path)?;
        }
        Ok(())
    }

    pub fn commit(&mut self, message: &str) -> Result<String> {
        let index = self.working_copy.get_index();
        let index_entries = index.list_all()?;

        let mut changeset_changes: BTreeMap<NodeId, ModelFileChange> = BTreeMap::new();
        
        for entry in index_entries {
            let file_change = ModelFileChange::Added { oid: entry.oid.clone() };
            changeset_changes.insert(entry.node_id.clone(), file_change);
        }

        let manifest = self.build_current_manifest()?;
        let manifest_data = serde_json::to_vec(&manifest)?;
        let manifest_oid = self.storage.write(&manifest_data)?;

        let parents = if let Some(branch_id) = &self.current_branch {
            let branch = self.read_branch(branch_id)?;
            if branch.head.is_empty() {
                vec![]
            } else {
                vec![branch.head.clone()]
            }
        } else {
            vec![]
        };

        let author = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let changeset = Changeset::new(parents, changeset_changes, message.to_string(), author, manifest_oid);

        let changeset_data = serde_json::to_vec(&changeset)?;
        let changeset_oid = self.storage.write(&changeset_data)?;

        if let Some(branch_id) = &self.current_branch {
            let mut branch = self.read_branch(branch_id)?;
            branch.head = changeset_oid.clone();
            self.write_branch(&branch)?;
        }

        Ok(changeset_oid)
    }

    pub fn checkout(&mut self, target: &str) -> Result<()> {
        let branch = self.find_branch_by_name(target)?;
        self.current_branch = Some(branch.id.clone());

        let head_path = self.wind_dir.join("HEAD");
        fs::write(head_path, &branch.id)?;

        Ok(())
    }

    pub fn merge(&mut self, other_oid: String) -> Result<MergeResult> {
        let current_branch = self.current_branch.as_ref().ok_or_else(|| anyhow!("No current branch"))?;
        let branch = self.read_branch(current_branch)?;

        let base_data = self.storage.read(&branch.head)?;
        let base: Changeset = serde_json::from_slice(&base_data)?;

        let ours_data = self.storage.read(&branch.head)?;
        let ours: Changeset = serde_json::from_slice(&ours_data)?;

        let theirs_data = self.storage.read(&other_oid)?;
        let theirs: Changeset = serde_json::from_slice(&theirs_data)?;

        self.merge_engine.merge(&base, &ours, &theirs)
    }

    pub fn branches(&self) -> Result<Vec<Branch>> {
        let refs_dir = self.wind_dir.join("refs/heads");
        let mut branches = Vec::new();

        if refs_dir.exists() {
            for entry in fs::read_dir(&refs_dir)? {
                let entry = entry?;
                let branch_id = entry.file_name().to_string_lossy().to_string();
                let branch = self.read_branch(&branch_id)?;
                branches.push(branch);
            }
        }

        Ok(branches)
    }

    pub fn log(&self, limit: usize) -> Result<Vec<Changeset>> {
        let current_branch = self.current_branch.as_ref().ok_or_else(|| anyhow!("No current branch"))?;
        let branch = self.read_branch(current_branch)?;

        let mut changesets = Vec::new();
        let mut current_oid = branch.head.clone();

        for _ in 0..limit {
            if current_oid.is_empty() {
                break;
            }

            let data = self.storage.read(&current_oid)?;
            let changeset: Changeset = serde_json::from_slice(&data)?;

            let parent = changeset.parents.first().cloned();
            changesets.push(changeset);

            if let Some(parent_oid) = parent {
                current_oid = parent_oid;
            } else {
                break;
            }
        }

        Ok(changesets)
    }

    pub fn sync_with_git(&mut self) -> Result<()> {
        let git_dir = self.root_path.join(".git");
        if !git_dir.exists() {
            return Err(anyhow!("No .git directory found"));
        }

        let db_path = self.wind_dir.join("bridge.db");
        let mut importer = GitImporter::new(&git_dir, &db_path)?;
        importer.import_all()?;

        Ok(())
    }

    pub fn import_git(git_path: PathBuf) -> Result<Self> {
        let wind_dir = git_path.join(".wind");
        fs::create_dir_all(&wind_dir)?;

        let db_path = wind_dir.join("bridge.db");
        let mut importer = GitImporter::new(&git_path.join(".git"), &db_path)?;
        importer.import_all()?;

        Self::open(git_path)
    }

    pub fn export_git(&self, git_path: PathBuf) -> Result<()> {
        fs::create_dir_all(&git_path)?;
        git2::Repository::init(&git_path)?;

        let db_path = self.wind_dir.join("bridge.db");
        let mut exporter = GitExporter::new(
            &git_path.join(".git"),
            self.storage.clone() as Arc<dyn wind_storage::SyncObjectStore>,
            &db_path,
        )?;

        if let Some(branch_id) = &self.current_branch {
            let branch = self.read_branch(branch_id)?;
            if !branch.head.is_empty() {
                let count = exporter.export_all(&branch.head)?;
                exporter.update_git_branch(&branch.name, &branch.head)?;
                println!("Exported {} changesets to Git", count);
            }
        }

        Ok(())
    }

    fn build_current_manifest(&self) -> Result<Manifest> {
        let mut manifest = Manifest::new();
        let index = self.working_copy.get_index();
        let entries = index.list_all()?;

        for entry in entries {
            manifest.add(
                entry.path.to_string_lossy().to_string(),
                entry.node_id,
                entry.oid,
                entry.permissions,
            );
        }

        Ok(manifest)
    }

    fn write_branch(&self, branch: &Branch) -> Result<()> {
        let branch_path = self.wind_dir.join("refs/heads").join(&branch.id);
        let branch_data = serde_json::to_vec(branch)?;
        fs::write(branch_path, branch_data)?;
        Ok(())
    }

    fn read_branch(&self, branch_id: &str) -> Result<Branch> {
        let branch_path = self.wind_dir.join("refs/heads").join(branch_id);
        let branch_data = fs::read(branch_path)?;
        let branch: Branch = serde_json::from_slice(&branch_data)?;
        Ok(branch)
    }

    fn find_branch_by_name(&self, name: &str) -> Result<Branch> {
        let branches = self.branches()?;
        branches
            .into_iter()
            .find(|b| b.name == name)
            .ok_or_else(|| anyhow!("Branch not found: {}", name))
    }
}
