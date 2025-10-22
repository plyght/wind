# Wind VCS Architecture

## Crates

### wind-git
Thin wrapper around libgit2 (git2-rs) providing low-level Git operations.

**Key APIs:**
- `GitRepository`: Wrapper around git2::Repository with convenience methods
- `open()`, `discover()`, `init()`: Repository lifecycle
- Re-exports git2 types: Oid, Commit, Branch, Status, etc.
- Error handling with `GitError` thiserror-based type

### wind  
Combined core engine, CLI, and TUI built on top of wind-git.

**Key APIs:**

1. **Repository (`repository.rs`)**
   - `Repository::init()`: Initialize with .wind directory
   - `Repository::open()`: Discover and open existing repo
   - `status()`: Get working tree status
   - `add()`, `commit()`: Stage and commit operations
   - `create_branch()`, `checkout()`: Branch operations
   - `rebase()`: Rebase current branch onto target
   - `config_get/set/list()`: Configuration management

2. **Stack Engine (`stack.rs`)**
   - `BranchMetadata`: Stores parent/child relationships in `.wind/stacks/`
   - `StackEngine::compute_stack()`: Build stack from branch metadata
   - `create_child_branch()`: Create branch with parent tracking
   - `rebase_stack()`: Rebase entire stack maintaining relationships
   - `promote()`: Move branch up in stack hierarchy
   - `land()`: Merge stack branch to target

## Storage Layout

- Uses `.git` if exists, otherwise creates `.wind` as gitdir
- `.git` file with `gitdir: ./.wind` points to Wind storage
- Wind metadata in `.wind/` subdirectories:
  - `.wind/stacks/`: Branch relationship metadata (JSON)
  - `.wind/config.toml`: Wind-specific config

## Design Decisions

1. **Git Compatibility**: Full bidirectional compatibility maintained by using libgit2
2. **Async-ready**: All Repository trait methods use `async fn` (currently sync internally)
3. **Error Handling**: Uses anyhow::Result for the wind crate, thiserror for wind-git
4. **Stack Metadata**: Stored as JSON in .wind/stacks/, serialized with serde
5. **Workspace Structure**: Cargo workspace with shared dependencies

## Dependencies

- `git2 = "0.19"`: libgit2 bindings for Git operations  
- `tokio = "1.42"`: Async runtime (prepared for future async I/O)
- `anyhow = "1.0"`: Error handling in the wind crate
- `thiserror = "2.0"`: Error types in wind-git
- `serde/serde_json`: Stack metadata serialization
- `notify = "6.1"`: File watching (prepared for future use)

## Next Steps

1. Implement conflict resolution API
2. Add file watcher integration  
3. Implement stash operations
4. Add merge operation
5. Handle edge cases: bare repos, worktrees, submodules
