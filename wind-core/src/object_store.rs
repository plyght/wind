use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct ObjectStore {
    objects_dir: PathBuf,
    cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl ObjectStore {
    pub fn new(wind_dir: &Path) -> Result<Self> {
        let objects_dir = wind_dir.join("objects");
        fs::create_dir_all(&objects_dir).context("Failed to create objects directory")?;
        Ok(Self {
            objects_dir,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    fn object_path(&self, oid: &str) -> PathBuf {
        let (prefix, suffix) = oid.split_at(2);
        self.objects_dir.join(prefix).join(suffix)
    }

    pub fn write(&self, data: &[u8]) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(data);
        let oid = hex::encode(hasher.finalize());

        let path = self.object_path(&oid);

        if !path.exists() {
            fs::create_dir_all(path.parent().unwrap())?;
            fs::write(&path, data)?;
        }

        let mut cache = self.cache.write().unwrap();
        cache.insert(oid.clone(), data.to_vec());

        Ok(oid)
    }

    pub fn read(&self, oid: &str) -> Result<Vec<u8>> {
        {
            let cache = self.cache.read().unwrap();
            if let Some(data) = cache.get(oid) {
                return Ok(data.clone());
            }
        }

        let path = self.object_path(oid);
        let data = fs::read(&path).with_context(|| format!("Failed to read object {}", oid))?;

        let mut cache = self.cache.write().unwrap();
        cache.insert(oid.to_string(), data.clone());

        Ok(data)
    }

    pub fn exists(&self, oid: &str) -> bool {
        let path = self.object_path(oid);
        path.exists()
    }
}
