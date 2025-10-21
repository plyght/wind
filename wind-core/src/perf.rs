use anyhow::Result;
use git2::Repository as GitRepository;
use std::path::Path;

pub struct RepoInfo {
    pub is_large: bool,
    pub file_count: usize,
    pub commit_count: usize,
    pub repo_size_mb: f64,
}

pub fn analyze_repo(repo: &GitRepository) -> Result<RepoInfo> {
    let workdir = repo.workdir().unwrap_or(Path::new("."));

    let mut file_count = 0;
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(workdir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    file_count += 1;
                    total_size += metadata.len();
                }
            }
        }
    }

    let mut revwalk = repo.revwalk()?;
    let _ = revwalk.push_head();
    let commit_count = revwalk.count();

    let repo_size_mb = total_size as f64 / (1024.0 * 1024.0);
    let is_large = file_count > 10_000 || repo_size_mb > 1024.0;

    Ok(RepoInfo {
        is_large,
        file_count,
        commit_count,
        repo_size_mb,
    })
}

pub struct PerfConfig {
    pub cache_ttl_ms: u64,
    pub auto_refresh: bool,
    pub diff_context_lines: usize,
    pub log_page_size: usize,
    pub status_untracked: bool,
}

impl PerfConfig {
    pub fn default() -> Self {
        Self {
            cache_ttl_ms: 1000,
            auto_refresh: true,
            diff_context_lines: 3,
            log_page_size: 50,
            status_untracked: true,
        }
    }

    pub fn for_large_repo() -> Self {
        Self {
            cache_ttl_ms: 5000,
            auto_refresh: false,
            diff_context_lines: 1,
            log_page_size: 20,
            status_untracked: false,
        }
    }

    pub fn adjust_for_repo(info: &RepoInfo) -> Self {
        if info.is_large {
            Self::for_large_repo()
        } else {
            Self::default()
        }
    }
}
