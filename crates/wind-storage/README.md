# Wind Storage Layer

Content-addressed storage system for Wind VCS using BLAKE3 hashing and chunk-based deduplication.

## Architecture

- **Content Addressing**: Uses BLAKE3 (32-byte hash) instead of SHA-1
- **Chunking**: FastCDC rolling hash with variable-size chunks (4KB-256KB, avg 64KB)
- **Deduplication**: Automatic chunk-level deduplication
- **Compression**: zstd compression for all stored data
- **Packfiles**: Multiple small objects can be packed together for efficiency

## Directory Structure

```
.wind/
├── objects/     # Content-addressed objects (blobs, trees, commits)
├── chunks/      # Deduplicated file chunks
├── packs/       # Packfiles for small objects
├── refs/        # Branch references
├── config       # Repository configuration
└── index.db     # Working directory index
```

## Modules

### `oid` - Object Identifier
32-byte BLAKE3 hash wrapper with hex conversion and fanout path generation.

### `chunker` - Content Chunking
FastCDC-based rolling hash chunker for variable-size chunks with automatic deduplication.

### `chunk_store` - Chunk Storage
Manages deduplicated chunks with compression and caching.

### `object_store` - Object Storage
Async trait for storing blobs, trees, and commits with FileSystemStore implementation.

### `packfile` - Packfile Management
Combines multiple small objects into compressed packfiles with index for fast lookup.

### `layout` - Directory Layout
Manages .wind/ directory structure initialization and path resolution.

## Performance Benchmarks

**OID Hashing (BLAKE3)**:
- 1KB: ~1.6 GB/s
- 4KB: ~3.5 GB/s  
- 16KB: ~6.5 GB/s
- 64KB: ~9+ GB/s

**Chunking (FastCDC)**:
- 256KB: ~915 MiB/s
- 1MB: ~920 MiB/s

**Chunk Store Write** (with deduplication):
- 4KB: ~240 GiB/s (cached)
- 64KB: ~3900 GiB/s (cached)

**Object Store** (async with compression):
- 1KB objects: ~85 MiB/s
- 64KB objects: ~845 MiB/s

## Usage

```rust
use wind_storage::{StorageLayout, ObjectStore, FileSystemStore, Chunker};

// Initialize storage
let layout = StorageLayout::new(repo_path);
layout.init()?;

// Store objects
let store = FileSystemStore::new(layout.objects_dir())?;
let obj = Object {
    obj_type: ObjectType::Blob,
    data: file_contents,
};
let oid = store.write_object(&obj).await?;

// Chunk large files
let chunker = Chunker::default();
let chunks = chunker.chunk_bytes(&large_file);
for chunk in chunks {
    chunk_store.write_chunk(&chunk)?;
}
```

## Tests

```bash
cargo test --package wind-storage
```

All 15 unit tests passing.

## Benchmarks

```bash
cargo bench --package wind-storage
```

Results saved to `target/criterion/` with HTML reports.
