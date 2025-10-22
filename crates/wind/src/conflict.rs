use anyhow::{Context, Result};
use git2::Repository as GitRepository;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ConflictFile {
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct ConflictContent {
    pub base: Option<String>,
    pub ours: String,
    pub theirs: String,
}

pub struct ConflictResolver<'repo> {
    git_repo: &'repo GitRepository,
}

impl<'repo> ConflictResolver<'repo> {
    pub fn new(git_repo: &'repo GitRepository) -> Self {
        Self { git_repo }
    }

    pub fn detect_conflicts(&self) -> Result<Vec<ConflictFile>> {
        let index = self.git_repo.index()?;
        let mut conflicts = Vec::new();

        if index.has_conflicts() {
            let conflicts_iter = index.conflicts()?;
            for conflict in conflicts_iter {
                let conflict = conflict?;
                if let Some(our) = conflict.our {
                    let path = String::from_utf8_lossy(&our.path).to_string();
                    conflicts.push(ConflictFile { path });
                }
            }
        }

        Ok(conflicts)
    }

    pub fn get_conflict_content(&self, path: &str) -> Result<ConflictContent> {
        let index = self.git_repo.index()?;
        let conflicts_iter = index.conflicts()?;

        for conflict in conflicts_iter {
            let conflict = conflict?;

            let conflict_path = if let Some(our) = &conflict.our {
                String::from_utf8_lossy(&our.path).to_string()
            } else if let Some(their) = &conflict.their {
                String::from_utf8_lossy(&their.path).to_string()
            } else {
                continue;
            };

            if conflict_path == path {
                let base = if let Some(ancestor) = conflict.ancestor {
                    let blob = self.git_repo.find_blob(ancestor.id)?;
                    Some(String::from_utf8_lossy(blob.content()).to_string())
                } else {
                    None
                };

                let ours = if let Some(our) = conflict.our {
                    let blob = self.git_repo.find_blob(our.id)?;
                    String::from_utf8_lossy(blob.content()).to_string()
                } else {
                    String::new()
                };

                let theirs = if let Some(their) = conflict.their {
                    let blob = self.git_repo.find_blob(their.id)?;
                    String::from_utf8_lossy(blob.content()).to_string()
                } else {
                    String::new()
                };

                return Ok(ConflictContent { base, ours, theirs });
            }
        }

        anyhow::bail!("No conflict found for path: {path}")
    }

    pub fn apply_resolution(&self, path: &str, content: &str) -> Result<()> {
        let repo_path = self
            .git_repo
            .path()
            .parent()
            .context("Failed to get repository path")?;
        let file_path = repo_path.join(path);
        std::fs::write(&file_path, content).context("Failed to write resolved content")?;
        Ok(())
    }

    pub fn mark_resolved(&self, path: &str) -> Result<()> {
        let mut index = self.git_repo.index()?;
        index.add_path(Path::new(path))?;
        index.write()?;
        Ok(())
    }
}
