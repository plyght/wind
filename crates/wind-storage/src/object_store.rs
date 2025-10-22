use crate::Oid;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub trait SyncObjectStore: Send + Sync {
    fn write(&self, data: &[u8]) -> Result<String>;
    fn read(&self, oid: &str) -> Result<Vec<u8>>;
    fn exists(&self, oid: &str) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub obj_type: ObjectType,
    pub data: Vec<u8>,
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn write_object(&self, obj: &Object) -> Result<Oid>;
    async fn read_object(&self, oid: &Oid) -> Result<Object>;
    async fn has_object(&self, oid: &Oid) -> Result<bool>;
}

pub struct FileSystemStore {
    base_path: PathBuf,
}

impl FileSystemStore {
    pub fn new(base_path: &std::path::Path) -> Result<Self> {
        std::fs::create_dir_all(base_path)?;
        Ok(Self {
            base_path: base_path.to_path_buf(),
        })
    }

    fn object_path(&self, oid: &Oid) -> PathBuf {
        let (dir, file) = oid.fanout_path();
        self.base_path.join(dir).join(file)
    }
}

impl SyncObjectStore for FileSystemStore {
    fn write(&self, data: &[u8]) -> Result<String> {
        let oid = Oid::hash_bytes(data);
        let oid_str = oid.to_string();

        if self.exists(&oid_str) {
            return Ok(oid_str);
        }

        let (dir, _) = oid.fanout_path();
        let dir_path = self.base_path.join(&dir);
        std::fs::create_dir_all(&dir_path)?;

        let compressed = zstd::encode_all(data, 3)?;
        let path = self.object_path(&oid);
        std::fs::write(&path, compressed)?;

        Ok(oid_str)
    }

    fn read(&self, oid_str: &str) -> Result<Vec<u8>> {
        let oid = Oid::from_hex(oid_str)?;
        let path = self.object_path(&oid);
        let compressed = std::fs::read(&path)?;
        let data = zstd::decode_all(&compressed[..])?;
        Ok(data)
    }

    fn exists(&self, oid_str: &str) -> bool {
        if let Ok(oid) = Oid::from_hex(oid_str) {
            let path = self.object_path(&oid);
            path.exists()
        } else {
            false
        }
    }
}

#[async_trait]
impl ObjectStore for FileSystemStore {
    async fn write_object(&self, obj: &Object) -> Result<Oid> {
        let encoded = bincode::serialize(obj)?;
        let oid = Oid::hash_bytes(&encoded);

        if self.has_object(&oid).await? {
            return Ok(oid);
        }

        let (dir, _) = oid.fanout_path();
        let dir_path = self.base_path.join(&dir);
        tokio::fs::create_dir_all(&dir_path).await?;

        let compressed = zstd::encode_all(&encoded[..], 3)?;
        let path = self.object_path(&oid);
        tokio::fs::write(&path, compressed).await?;

        Ok(oid)
    }

    async fn read_object(&self, oid: &Oid) -> Result<Object> {
        let path = self.object_path(oid);
        let compressed = tokio::fs::read(&path).await?;
        let encoded = zstd::decode_all(&compressed[..])?;
        let obj = bincode::deserialize(&encoded)?;
        Ok(obj)
    }

    async fn has_object(&self, oid: &Oid) -> Result<bool> {
        let path = self.object_path(oid);
        Ok(tokio::fs::try_exists(&path).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_write_read_object() {
        let temp = TempDir::new().unwrap();
        let store = FileSystemStore::new(&temp.path().join("objects")).unwrap();

        let obj = Object {
            obj_type: ObjectType::Blob,
            data: b"test data".to_vec(),
        };

        let oid = store.write_object(&obj).await.unwrap();
        let read_obj = store.read_object(&oid).await.unwrap();

        assert_eq!(obj.data, read_obj.data);
    }

    #[tokio::test]
    async fn test_has_object() {
        let temp = TempDir::new().unwrap();
        let store = FileSystemStore::new(&temp.path().join("objects")).unwrap();

        let obj = Object {
            obj_type: ObjectType::Blob,
            data: b"test".to_vec(),
        };

        let oid = store.write_object(&obj).await.unwrap();
        assert!(store.has_object(&oid).await.unwrap());

        let fake_oid = Oid::hash_bytes(b"nonexistent");
        assert!(!store.has_object(&fake_oid).await.unwrap());
    }

    #[tokio::test]
    async fn test_compression() {
        let temp = TempDir::new().unwrap();
        let store = FileSystemStore::new(&temp.path().join("objects")).unwrap();

        let data = vec![0u8; 10000];
        let obj = Object {
            obj_type: ObjectType::Blob,
            data: data.clone(),
        };

        let oid = store.write_object(&obj).await.unwrap();
        let path = store.object_path(&oid);
        let file_size = std::fs::metadata(&path).unwrap().len();

        assert!(file_size < data.len() as u64);
    }
}
