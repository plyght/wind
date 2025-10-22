use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitSha(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindOid(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOp {
    pub op_type: OpType,
    pub path: String,
    pub node_id: Option<NodeId>,
    pub content: Option<Vec<u8>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    Add,
    Edit,
    Delete,
    Rename { from: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changeset {
    pub oid: WindOid,
    pub parent: Option<WindOid>,
    pub message: String,
    pub author: String,
    pub timestamp: i64,
    pub ops: Vec<FileOp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: String,
    pub node_id: NodeId,
    pub content: Vec<u8>,
}
