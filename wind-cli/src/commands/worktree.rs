use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: crate::WorktreeAction) -> Result<()> {
    use crate::WorktreeAction;

    match action {
        WorktreeAction::List => list().await,
        WorktreeAction::Add { path, branch } => add(path, branch).await,
        WorktreeAction::Remove { path } => remove(path).await,
    }
}

async fn list() -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;
    let worktrees = repo.list_worktrees()?;

    if worktrees.is_empty() {
        println!("{}", "No worktrees found".dimmed());
        return Ok(());
    }

    for wt in worktrees {
        let marker = if wt.is_main {
            "(main)".cyan()
        } else {
            "      ".normal()
        };

        let branch_str = wt
            .branch
            .as_ref()
            .map(|b| b.green().to_string())
            .unwrap_or_else(|| "(detached)".yellow().to_string());

        println!(
            "{} {} {}",
            marker,
            wt.path.display().to_string().bold(),
            branch_str
        );
    }

    Ok(())
}

async fn add(path: String, branch: Option<String>) -> Result<()> {
    println!(
        "{}",
        "Worktree add functionality requires git CLI integration".yellow()
    );
    println!(
        "Use: git worktree add {} {}",
        path,
        branch.unwrap_or_default()
    );
    Ok(())
}

async fn remove(path: String) -> Result<()> {
    println!(
        "{}",
        "Worktree remove functionality requires git CLI integration".yellow()
    );
    println!("Use: git worktree remove {}", path);
    Ok(())
}
