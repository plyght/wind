use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::repository::Status;

#[derive(Clone)]
pub struct StatusCache {
    inner: Arc<Mutex<StatusCacheInner>>,
}

struct StatusCacheInner {
    cache: HashMap<PathBuf, CachedStatus>,
    ttl: Duration,
    dirty: bool,
}

struct CachedStatus {
    status: Status,
    timestamp: Instant,
}

impl StatusCache {
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(StatusCacheInner {
                cache: HashMap::new(),
                ttl: Duration::from_millis(ttl_ms),
                dirty: false,
            })),
        }
    }

    pub fn get(&self, path: &PathBuf) -> Option<Status> {
        let inner = self.inner.lock().unwrap();
        if let Some(cached) = inner.cache.get(path) {
            if cached.timestamp.elapsed() < inner.ttl && !inner.dirty {
                return Some(cached.status.clone());
            }
        }
        None
    }

    pub fn set(&self, path: PathBuf, status: Status) {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.insert(
            path,
            CachedStatus {
                status,
                timestamp: Instant::now(),
            },
        );
        inner.dirty = false;
    }

    pub fn invalidate(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.dirty = true;
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.clear();
        inner.dirty = false;
    }

    pub fn set_ttl(&self, ttl_ms: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.ttl = Duration::from_millis(ttl_ms);
    }
}

pub struct DiffCache {
    inner: Arc<Mutex<DiffCacheInner>>,
}

struct DiffCacheInner {
    cache: HashMap<String, CachedDiff>,
    ttl: Duration,
}

struct CachedDiff {
    content: String,
    timestamp: Instant,
}

impl DiffCache {
    pub fn new(ttl_ms: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(DiffCacheInner {
                cache: HashMap::new(),
                ttl: Duration::from_millis(ttl_ms),
            })),
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        if let Some(cached) = inner.cache.get(key) {
            if cached.timestamp.elapsed() < inner.ttl {
                return Some(cached.content.clone());
            }
        }
        None
    }

    pub fn set(&self, key: String, content: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.insert(
            key,
            CachedDiff {
                content,
                timestamp: Instant::now(),
            },
        );
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.cache.clear();
    }
}

pub fn get_stats() -> Result<CacheStats> {
    Ok(CacheStats {
        status_hits: 0,
        status_misses: 0,
        diff_hits: 0,
        diff_misses: 0,
    })
}

pub struct CacheStats {
    pub status_hits: u64,
    pub status_misses: u64,
    pub diff_hits: u64,
    pub diff_misses: u64,
}
