use crate::model::{Changeset, FileChange, Manifest, NodeId};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

pub struct MergeEngine {
    storage: Arc<dyn wind_storage::SyncObjectStore>,
}

#[derive(Debug, Clone)]
pub enum MergeResult {
    Clean { new_changeset_id: String },
    Conflicts { conflicts: Vec<ConflictInfo> },
}

#[derive(Debug, Clone)]
pub struct ConflictInfo {
    pub node_id: NodeId,
    pub path: String,
    pub base_oid: Option<String>,
    pub ours_oid: Option<String>,
    pub theirs_oid: Option<String>,
}

impl MergeEngine {
    pub fn new(storage: Arc<dyn wind_storage::SyncObjectStore>) -> Self {
        Self { storage }
    }

    pub fn merge(
        &self,
        base: &Changeset,
        ours: &Changeset,
        theirs: &Changeset,
    ) -> Result<MergeResult> {
        let base_manifest = self.load_manifest(&base.root_manifest)?;
        let ours_manifest = self.load_manifest(&ours.root_manifest)?;
        let theirs_manifest = self.load_manifest(&theirs.root_manifest)?;

        let all_node_ids = self.collect_all_node_ids(&base_manifest, &ours_manifest, &theirs_manifest);

        let mut conflicts = Vec::new();
        let mut merged_changes: BTreeMap<NodeId, FileChange> = BTreeMap::new();

        for node_id in all_node_ids {
            let base_entry = base_manifest.entries.values().find(|e| e.node_id == node_id);
            let ours_entry = ours_manifest.entries.values().find(|e| e.node_id == node_id);
            let theirs_entry = theirs_manifest.entries.values().find(|e| e.node_id == node_id);

            let base_oid = base_entry.map(|e| e.oid.clone());
            let ours_oid = ours_entry.map(|e| e.oid.clone());
            let theirs_oid = theirs_entry.map(|e| e.oid.clone());

            match (base_oid.as_ref(), ours_oid.as_ref(), theirs_oid.as_ref()) {
                (Some(_b), Some(o), Some(t)) if o == t => {
                    continue;
                }
                (Some(b), Some(o), Some(t)) if b == o && b != t => {
                    merged_changes.insert(node_id.clone(), FileChange::Modified { oid: t.clone() });
                }
                (Some(b), Some(o), Some(t)) if b == t && b != o => {
                    merged_changes.insert(node_id.clone(), FileChange::Modified { oid: o.clone() });
                }
                (Some(_), Some(o), Some(t)) if o != t => {
                    let path = self.find_path_for_node(&ours_manifest, &node_id)
                        .or_else(|| self.find_path_for_node(&theirs_manifest, &node_id))
                        .unwrap_or_else(|| format!("unknown_{}", node_id));
                    
                    conflicts.push(ConflictInfo {
                        node_id: node_id.clone(),
                        path,
                        base_oid: base_oid.clone(),
                        ours_oid: ours_oid.clone(),
                        theirs_oid: theirs_oid.clone(),
                    });
                }
                (None, Some(o), None) => {
                    merged_changes.insert(node_id.clone(), FileChange::Added { oid: o.clone() });
                }
                (None, None, Some(t)) => {
                    merged_changes.insert(node_id.clone(), FileChange::Added { oid: t.clone() });
                }
                (Some(_), None, None) => {
                    merged_changes.insert(node_id.clone(), FileChange::Deleted);
                }
                (None, Some(o), Some(t)) if o != t => {
                    let path = self.find_path_for_node(&ours_manifest, &node_id)
                        .or_else(|| self.find_path_for_node(&theirs_manifest, &node_id))
                        .unwrap_or_else(|| format!("unknown_{}", node_id));
                    
                    conflicts.push(ConflictInfo {
                        node_id: node_id.clone(),
                        path,
                        base_oid: None,
                        ours_oid: ours_oid.clone(),
                        theirs_oid: theirs_oid.clone(),
                    });
                }
                (Some(_), Some(_o), None) | (Some(_), None, Some(_o)) => {
                    let path = self.find_path_for_node(&ours_manifest, &node_id)
                        .or_else(|| self.find_path_for_node(&theirs_manifest, &node_id))
                        .unwrap_or_else(|| format!("unknown_{}", node_id));
                    
                    conflicts.push(ConflictInfo {
                        node_id: node_id.clone(),
                        path,
                        base_oid: base_oid.clone(),
                        ours_oid: ours_oid.clone(),
                        theirs_oid: theirs_oid.clone(),
                    });
                }
                _ => {}
            }
        }

        if !conflicts.is_empty() {
            return Ok(MergeResult::Conflicts { conflicts });
        }

        let new_changeset_id = uuid::Uuid::new_v4().to_string();
        Ok(MergeResult::Clean { new_changeset_id })
    }

    fn load_manifest(&self, oid: &str) -> Result<Manifest> {
        let data = self.storage.read(oid).context("Failed to read manifest")?;
        let manifest: Manifest = serde_json::from_slice(&data).context("Failed to deserialize manifest")?;
        Ok(manifest)
    }

    fn collect_all_node_ids(
        &self,
        base: &Manifest,
        ours: &Manifest,
        theirs: &Manifest,
    ) -> HashSet<NodeId> {
        let mut all_ids = HashSet::new();
        for entry in base.entries.values() {
            all_ids.insert(entry.node_id.clone());
        }
        for entry in ours.entries.values() {
            all_ids.insert(entry.node_id.clone());
        }
        for entry in theirs.entries.values() {
            all_ids.insert(entry.node_id.clone());
        }
        all_ids
    }

    fn find_path_for_node(&self, manifest: &Manifest, node_id: &NodeId) -> Option<String> {
        manifest
            .entries
            .iter()
            .find(|(_, entry)| entry.node_id == *node_id)
            .map(|(path, _)| path.clone())
    }
}
