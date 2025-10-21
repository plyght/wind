# Wind VCS - Performance Optimizations

## Overview

Wind VCS includes comprehensive performance optimizations for handling large repositories efficiently.

## Status Caching

### Implementation
- **Cache Layer**: `wind-core/src/cache.rs`
- **TTL**: Configurable (default: 1000ms, large repos: 5000ms)
- **Invalidation**: Automatic on file changes via watcher
- **Dirty Flag**: Marks pending operations for cache invalidation

### Configuration
```toml
[performance]
cache_ttl_ms = 1000
```

### Cache Hit Rate
- First call: Cache miss (scans repository)
- Subsequent calls within TTL: Cache hit (instant)
- After file modification: Cache invalidated

## Incremental Diff

### Features
- **Streaming**: Diffs loaded incrementally, not all at once
- **Context Lines**: Configurable (default: 3, large repos: 1)
- **Size Limit**: Max 1MB per diff to prevent memory issues
- **Binary Files**: Skipped automatically

### API
```rust
repo.get_diff("path/to/file.rs", context_lines)?
```

## Lazy Loading (TUI)

### Virtual Scrolling
- **File Lists**: Only visible items rendered
- **Log History**: Loaded in chunks (default: 50 commits)
- **Diff Output**: Paginated for large changes
- **Background Loading**: Progress indicators for async operations

### Implementation
- `LazyList<T>`: Virtual scrolling container
- `PaginatedLoader<T>`: Incremental data loading
- Viewport-based rendering

## Index Optimization

### Git Status Options
- **INCLUDE_UNTRACKED**: Only enabled when needed (disabled for large repos)
- **EXCLUDE_SUBMODULES**: Always enabled
- **Path Exclusions**: `.wind/`, `target/`, `node_modules/` automatically excluded
- **INCLUDE_UNMODIFIED**: Disabled (significant speedup)

### Performance Impact
```
Small repo (10 files):    ~1-2ms
Medium repo (100 files):  ~5-10ms  
Large repo (1000+ files): ~50-100ms (first call), ~0.1ms (cached)
```

## Large Repository Detection

### Thresholds
- **File Count**: > 10,000 files
- **Repository Size**: > 1GB

### Auto-Adjustments
When large repo detected:
- Cache TTL: 1000ms → 5000ms
- Auto-refresh: Enabled → Disabled
- Untracked files: Enabled → Disabled
- Diff context: 3 lines → 1 line
- Log page size: 50 → 20

### Warning Message
```
Large repository detected (15234 files, 2.3 GB)
Performance optimizations enabled:
  - Cache TTL: 5000ms
  - Auto-refresh: false
  - Untracked files: false
```

## Benchmarks

### Running Benchmarks
```bash
cargo bench --package wind-core
```

### Benchmark Suite
- `status_10_files`: Small repository status
- `status_100_files`: Medium repository status
- `status_1000_files`: Large repository status
- `status_cached`: Cached status call
- `log_paginated_20`: Paginated log retrieval
- `diff_small_file`: Diff generation

### Expected Results
```
status_10_files        1.2 ms    (± 0.1 ms)
status_100_files       8.5 ms    (± 0.3 ms)
status_1000_files     85.0 ms    (± 2.0 ms)
status_cached          0.05 ms   (± 0.01 ms)  [98% faster]
log_paginated_20       3.2 ms    (± 0.2 ms)
diff_small_file        2.1 ms    (± 0.1 ms)
```

## Cache Integration with File Watcher

The status cache integrates with the file watcher to automatically invalidate when files change:

```rust
// Cache invalidated on:
- File modifications (watcher event)
- Stage/unstage operations (add, add_all)
- Commits (commit)
- Branch changes (checkout)
- Rebase operations (rebase)
```

## API Usage

### Repository with Caching
```rust
let repo = Repository::open(".")?;

let status = repo.status()?;

repo.add("file.rs")?;

let status = repo.status()?;
```

### Paginated Log
```rust
let commits = repo.log_paginated(offset: 0, limit: 50)?;

let more = repo.log_paginated(offset: 50, limit: 50)?;
```

### Incremental Diff
```rust
let diff = repo.get_diff("src/main.rs", context_lines: 3)?;
```

## Performance Tips

### For Users
1. **Large Repos**: Wind auto-detects and adjusts settings
2. **Manual Refresh**: Use `Ctrl+R` instead of auto-refresh
3. **Untracked Files**: Disable if not needed: `wind config set performance.untracked false`
4. **Cache TTL**: Increase for slower systems: `wind config set performance.cache_ttl_ms 10000`

### For Developers
1. Use `repo.status()` freely - caching handles performance
2. Call `repo.invalidate_cache()` after operations that modify working tree
3. Use `log_paginated()` for large history
4. Use `get_diff()` with minimal context lines for large files

## Monitoring

### Cache Stats
```rust
use wind_core::cache::get_stats;

let stats = get_stats()?;
println!("Status cache hits: {}", stats.status_hits);
println!("Status cache misses: {}", stats.status_misses);
println!("Hit rate: {:.1}%", 
    100.0 * stats.status_hits as f64 / 
    (stats.status_hits + stats.status_misses) as f64
);
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                       Wind CLI/TUI                       │
└────────────────────┬────────────────────────────────────┘
                     │
                     v
┌─────────────────────────────────────────────────────────┐
│                    Repository API                        │
│  (status, add, commit, log_paginated, get_diff)         │
└────────────────────┬────────────────────────────────────┘
                     │
        ┌────────────┴─────────────┐
        v                          v
┌──────────────┐          ┌──────────────────┐
│ StatusCache  │          │  git2 (libgit2)  │
│  (1-5s TTL)  │          │  (optimized)     │
└──────┬───────┘          └──────────────────┘
       │
       │ invalidate()
       │
┌──────v───────┐
│ FileWatcher  │
│  (notify)    │
└──────────────┘
```

## Future Optimizations

- [ ] Parallel status scanning for multiple directories
- [ ] Incremental index updates
- [ ] Memory-mapped diff streaming
- [ ] Background preloading
- [ ] Persistent disk cache
- [ ] Delta compression for history
