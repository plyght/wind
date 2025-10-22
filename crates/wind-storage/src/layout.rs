use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct StorageLayout {
    root: PathBuf,
}

impl StorageLayout {
    pub fn new(repo_path: &Path) -> Self {
        Self {
            root: repo_path.join(".wind"),
        }
    }

    pub fn init(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(self.objects_dir())?;
        std::fs::create_dir_all(self.chunks_dir())?;
        std::fs::create_dir_all(self.packs_dir())?;
        std::fs::create_dir_all(self.refs_dir())?;

        let config_path = self.config_file();
        if !config_path.exists() {
            std::fs::write(&config_path, b"[core]\nversion = 1\n")?;
        }

        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.root.join("objects")
    }

    pub fn chunks_dir(&self) -> PathBuf {
        self.root.join("chunks")
    }

    pub fn packs_dir(&self) -> PathBuf {
        self.root.join("packs")
    }

    pub fn refs_dir(&self) -> PathBuf {
        self.root.join("refs")
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("config")
    }

    pub fn index_db(&self) -> PathBuf {
        self.root.join("index.db")
    }

    pub fn exists(&self) -> bool {
        self.root.exists() && self.config_file().exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_layout_init() {
        let temp = TempDir::new().unwrap();
        let layout = StorageLayout::new(temp.path());

        layout.init().unwrap();

        assert!(layout.exists());
        assert!(layout.objects_dir().exists());
        assert!(layout.chunks_dir().exists());
        assert!(layout.packs_dir().exists());
        assert!(layout.refs_dir().exists());
        assert!(layout.config_file().exists());
    }

    #[test]
    fn test_layout_paths() {
        let temp = TempDir::new().unwrap();
        let layout = StorageLayout::new(temp.path());

        assert!(layout.root().ends_with(".wind"));
        assert!(layout.objects_dir().ends_with("objects"));
        assert!(layout.chunks_dir().ends_with("chunks"));
        assert!(layout.packs_dir().ends_with("packs"));
        assert!(layout.refs_dir().ends_with("refs"));
        assert!(layout.config_file().ends_with("config"));
        assert!(layout.index_db().ends_with("index.db"));
    }
}
