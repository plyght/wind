use anyhow::{Context, Result};
use git2::Repository as GitRepository;
use std::path::{Path, PathBuf};

use crate::cache::StatusCache;
use crate::conflict::{ConflictContent, ConflictFile, ConflictResolver};
use crate::perf::{analyze_repo, PerfConfig};
use crate::submodule::{is_inside_submodule, list_submodules, Submodule};
use crate::worktree::{is_worktree, list_worktrees, Worktree};

pub struct Repository {
    git_repo: GitRepository,
    workdir: PathBuf,
    status_cache: StatusCache,
    perf_config: PerfConfig,
}

#[derive(Clone)]
pub struct Status {
    pub branch: String,
    pub staged: Vec<String>,
    pub modified: Vec<String>,
    pub untracked: Vec<String>,
    pub is_worktree: bool,
    pub submodules: Vec<SubmoduleStatus>,
}

#[derive(Clone)]
pub struct SubmoduleStatus {
    pub name: String,
    pub path: PathBuf,
    pub initialized: bool,
}

pub struct Commit {
    pub id: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

impl Repository {
    pub fn init(path: &Path) -> Result<Self> {
        let git_repo = GitRepository::init(path).context("Failed to initialize git repository")?;

        std::fs::create_dir_all(path.join(".wind"))?;
        std::fs::write(path.join(".wind/config.toml"), "")?;

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
            std::fs::write(windignore_path, default_content)?;
        }

        let gitignore_content = if gitignore_path.exists() {
            let existing = std::fs::read_to_string(&gitignore_path)?;
            if !existing.contains(".wind") {
                format!("{}\n.wind/\n", existing.trim_end())
            } else {
                existing
            }
        } else {
            ".wind/\n".to_string()
        };
        std::fs::write(gitignore_path, gitignore_content)?;

        {
            let mut index = git_repo.index()?;
            index.add_path(Path::new(".gitignore"))?;
            index.write()?;

            let tree_id = index.write_tree()?;
            let tree = git_repo.find_tree(tree_id)?;
            let signature = git_repo
                .signature()
                .or_else(|_| git2::Signature::now("Wind", "wind@example.com"))?;

            git_repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                "Initial commit",
                &tree,
                &[],
            )?;
        }

        let perf_config = PerfConfig::default();
        let status_cache = StatusCache::new(perf_config.cache_ttl_ms);

        Ok(Self {
            git_repo,
            workdir: path.to_path_buf(),
            status_cache,
            perf_config,
        })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let git_repo = GitRepository::open(path.as_ref())
            .or_else(|_| GitRepository::discover(path.as_ref()))
            .context("Not a wind repository")?;

        let workdir = git_repo
            .workdir()
            .context("Repository has no working directory")?
            .to_path_buf();

        let repo_info = analyze_repo(&git_repo)?;
        let perf_config = PerfConfig::adjust_for_repo(&repo_info);
        let status_cache = StatusCache::new(perf_config.cache_ttl_ms);

        if repo_info.is_large {
            eprintln!(
                "Large repository detected ({} files, {:.2} MB)",
                repo_info.file_count, repo_info.repo_size_mb
            );
            eprintln!("Performance optimizations enabled:");
            eprintln!("  - Cache TTL: {}ms", perf_config.cache_ttl_ms);
            eprintln!("  - Auto-refresh: {}", perf_config.auto_refresh);
            eprintln!("  - Untracked files: {}", perf_config.status_untracked);
        }

