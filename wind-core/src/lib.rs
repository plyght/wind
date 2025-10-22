pub mod cache;
pub mod conflict;
pub mod diff;
pub mod index;
pub mod merge;
pub mod model;
pub mod object_store;
pub mod perf;
pub mod repository;
pub mod stack;
pub mod submodule;
pub mod unified_repository;
pub mod watcher;
pub mod working_copy;
pub mod worktree;

pub use conflict::{ConflictContent, ConflictFile, ConflictResolver};
pub use diff::{DiffEngine, DiffHunk, DiffLine, DiffType, FileDiff, LineChange};
pub use index::{get_mtime, Index, IndexEntry};
pub use merge::{ConflictInfo, MergeEngine, MergeResult};
pub use model::{
    Branch, BranchId, Changeset, FileChange as ModelFileChange, Manifest, ManifestEntry, NodeId,
};
pub use object_store::ObjectStore;
pub use repository::{Commit, Repository, Status, SubmoduleStatus};
pub use submodule::Submodule;
pub use unified_repository::UnifiedRepository;
pub use watcher::{FileEvent, FileWatcher};
pub use working_copy::{FileChange, FileStatus, WorkingCopy};
pub use worktree::Worktree;

pub type OID = String;
