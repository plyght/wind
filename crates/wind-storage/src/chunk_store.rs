use crate::{Chunk, Oid};
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ChunkStore {
    base_path: PathBuf,
    cache: HashMap<Oid, Vec<u8>>,
}

impl ChunkStore {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&base_path)?;
        Ok(Self {
            base_path,
            cache: HashMap::new(),
        })
    }

    pub fn write_chunk(&mut self, chunk: &Chunk) -> Result<()> {
        if self.has_chunk(&chunk.oid)? {
            return Ok(());
        }

        let (dir, file) = chunk.oid.fanout_path();
        let dir_path = self.base_path.join(&dir);
        std::fs::create_dir_all(&dir_path)?;

        let file_path = dir_path.join(&file);
        let compressed = zstd::encode_all(&chunk.data[..], 3)?;
        std::fs::write(&file_path, compressed)?;

        self.cache.insert(chunk.oid, chunk.data.clone());
        Ok(())
    }

    pub fn read_chunk(&mut self, oid: &Oid) -> Result<Vec<u8>> {
        if let Some(data) = self.cache.get(oid) {
            return Ok(data.clone());
        }

        let (dir, file) = oid.fanout_path();
        let file_path = self.base_path.join(&dir).join(&file);

        let compressed = std::fs::read(&file_path)?;
        let data = zstd::decode_all(&compressed[..])?;

        self.cache.insert(*oid, data.clone());
        Ok(data)
    }

    pub fn has_chunk(&self, oid: &Oid) -> Result<bool> {
        if self.cache.contains_key(oid) {
            return Ok(true);
        }

        let (dir, file) = oid.fanout_path();
        let file_path = self.base_path.join(&dir).join(&file);
        Ok(file_path.exists())
    }

    pub fn stats(&self) -> ChunkStats {
        ChunkStats {
            cached_chunks: self.cache.len(),
        }
    }
}

pub struct ChunkStats {
    pub cached_chunks: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_read_chunk() {
        let temp = TempDir::new().unwrap();
        let mut store = ChunkStore::new(temp.path().join("chunks")).unwrap();

        let data = b"test chunk data".to_vec();
        let oid = Oid::hash_bytes(&data);
        let chunk = Chunk {
            data: data.clone(),
            oid,
            offset: 0,
            length: data.len(),
        };

        store.write_chunk(&chunk).unwrap();
        let read_data = store.read_chunk(&oid).unwrap();
        assert_eq!(data, read_data);
    }

    #[test]
    fn test_deduplication() {
        let temp = TempDir::new().unwrap();
        let mut store = ChunkStore::new(temp.path().join("chunks")).unwrap();

        let data = b"duplicate data".to_vec();
        let oid = Oid::hash_bytes(&data);
        let chunk = Chunk {
            data: data.clone(),
            oid,
            offset: 0,
            length: data.len(),
        };

        store.write_chunk(&chunk).unwrap();
        store.write_chunk(&chunk).unwrap();

        assert!(store.has_chunk(&oid).unwrap());
    }
}