        Ok(Self {
            git_repo,
            workdir,
            status_cache,
            perf_config,
        })
    }

    pub fn status(&self) -> Result<Status> {
        let cache_key = self.workdir.clone();

        if let Some(cached) = self.status_cache.get(&cache_key) {
            return Ok(cached);
        }

        let branch = match self.git_repo.head() {
            Ok(head) => head.shorthand().unwrap_or("HEAD").to_string(),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => "main".to_string(),
            Err(e) => return Err(e.into()),
        };

        let mut opts = git2::StatusOptions::new();
        opts.include_unmodified(false);

        if self.perf_config.status_untracked {
            opts.include_untracked(true);
        } else {
            opts.include_untracked(false);
        }

        opts.exclude_submodules(true);

        let statuses = self.git_repo.statuses(Some(&mut opts))?;

        let mut staged = Vec::new();
        let mut modified = Vec::new();
        let mut untracked = Vec::new();

        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();

            if path.starts_with(".wind/")
                || path.starts_with("target/")
                || path.starts_with("node_modules/")
            {
                continue;
            }

            let status = entry.status();

            if status.is_index_new() || status.is_index_modified() || status.is_index_deleted() {
                staged.push(path.clone());
            }
            if status.is_wt_modified() || status.is_wt_deleted() {
                modified.push(path.clone());
            }
            if status.is_wt_new() {
                untracked.push(path);
            }
        }

        let is_worktree = is_worktree(&self.workdir).unwrap_or(false);
        let submodules = list_submodules(&self.workdir)
            .ok()
            .unwrap_or_default()
            .into_iter()
            .map(|s| SubmoduleStatus {
                name: s.name,
                path: s.path,
                initialized: s.initialized,
            })
            .collect();

        let status = Status {
            branch,
            staged,
            modified,
            untracked,
            is_worktree,
            submodules,
        };

        self.status_cache.set(cache_key, status.clone());

        Ok(status)
    }

    pub fn invalidate_cache(&self) {
        self.status_cache.invalidate();
    }

    pub fn get_diff(&self, path: &str, context_lines: usize) -> Result<String> {
        let head = self.git_repo.head()?.peel_to_tree()?;
        let mut diff_opts = git2::DiffOptions::new();
        diff_opts.context_lines(context_lines as u32);
        diff_opts.pathspec(path);

        let diff = self
            .git_repo
            .diff_tree_to_workdir_with_index(Some(&head), Some(&mut diff_opts))?;

        let mut output = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            if output.len() > 1_000_000 {
                return false;
            }
            if let Ok(content) = std::str::from_utf8(line.content()) {
                output.push_str(content);
            }
            true
        })?;

        Ok(output)
    }

    pub fn add(&self, path: &str) -> Result<()> {
        let mut index = self.git_repo.index()?;
        index.add_path(Path::new(path))?;
        index.write()?;
        self.invalidate_cache();
        Ok(())
    }

    pub fn add_all(&self) -> Result<()> {
        let mut index = self.git_repo.index()?;
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        self.invalidate_cache();
        Ok(())
    }

    pub fn commit(&self, message: &str) -> Result<String> {
        let mut index = self.git_repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.git_repo.find_tree(tree_id)?;

        let signature = self.git_repo.signature()?;

        let parents = match self.git_repo.head() {
            Ok(head) => vec![head.peel_to_commit()?],
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => vec![],
            Err(e) => return Err(e.into()),
        };

        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();

        let commit_id = self.git_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parent_refs,
        )?;

        self.invalidate_cache();
        Ok(commit_id.to_string())
    }

    pub fn log(&self, limit: Option<usize>) -> Result<Vec<Commit>> {
        let mut revwalk = self.git_repo.revwalk()?;

        if let Err(e) = revwalk.push_head() {
            if e.code() == git2::ErrorCode::UnbornBranch {
                return Ok(Vec::new());
            }
            return Err(e.into());
        }

        let mut commits = Vec::new();
        let limit = limit.unwrap_or(usize::MAX);

        for (i, oid) in revwalk.enumerate() {
            if i >= limit {
                break;
            }

            let oid = oid?;
            let commit = self.git_repo.find_commit(oid)?;

            commits.push(Commit {
                id: oid.to_string(),
                author: commit.author().to_string(),
                date: format!("{}", commit.time().seconds()),
                message: commit.message().unwrap_or("").to_string(),
            });
        }

        Ok(commits)
    }

    pub fn log_paginated(&self, offset: usize, limit: usize) -> Result<Vec<Commit>> {
        let mut revwalk = self.git_repo.revwalk()?;

        if let Err(e) = revwalk.push_head() {
            if e.code() == git2::ErrorCode::UnbornBranch {
                return Ok(Vec::new());
            }
            return Err(e.into());
        }

        let mut commits = Vec::new();

        for (i, oid) in revwalk.enumerate() {
            if i < offset {
                continue;
            }
            if commits.len() >= limit {
                break;
            }

            let oid = oid?;
            let commit = self.git_repo.find_commit(oid)?;

            commits.push(Commit {
                id: oid.to_string(),
                author: commit.author().to_string(),
                date: format!("{}", commit.time().seconds()),
                message: commit.message().unwrap_or("").to_string(),
            });
        }

        Ok(commits)
    }

    pub fn create_branch(&self, name: &str) -> Result<()> {
        let commit = match self.git_repo.head() {
            Ok(head) => head.peel_to_commit()?,
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                anyhow::bail!("Cannot create branch on empty repository. Create a commit first.");
            }
            Err(e) => return Err(e.into()),
        };
        self.git_repo.branch(name, &commit, false)?;
        Ok(())
    }

    pub fn delete_branch(&self, name: &str) -> Result<()> {
        let mut branch = self.git_repo.find_branch(name, git2::BranchType::Local)?;
        branch.delete()?;
        Ok(())
    }

    pub fn list_branches(&self) -> Result<Vec<String>> {
        let branches = self.git_repo.branches(Some(git2::BranchType::Local))?;
        let mut result = Vec::new();

        for branch in branches {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                result.push(name.to_string());
            }
        }

        Ok(result)
    }

    pub fn current_branch(&self) -> Result<String> {
        match self.git_repo.head() {
            Ok(head) => Ok(head.shorthand().unwrap_or("HEAD").to_string()),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => Ok("main".to_string()),
            Err(e) => Err(e.into()),
        }
    }

    pub fn checkout(&self, target: &str) -> Result<()> {
        let obj = self.git_repo.revparse_single(target)?;
        self.git_repo.checkout_tree(&obj, None)?;
        self.git_repo.set_head(&format!("refs/heads/{target}"))?;
        self.invalidate_cache();
        Ok(())
    }

    pub fn rebase(&self, onto: &str) -> Result<()> {
        let onto_annotated = self
            .git_repo
            .find_annotated_commit(self.git_repo.revparse_single(onto)?.id())?;
        let head_annotated = self
            .git_repo
            .find_annotated_commit(self.git_repo.head()?.peel_to_commit()?.id())?;

        let mut rebase =
            self.git_repo
                .rebase(Some(&head_annotated), Some(&onto_annotated), None, None)?;

        while let Some(_op) = rebase.next() {
            rebase.commit(None, &self.git_repo.signature()?, None)?;
        }

        rebase.finish(None)?;
        self.invalidate_cache();
        Ok(())
    }

    pub fn config_get(&self, key: &str) -> Result<String> {
        let config = self.git_repo.config()?;
        Ok(config.get_string(key)?)
    }

    pub fn config_set(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.git_repo.config()?;
        config.set_str(key, value)?;
        Ok(())
    }

    pub fn config_list(&self) -> Result<Vec<(String, String)>> {
        let config = self.git_repo.config()?;
        let mut result = Vec::new();

        let mut entries = config.entries(None)?;
        while let Some(entry) = entries.next() {
            let entry = entry?;
            if let (Some(name), Some(value)) = (entry.name(), entry.value()) {
                result.push((name.to_string(), value.to_string()));
            }
        }

        Ok(result)
    }

    pub fn detect_conflicts(&self) -> Result<Vec<ConflictFile>> {
        let resolver = ConflictResolver::new(&self.git_repo);
        resolver.detect_conflicts()
    }

    pub fn get_conflict_content(&self, path: &str) -> Result<ConflictContent> {
        let resolver = ConflictResolver::new(&self.git_repo);
        resolver.get_conflict_content(path)
    }

    pub fn apply_resolution(&self, path: &str, content: &str) -> Result<()> {
        let resolver = ConflictResolver::new(&self.git_repo);
        resolver.apply_resolution(path, content)
    }

    pub fn mark_resolved(&self, path: &str) -> Result<()> {
        let resolver = ConflictResolver::new(&self.git_repo);
        resolver.mark_resolved(path)
    }

    pub fn list_worktrees(&self) -> Result<Vec<Worktree>> {
        list_worktrees(&self.workdir)
    }

    pub fn list_submodules(&self) -> Result<Vec<Submodule>> {
        list_submodules(&self.workdir)
    }

    pub fn is_inside_submodule(&self) -> Result<bool> {
        is_inside_submodule(&self.workdir)
    }
}
