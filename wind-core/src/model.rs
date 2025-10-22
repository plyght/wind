use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub type NodeId = String;
pub type BranchId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Changeset {
    pub id: String,
    pub parents: Vec<String>,
    pub changes: BTreeMap<NodeId, FileChange>,
    pub commit_message: String,
    pub author: String,
    pub timestamp: i64,
    pub root_manifest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileChange {
    Added { oid: String },
    Modified { oid: String },
    Deleted,
    Renamed { from: NodeId, oid: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub entries: BTreeMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    pub node_id: NodeId,
    pub oid: String,
    pub permissions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub id: BranchId,
    pub name: String,
    pub head: String,
}

impl Changeset {
    pub fn new(
        parents: Vec<String>,
        changes: BTreeMap<NodeId, FileChange>,
        commit_message: String,
        author: String,
        root_manifest: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            parents,
            changes,
            commit_message,
            author,
            timestamp: chrono::Utc::now().timestamp(),
            root_manifest,
        }
    }
}

impl Manifest {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, path: String, node_id: NodeId, oid: String, permissions: u32) {
        self.entries.insert(
            path,
            ManifestEntry {
                node_id,
                oid,
                permissions,
            },
        );
    }

    pub fn remove(&mut self, path: &str) {
        self.entries.remove(path);
    }

    pub fn get(&self, path: &str) -> Option<&ManifestEntry> {
        self.entries.get(path)
    }
}

impl Default for Manifest {
    fn default() -> Self {
        Self::new()
    }
}
