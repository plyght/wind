# Wind Bridge

Bidirectional Git ↔ Wind VCS bridge for seamless repository synchronization.

## Features

- **Import Git → Wind**: Convert Git commits to Wind changesets
- **Export Wind → Git**: Create Git commits from Wind operations
- **Mapping Database**: SQLite-based tracking of Git SHAs ↔ Wind OIDs
- **NodeID Tracking**: Stable file identity across renames
- **Auto-Sync Hooks**: Git hooks for automatic synchronization
- **Rename Detection**: 80%+ similarity threshold for renames

## Architecture

```
Git Repository           Wind Repository
    ↓                           ↑
GitImporter         →    Changesets
    ↓                           ↓
MappingDatabase     ←    NodeIDs/OIDs
    ↑                           ↑
GitExporter         ←    Operations
    ↑                           ↓
Git Commits                 Manifests
```

## Components

### Importer (Git → Wind)

```rust
use wind_bridge::GitImporter;

let mut importer = GitImporter::new("path/to/repo", "path/to/db")?;
let changesets = importer.import_all()?;
```

Extracts operations from Git diffs:
- **Add**: New file detection
- **Edit**: Content modification
- **Delete**: File removal
- **Rename**: Path changes (80%+ similarity)

### Exporter (Wind → Git)

```rust
use wind_bridge::GitExporter;

let mut exporter = GitExporter::new("path/to/repo", "path/to/db")?;
let git_sha = exporter.export_changeset(&changeset)?;
```

Creates Git commits from Wind changesets:
- Materializes manifest as Git tree
- Preserves authorship and timestamps
- Tracks bidirectional mapping

### Mapping Database

Schema:
```sql
sha_oid_mapping:     git_sha ↔ wind_oid
node_path_mapping:   node_id ↔ current_path
path_history:        node_id history over time
```

### Git Hooks

Install with:
```bash
wind sync --install
```

Hooks installed:
- **post-commit**: Import new commits
- **post-merge**: Import merge commits
- **post-checkout**: Sync on branch switch

### Sync Command

```bash
wind sync              # Full bidirectional sync
wind sync --quiet      # Silent mode
wind sync --install    # Install hooks only
```

## Performance

- **Import**: ~1000 commits/sec (small files)
- **Export**: ~800 commits/sec (avg file size)
- **Rename Detection**: O(n²) with early exit
- **Database**: Indexed lookups, O(1) avg

## Database Location

`.wind/bridge/mapping.db` (SQLite with bundled driver)

## Dependencies

- `git2`: Git operations (libgit2 bindings)
- `rusqlite`: SQLite database with bundled driver
- `anyhow`: Error handling
- `tracing`: Logging

## Testing

```bash
cargo test --package wind-bridge
```

Tests cover:
- Full import workflow
- Database mapping operations
- NodeID tracking
- Rename detection
