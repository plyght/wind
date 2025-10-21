use std::path::Path;

pub use git2::{
    Branch, BranchType, Commit, DiffOptions, Index, IndexAddOption, Oid, Reference, Repository,
    Signature, Status, StatusOptions, Tree,
};

#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GitError>;

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            repo: Repository::open(path)?,
        })
    }

    pub fn discover<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            repo: Repository::discover(path)?,
        })
    }

    pub fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<Self> {
        Ok(Self {
            repo: if bare {
                Repository::init_bare(path)?
            } else {
                Repository::init(path)?
            },
        })
    }

    pub fn inner(&self) -> &Repository {
        &self.repo
    }

    pub fn inner_mut(&mut self) -> &mut Repository {
        &mut self.repo
    }

    pub fn path(&self) -> &Path {
        self.repo.path()
    }

    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    pub fn is_bare(&self) -> bool {
        self.repo.is_bare()
    }

    pub fn is_worktree(&self) -> bool {
        self.repo.is_worktree()
    }
}
