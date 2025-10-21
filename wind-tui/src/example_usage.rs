use anyhow::Result;
use wind_core::{Config, FileWatcher, Repository};

pub async fn example_tui_with_watcher(repo: &Repository) -> Result<()> {
    let config = Config::load(repo.workdir())?;
    
    let mut watcher_opt = repo.watch(config.ui.auto_refresh)?;
    
    if let Some(ref mut watcher) = watcher_opt {
        println!("File watching enabled. TUI will auto-refresh on file changes.");
        
        loop {
            if let Some(_event) = watcher.recv().await {
                println!("Files changed, refreshing...");
                let status = repo.status()?;
                println!("Branch: {}", status.branch);
                println!("Staged: {}", status.staged.len());
                println!("Modified: {}", status.modified.len());
                println!("Untracked: {}", status.untracked.len());
            }
        }
    } else {
        println!("Auto-refresh disabled via config.");
    }
    
    Ok(())
}
