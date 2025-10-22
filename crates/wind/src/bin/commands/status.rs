use anyhow::Result;
use colored::Colorize;
use wind::{FileStatus, UnifiedRepository};

pub async fn execute() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join(".wind").exists() {
        if current_dir.join(".git").exists() {
            println!(
                "{}",
                "Git repository detected. Use 'wind import-git .' to import.".yellow()
            );
            return Ok(());
        } else {
            anyhow::bail!("Not a Wind repository");
        }
    }

    let repo = UnifiedRepository::open(current_dir)?;
    let changes = repo.status()?;

    if changes.is_empty() {
        println!("{}", "nothing to commit, working tree clean".dimmed());
        return Ok(());
    }

    println!("{}", "On branch main".bold());
    println!();

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    let mut renamed = Vec::new();
    let mut untracked = Vec::new();

    for change in changes {
        match change.status {
            FileStatus::Added => added.push(change),
            FileStatus::Modified => modified.push(change),
            FileStatus::Deleted => deleted.push(change),
            FileStatus::Renamed { .. } => renamed.push(change),
            FileStatus::Untracked => untracked.push(change),
        }
    }

    if !added.is_empty() || !modified.is_empty() || !deleted.is_empty() || !renamed.is_empty() {
        println!("{}", "Changes to be committed:".green().bold());

        for change in &renamed {
            if let FileStatus::Renamed { ref from, ref to } = change.status {
                let node_id = change.node_id.as_deref().unwrap_or("unknown");
                println!(
                    "  renamed:    {} -> {} (NodeID: {})",
                    from.display().to_string().dimmed(),
                    to.display().to_string().green(),
                    &node_id[..8].bright_blue()
                );
            }
        }

        for change in &modified {
            let node_id = change.node_id.as_deref().unwrap_or("unknown");
            println!(
                "  modified:   {} (NodeID: {})",
                change.path.display().to_string().yellow(),
                &node_id[..8].bright_blue()
            );
        }

        for change in &added {
            let node_id = change.node_id.as_deref().unwrap_or("unknown");
            println!(
                "  new file:   {} (NodeID: {})",
                change.path.display().to_string().green(),
                &node_id[..8].bright_blue()
            );
        }

        for change in &deleted {
            let node_id = change.node_id.as_deref().unwrap_or("unknown");
            println!(
                "  deleted:    {} (NodeID: {})",
                change.path.display().to_string().red(),
                &node_id[..8].bright_blue()
            );
        }

        println!();
    }

    if !untracked.is_empty() {
        println!("{}", "Untracked files:".dimmed());
        println!("  (use \"wind add <file>...\" to include in what will be committed)");
        println!();
        for change in &untracked {
            println!("        {}", change.path.display().to_string().red());
        }
        println!();
    }

    Ok(())
}
