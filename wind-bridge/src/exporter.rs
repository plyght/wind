use anyhow::{Context, Result};
use git2::{Oid, Repository, Signature, Time};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{debug, info};
use wind_storage::SyncObjectStore;

use crate::database::MappingDatabase;
use crate::types::{GitSha, WindOid};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    pub id: String,
    pub parents: Vec<String>,
    pub changes: BTreeMap<String, FileChange>,
    pub commit_message: String,
    pub author: String,
    pub timestamp: i64,
    pub root_manifest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileChange {
    Added { oid: String },
    Modified { oid: String },
    Deleted,
    Renamed { from: String, oid: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub entries: BTreeMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub node_id: String,
    pub oid: String,
    pub permissions: u32,
}

pub struct GitExporter {
    git_repo: Repository,
    wind_storage: Arc<dyn SyncObjectStore>,
    db: MappingDatabase,
}

impl GitExporter {
    pub fn new<P: AsRef<Path>>(
        git_repo_path: P,
        wind_storage: Arc<dyn SyncObjectStore>,
        db_path: P,
    ) -> Result<Self> {
        let git_repo = Repository::open(git_repo_path)?;
        let db = MappingDatabase::open(db_path)?;
        Ok(Self {
            git_repo,
            wind_storage,
            db,
        })
    }

    pub fn export_changeset(&mut self, wind_oid: &str) -> Result<GitSha> {
        info!("Exporting Wind changeset {} to Git", wind_oid);

        let wind_oid_obj = WindOid(wind_oid.to_string());
        if let Some(existing_sha) = self.db.get_git_sha(&wind_oid_obj)? {
            debug!("Changeset already exported as {}", existing_sha.0);
            return Ok(existing_sha);
        }

        let changeset_data = self.wind_storage.read(wind_oid)?;
        let changeset: Changeset = serde_json::from_slice(&changeset_data)
            .context("Failed to deserialize Wind changeset")?;

        let manifest_data = self.wind_storage.read(&changeset.root_manifest)?;
        let manifest: Manifest = serde_json::from_slice(&manifest_data)
            .context("Failed to deserialize manifest")?;

        let tree_oid = self.build_git_tree(&manifest)?;

        let parent_oids = self.resolve_parent_commits(&changeset)?;
        let parent_commits: Vec<_> = parent_oids
            .iter()
            .filter_map(|oid| self.git_repo.find_commit(*oid).ok())
            .collect();
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();

        let sig = parse_signature(&changeset.author, changeset.timestamp)?;
        let tree = self.git_repo.find_tree(tree_oid)?;

        let commit_oid = self.git_repo.commit(
            None,
            &sig,
            &sig,
            &changeset.commit_message,
            &tree,
            &parent_refs,
        )?;

        let git_sha = GitSha(commit_oid.to_string());
        self.db.insert_mapping(&git_sha, &wind_oid_obj)?;

        info!("Exported {} -> {}", wind_oid, git_sha.0);
        Ok(git_sha)
    }

    pub fn export_all(&mut self, wind_head_oid: &str) -> Result<usize> {
        info!("Exporting all changesets from Wind head {}", wind_head_oid);

        if let Some(workdir) = self.git_repo.workdir() {
            let windignore_path = workdir.join(".windignore");
            let gitignore_path = workdir.join(".gitignore");
            
            if windignore_path.exists() && !gitignore_path.exists() {
                std::fs::copy(&windignore_path, &gitignore_path)
                    .context("Failed to copy .windignore to .gitignore")?;
                info!("Created .gitignore from .windignore for Git compatibility");
            }
        }

        let changesets = self.collect_changesets_in_order(wind_head_oid)?;
        let count = changesets.len();

        for changeset_oid in changesets {
            self.export_changeset(&changeset_oid)?;
        }

        Ok(count)
    }

    pub fn update_git_branch(&mut self, branch_name: &str, wind_head_oid: &str) -> Result<()> {
        let wind_oid = WindOid(wind_head_oid.to_string());
        let git_sha = self
            .db
            .get_git_sha(&wind_oid)?
            .ok_or_else(|| anyhow::anyhow!("Wind changeset {} not exported yet", wind_head_oid))?;

        let git_oid = Oid::from_str(&git_sha.0)?;
        let refname = format!("refs/heads/{}", branch_name);
        self.git_repo.reference(&refname, git_oid, true, "Wind export")?;

        self.git_repo.set_head(&refname)?;
        self.git_repo.checkout_head(Some(
            git2::build::CheckoutBuilder::default()
                .force()
                .remove_untracked(true),
        ))?;

        info!("Updated Git branch {} to {}", branch_name, git_sha.0);
        Ok(())
    }

    fn build_git_tree(&self, manifest: &Manifest) -> Result<Oid> {
        let mut builder = self.git_repo.treebuilder(None)?;

        for (path, entry) in &manifest.entries {
            // Skip .git and .wind directories
            let path_str = path.to_string_lossy();
            if path_str.starts_with(".git") || path_str.starts_with(".wind") {
                continue;
            }
            
            let content = self.wind_storage.read(&entry.oid)?;
            let blob_oid = self.git_repo.blob(&content)?;

            let filemode = if entry.permissions & 0o111 != 0 {
                0o100755
            } else {
                0o100644
            };

            if path_str.contains('/') {
                self.add_nested_path(&mut builder, path, blob_oid, filemode)?;
            } else {
                builder.insert(&path_str, blob_oid, filemode)?;
            }
        }

        let tree_oid = builder.write()?;
        Ok(tree_oid)
    }

    fn add_nested_path(
        &self,
        builder: &mut git2::TreeBuilder,
        path: &str,
        blob_oid: Oid,
        filemode: i32,
    ) -> Result<()> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 1 {
            builder.insert(parts[0], blob_oid, filemode)?;
            return Ok(());
        }

        let dir_name = parts[0];
        let rest_path = parts[1..].join("/");

        let existing_tree = if let Ok(Some(entry)) = builder.get(dir_name) {
            if entry.filemode() == 0o040000 {
                self.git_repo.find_tree(entry.id()).ok()
            } else {
                None
            }
        } else {
            None
        };

        let mut sub_builder = if let Some(tree) = existing_tree {
            self.git_repo.treebuilder(Some(&tree))?
        } else {
            self.git_repo.treebuilder(None)?
        };

        self.add_nested_path(&mut sub_builder, &rest_path, blob_oid, filemode)?;
        let subtree_oid = sub_builder.write()?;

        builder.insert(dir_name, subtree_oid, 0o040000)?;
        Ok(())
    }

    fn resolve_parent_commits(&mut self, changeset: &Changeset) -> Result<Vec<Oid>> {
        let mut parent_oids = Vec::new();

        for parent_wind_oid in &changeset.parents {
            let wind_oid = WindOid(parent_wind_oid.clone());
            if let Some(git_sha) = self.db.get_git_sha(&wind_oid)? {
                let oid = Oid::from_str(&git_sha.0)?;
                parent_oids.push(oid);
            } else {
                self.export_changeset(parent_wind_oid)?;
                if let Some(git_sha) = self.db.get_git_sha(&wind_oid)? {
                    let oid = Oid::from_str(&git_sha.0)?;
                    parent_oids.push(oid);
                }
            }
        }

        Ok(parent_oids)
    }

    fn collect_changesets_in_order(&self, head_oid: &str) -> Result<Vec<String>> {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![head_oid.to_string()];
        let mut result = Vec::new();

        while let Some(current_oid) = stack.pop() {
            if visited.contains(&current_oid) {
                continue;
            }

            let wind_oid = WindOid(current_oid.clone());
            if self.db.get_git_sha(&wind_oid)?.is_some() {
                visited.insert(current_oid);
                continue;
            }

            let data = self.wind_storage.read(&current_oid)?;
            let changeset: Changeset = serde_json::from_slice(&data)?;

            let parents_exported = changeset
                .parents
                .iter()
                .all(|p| self.db.get_git_sha(&WindOid(p.clone())).ok().flatten().is_some());

            if parents_exported {
                result.push(current_oid.clone());
                visited.insert(current_oid);
            } else {
                stack.push(current_oid.clone());
                for parent in &changeset.parents {
                    if !visited.contains(parent) {
                        stack.push(parent.clone());
                    }
                }
            }
        }

        Ok(result)
    }
}

fn parse_signature(author: &str, timestamp: i64) -> Result<Signature> {
    let parts: Vec<&str> = author.split('<').collect();
    let name = parts[0].trim();
    let email = parts
        .get(1)
        .and_then(|s| s.strip_suffix('>'))
        .unwrap_or("unknown@localhost");

    let time = Time::new(timestamp, 0);
    Ok(Signature::new(name, email, &time)?)
}
