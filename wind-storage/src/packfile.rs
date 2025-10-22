use crate::object_store::Object;
use crate::Oid;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Default)]
pub struct PackFile {
    objects: Vec<PackedObject>,
}

#[derive(Serialize, Deserialize)]
struct PackedObject {
    oid: Oid,
    offset: u64,
    size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct PackIndex {
    entries: HashMap<Oid, PackEntry>,
    pack_path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
struct PackEntry {
    offset: u64,
    size: usize,
}

impl PackFile {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_object(&mut self, oid: Oid, data: &[u8]) -> u64 {
        let offset = self
            .objects
            .iter()
            .map(|o| o.offset + o.size as u64)
            .max()
            .unwrap_or(0);

        self.objects.push(PackedObject {
            oid,
            offset,
            size: data.len(),
        });

        offset
    }

    pub fn write(&self, pack_dir: &Path, data: &[u8]) -> Result<(PathBuf, PackIndex)> {
        std::fs::create_dir_all(pack_dir)?;

        let pack_id = Oid::hash_bytes(data);
        let pack_path = pack_dir.join(format!("pack-{}.pack", pack_id.to_hex()));
        let index_path = pack_dir.join(format!("pack-{}.idx", pack_id.to_hex()));

        let compressed = zstd::encode_all(data, 3)?;
        std::fs::write(&pack_path, compressed)?;

        let mut entries = HashMap::new();
        for obj in &self.objects {
            entries.insert(
                obj.oid,
                PackEntry {
                    offset: obj.offset,
                    size: obj.size,
                },
            );
        }

        let index = PackIndex {
            entries,
            pack_path: pack_path.clone(),
        };

        let index_data = bincode::serialize(&index)?;
        std::fs::write(&index_path, index_data)?;

        Ok((pack_path, index))
    }
}

impl PackIndex {
    pub fn load(path: &Path) -> Result<Self> {
        let data = std::fs::read(path)?;
        let index = bincode::deserialize(&data)?;
        Ok(index)
    }

    pub fn lookup(&self, oid: &Oid) -> Option<(u64, usize)> {
        self.entries.get(oid).map(|e| (e.offset, e.size))
    }

    pub fn read_object(&self, oid: &Oid) -> Result<Object> {
        let (offset, size) = self
            .lookup(oid)
            .ok_or_else(|| anyhow::anyhow!("Object not in pack"))?;

        let compressed = std::fs::read(&self.pack_path)?;
        let full_data = zstd::decode_all(&compressed[..])?;

        let obj_data = &full_data[offset as usize..(offset as usize + size)];
        let obj = bincode::deserialize(obj_data)?;
        Ok(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object_store::{Object, ObjectType};
    use tempfile::TempDir;

    #[test]
    fn test_packfile_write_read() {
        let temp = TempDir::new().unwrap();
        let pack_dir = temp.path().join("packs");

        let mut pack = PackFile::new();

        let obj1 = Object {
            obj_type: ObjectType::Blob,
            data: b"test1".to_vec(),
        };
        let encoded1 = bincode::serialize(&obj1).unwrap();
        let oid1 = Oid::hash_bytes(&encoded1);

        let obj2 = Object {
            obj_type: ObjectType::Blob,
            data: b"test2".to_vec(),
        };
        let encoded2 = bincode::serialize(&obj2).unwrap();
        let oid2 = Oid::hash_bytes(&encoded2);

        pack.add_object(oid1, &encoded1);
        pack.add_object(oid2, &encoded2);

        let mut all_data = Vec::new();
        all_data.extend_from_slice(&encoded1);
        all_data.extend_from_slice(&encoded2);

        let (_pack_path, index) = pack.write(&pack_dir, &all_data).unwrap();

        assert!(index.lookup(&oid1).is_some());
        assert!(index.lookup(&oid2).is_some());
    }
}
