# File Watching in Wind VCS

## Overview

Wind includes automatic file watching capability for TUI auto-refresh. The system uses the `notify` crate to monitor working directory changes and debounces events to prevent spam.

## Architecture

### Components

1. **FileWatcher** (`crates/wind/src/watcher.rs`)
   - Watches working directory recursively
   - Filters out `.git/` and `.wind/` changes
   - Debounces events with 300ms delay
   - Sends events via unbounded tokio channel

2. **Config** (`crates/wind/src/config.rs`)
   - `ui.auto_refresh` setting (default: `true`)
   - Stored in `.wind/config.toml`

3. **Repository Integration**
   - `watch(auto_refresh: bool) -> Result<Option<FileWatcher>>`
   - Returns `None` if auto-refresh disabled
   - Watches repository working directory

## Usage

### Basic Setup

```rust
use wind::{config::Config, Repository};

let repo = Repository::open(".")?;
let config = Config::load(repo.workdir())?;

if let Some(mut watcher) = repo.watch(config.ui.auto_refresh)? {
    while let Some(event) = watcher.recv().await {
        // Refresh UI
        let status = repo.status()?;
        update_display(&status);
    }
}
```

### TUI Integration

```rust
use wind::{config::Config, FileEvent, Repository};
use tokio::select;

pub async fn run_tui(repo: &Repository) -> Result<()> {
    let config = Config::load(repo.workdir())?;
    let mut watcher = repo.watch(config.ui.auto_refresh)?;
    
    loop {
        select! {
            event = user_input() => {
                handle_user_input(event)?;
            }
            Some(FileEvent::Changed) = async {
                match &mut watcher {
                    Some(w) => w.recv().await,
                    None => std::future::pending().await,
                }
            } => {
                refresh_display(repo)?;
            }
        }
    }
}
```

## Configuration

### Enable/Disable Auto-Refresh

**Enable (default):**
```bash
wind config set ui.auto_refresh true
```

**Disable:**
```bash
wind config set ui.auto_refresh false
```

**Configuration file (`.wind/config.toml`):**
```toml
[ui]
auto_refresh = true
```

## Performance Characteristics

### Debounce Behavior

- **Delay:** 300ms between events
- **Rationale:** Prevents UI thrashing during bulk file operations
- **Trade-off:** Slight delay vs. CPU/battery efficiency

### Resource Usage

- **Memory:** ~100KB per FileWatcher instance
- **CPU:** Minimal (kernel-level filesystem events)
- **Battery:** Negligible on macOS (FSEvents), minimal on Linux (inotify)

### Filtered Events

The watcher **ignores:**
- `.git/` directory changes
- `.wind/` directory changes
- Non-file events (metadata, access times)

The watcher **processes:**
- File creation
- File modification
- File deletion

### Platform-Specific Backends

| Platform | Backend | Notes |
|----------|---------|-------|
| macOS | FSEvents | Native, battery-efficient |
| Linux | inotify | Kernel-level, reliable |
| Windows | ReadDirectoryChangesW | Native Windows API |

## Testing

### Unit Tests

```bash
cargo test --package wind --lib watcher
```

Tests verify:
- File change detection
- `.git/` filtering
- `.wind/` filtering
- Event debouncing

### Integration Testing

1. Start TUI in one terminal:
   ```bash
   wind tui
   ```

2. Edit files in another terminal:
   ```bash
   echo "test" > test.txt
   ```

3. Verify TUI auto-refreshes within 300ms

## Limitations

1. **Large Repositories:** May have higher latency with 100k+ files
2. **Network Filesystems:** Limited support on NFS/SMB mounts
3. **Symbolic Links:** May not detect changes through symlinks

## Troubleshooting

### TUI Not Auto-Refreshing

1. Check config:
   ```bash
   wind config get ui.auto_refresh
   ```

2. Verify file system supports watching
3. Check system file descriptor limits (Linux)

### High CPU Usage

1. Disable auto-refresh:
   ```bash
   wind config set ui.auto_refresh false
   ```

2. Check for runaway file creation loops
3. Verify `.git/` and `.wind/` are filtered

## Future Enhancements

- [ ] Configurable debounce delay
- [ ] Selective path watching (ignore patterns)
- [ ] Performance metrics/monitoring
- [ ] Graceful degradation for unsupported filesystems
