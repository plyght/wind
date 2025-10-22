use anyhow::Result;
use git2::{Diff, DiffOptions, Repository, Signature};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

use crate::database::MappingDatabase;
use crate::types::{Changeset, FileOp, GitSha, OpType, WindOid};

pub struct GitImporter {
    repo: Repository,
    db: MappingDatabase,
}

impl GitImporter {
    pub fn new<P: AsRef<Path>>(repo_path: P, db_path: P) -> Result<Self> {
        let repo = Repository::open(repo_path)?;
        let db = MappingDatabase::open(db_path)?;
        Ok(Self { repo, db })
    }

    pub fn import_all(&mut self) -> Result<Vec<Changeset>> {
        info!("Starting full Git repository import");
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        revwalk.set_sorting(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;

        let mut changesets = Vec::new();
        let oids: Vec<_> = revwalk.collect::<Result<_, _>>()?;

        for oid in oids {
            let git_sha = GitSha(oid.to_string());
            if let Some(existing_oid) = self.db.get_wind_oid(&git_sha)? {
                debug!("Commit {} already imported as {}", oid, existing_oid.0);
                continue;
            }

            let wind_oid = WindOid(format!("w{}", oid));

            let (parent_wind_oid, tree_id, parent_tree_id, message, author, timestamp) = {
                let commit = self.repo.find_commit(oid)?;

                let parent_wind_oid = if commit.parent_count() > 0 {
                    let parent = commit.parent(0)?;
                    let parent_sha = GitSha(parent.id().to_string());
                    self.db.get_wind_oid(&parent_sha)?
                } else {
                    None
                };

                let tree_id = commit.tree_id();
                let parent_tree_id = if commit.parent_count() > 0 {
                    Some(commit.parent(0)?.tree_id())
                } else {
                    None
                };

                let message = commit.message().unwrap_or("").to_string();
                let author = format_signature(commit.author());
                let timestamp = commit.time().seconds();

                (
                    parent_wind_oid,
                    tree_id,
                    parent_tree_id,
                    message,
                    author,
                    timestamp,
                )
            };

            let ops = self.extract_ops_from_trees(tree_id, parent_tree_id)?;

            let changeset = Changeset {
                oid: wind_oid.clone(),
                parent: parent_wind_oid,
                message,
                author,
                timestamp,
                ops,
            };

            self.db.insert_mapping(&git_sha, &wind_oid)?;
            debug!("Imported commit {} -> {}", git_sha.0, wind_oid.0);

            changesets.push(changeset);
        }

        info!("Imported {} changesets", changesets.len());
        Ok(changesets)
    }

    fn extract_ops_from_trees(
        &mut self,
        tree_id: git2::Oid,
        parent_tree_id: Option<git2::Oid>,
    ) -> Result<Vec<FileOp>> {
        let mut ops = Vec::new();

        let tree = self.repo.find_tree(tree_id)?;
        let parent_tree = parent_tree_id
            .map(|tid| self.repo.find_tree(tid))
            .transpose()?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(false);

        let diff =
            self.repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;

        let renames = Self::detect_renames_static(&diff)?;

        let delta_info: Vec<_> = diff
            .deltas()
            .map(|d| {
                let status = d.status();
                let new_path = d
                    .new_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string());
                let old_path = d
                    .old_file()
                    .path()
                    .and_then(|p| p.to_str())
                    .map(|s| s.to_string());
                (status, new_path, old_path)
            })
            .collect();

        for (status, new_path, _old_path) in delta_info {
            if let Some(path) = new_path {
                let node_id = match status {
                    git2::Delta::Added => self.db.get_node_id(&path).ok().flatten().or_else(|| {
                        let new_id = self.db.get_next_node_id().ok()?;
                        self.db.insert_node_mapping(&new_id, &path).ok()?;
                        Some(new_id)
                    }),
                    git2::Delta::Renamed => {
                        if let Some(old_path_str) = renames.get(&path) {
                            let nid = self.db.get_node_id(old_path_str).ok().flatten();
                            if let Some(ref n) = nid {
                                let _ = self.db.insert_node_mapping(n, &path);
                            }
                            nid
                        } else {
                            None
                        }
                    }
                    _ => self.db.get_node_id(&path).ok().flatten(),
                };

                let op = Self::create_file_op(status, &path, node_id, &renames);
                if let Some(op) = op {
                    ops.push(op);
                }
            }
        }

        Ok(ops)
    }

    fn detect_renames_static(diff: &Diff) -> Result<HashMap<String, String>> {
        let mut renames = HashMap::new();

        for delta in diff.deltas() {
            if delta.status() == git2::Delta::Renamed {
                if let (Some(old_path), Some(new_path)) = (
                    delta.old_file().path().and_then(|p| p.to_str()),
                    delta.new_file().path().and_then(|p| p.to_str()),
                ) {
                    renames.insert(new_path.to_string(), old_path.to_string());
                }
            }
        }

        Ok(renames)
    }

    fn create_file_op(
        status: git2::Delta,
        path: &str,
        node_id: Option<crate::types::NodeId>,
        renames: &HashMap<String, String>,
    ) -> Option<FileOp> {
        match status {
            git2::Delta::Added => Some(FileOp {
                op_type: OpType::Add,
                path: path.to_string(),
                node_id,
                content: None,
            }),
            git2::Delta::Modified => Some(FileOp {
                op_type: OpType::Edit,
                path: path.to_string(),
                node_id,
                content: None,
            }),
            git2::Delta::Deleted => Some(FileOp {
                op_type: OpType::Delete,
                path: path.to_string(),
                node_id,
                content: None,
            }),
            git2::Delta::Renamed => {
                let old_path_str = renames.get(path)?;
                Some(FileOp {
                    op_type: OpType::Rename {
                        from: old_path_str.clone(),
                    },
                    path: path.to_string(),
                    node_id,
                    content: None,
                })
            }
            _ => None,
        }
    }
}

fn format_signature(sig: Signature) -> String {
    format!(
        "{} <{}>",
        sig.name().unwrap_or(""),
        sig.email().unwrap_or("")
    )
}
