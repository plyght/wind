use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use crate::index::{get_mtime, Index, IndexEntry};

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
    Untracked,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: PathBuf,
    pub status: FileStatus,
    pub node_id: Option<String>,
}

pub struct WorkingCopy {
    root_path: PathBuf,
    index: Index,
    storage: Arc<dyn wind_storage::SyncObjectStore>,
}

impl WorkingCopy {
    pub fn new(
        root_path: PathBuf,
        wind_dir: &Path,
        storage: Arc<dyn wind_storage::SyncObjectStore>,
    ) -> Result<Self> {
        let index = Index::new(wind_dir)?;
        Ok(Self {
            root_path,
            index,
            storage,
        })
    }

    pub fn scan_working_tree(&self) -> Result<Vec<FileChange>> {
        let mut changes = Vec::new();
        let indexed = self.index.list_all()?;
        let mut indexed_map: HashMap<PathBuf, IndexEntry> =
            indexed.into_iter().map(|e| (e.path.clone(), e)).collect();

        let gitignore_path = self.root_path.join(".gitignore");
        let windignore_path = self.root_path.join(".windignore");

        let mut builder = ignore::WalkBuilder::new(&self.root_path);
        builder
            .add_custom_ignore_filename(".windignore")
            .hidden(false);

        if gitignore_path.exists() {
            builder.add_ignore(&gitignore_path);
        } else if windignore_path.exists() {
            builder.add_ignore(&windignore_path);
        }

        builder.filter_entry(|e| {
            !e.path()
                .components()
                .any(|c| c.as_os_str() == ".wind" || c.as_os_str() == ".git")
        });

        for result in builder.build() {
            let entry = match result {
                Ok(e) => e,
                Err(_) => continue,
            };

            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let abs_path = entry.path();
            let rel_path = abs_path
                .strip_prefix(&self.root_path)
                .unwrap()
                .to_path_buf();

            let metadata = entry.metadata()?;
            let mtime = get_mtime(abs_path)?;
            let size = metadata.len();

            if let Some(idx_entry) = indexed_map.remove(&rel_path) {
                if idx_entry.mtime != mtime || idx_entry.size != size {
                    let content = fs::read(abs_path)?;
                    let oid = self.storage.write(&content)?;

                    if oid != idx_entry.oid {
                        changes.push(FileChange {
                            path: rel_path,
                            status: FileStatus::Modified,
                            node_id: Some(idx_entry.node_id),
                        });
                    }
                }
            } else {
                use uuid::Uuid;
                let node_id = Uuid::new_v4().to_string();
                changes.push(FileChange {
                    path: rel_path,
                    status: FileStatus::Untracked,
                    node_id: Some(node_id),
                });
            }
        }

        let mut untracked_with_content: HashMap<PathBuf, (FileChange, String)> = HashMap::new();
        for change in &changes {
            if change.status == FileStatus::Untracked {
                let abs_path = self.root_path.join(&change.path);
                let content = fs::read(&abs_path)?;
                let oid = self.storage.write(&content)?;
                untracked_with_content.insert(change.path.clone(), (change.clone(), oid));
            }
        }

        let mut renamed = Vec::new();
        for (path, entry) in &indexed_map {
            let mut found_rename = false;
            for (untracked_path, (_, untracked_oid)) in &untracked_with_content {
                if *untracked_oid == entry.oid {
                    renamed.push(FileChange {
                        path: untracked_path.clone(),
                        status: FileStatus::Renamed {
                            from: path.clone(),
                            to: untracked_path.clone(),
                        },
                        node_id: Some(entry.node_id.clone()),
                    });
                    found_rename = true;
                    break;
                }
            }
            if !found_rename {
                changes.push(FileChange {
                    path: path.clone(),
                    status: FileStatus::Deleted,
                    node_id: Some(entry.node_id.clone()),
                });
            }
        }

        for rename_change in renamed {
            if let FileStatus::Renamed { ref to, .. } = rename_change.status {
                changes.retain(|c| c.path != *to || c.status != FileStatus::Untracked);
            }
            changes.push(rename_change);
        }

        Ok(changes)
    }

    pub fn add_file(&mut self, path: &Path) -> Result<()> {
        let abs_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root_path.join(path)
        };

        // Handle directories recursively
        if abs_path.is_dir() {
            for entry in WalkDir::new(&abs_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_file() {
                    self.add_single_file(entry.path())?;
                }
            }
            return Ok(());
        }

        self.add_single_file(&abs_path)
    }

    fn add_single_file(&mut self, abs_path: &Path) -> Result<()> {
        let rel_path = abs_path.strip_prefix(&self.root_path)?.to_path_buf();
        let content = fs::read(abs_path)?;
        let oid = self.storage.write(&content)?;

        let metadata = fs::metadata(&abs_path)?;
        let mtime = get_mtime(&abs_path)?;
        let size = content.len() as u64;

        let node_id = if let Some(entry) = self.index.lookup(&rel_path)? {
            entry.node_id
        } else {
            use uuid::Uuid;
            Uuid::new_v4().to_string()
        };

        #[cfg(unix)]
        let permissions = metadata.permissions().mode();
        #[cfg(not(unix))]
        let permissions = 0o644;

        self.index.add(&IndexEntry {
            path: rel_path,
            node_id,
            oid,
            mtime,
            size,
            permissions,
        })?;

        Ok(())
    }

    pub fn remove_file(&mut self, path: &Path) -> Result<()> {
        let rel_path = if path.is_absolute() {
            path.strip_prefix(&self.root_path)?.to_path_buf()
        } else {
            path.to_path_buf()
        };

        self.index.remove(&rel_path)?;
        Ok(())
    }

    pub fn get_index(&self) -> &Index {
        &self.index
    }
}
