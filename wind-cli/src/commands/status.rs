use anyhow::Result;
use colored::Colorize;

pub async fn execute() -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;
    let status = repo.status()?;

    println!("{}", "On branch".bold());
    println!("  {}", status.branch.green());

    if status.is_worktree {
        println!("{}", "  (worktree)".cyan());
    }

    if !status.submodules.is_empty() {
        println!("\n{}", "Submodules:".cyan().bold());
        for sub in &status.submodules {
            let status_str = if sub.initialized {
                "initialized".green()
            } else {
                "not initialized".yellow()
            };
            println!(
                "  {} {} ({})",
                sub.name.cyan(),
                sub.path.display(),
                status_str
            );
        }
    }

    if !status.staged.is_empty() {
        println!("\n{}", "Changes to be committed:".green().bold());
        for file in &status.staged {
            println!("  {} {}", "modified:".green(), file);
        }
    }

    if !status.modified.is_empty() {
        println!("\n{}", "Changes not staged for commit:".red().bold());
        for file in &status.modified {
            println!("  {} {}", "modified:".red(), file);
        }
    }

    if !status.untracked.is_empty() {
        println!("\n{}", "Untracked files:".yellow().bold());
        for file in &status.untracked {
            println!("  {}", file.yellow());
        }
    }

    if status.staged.is_empty() && status.modified.is_empty() && status.untracked.is_empty() {
        println!("\n{}", "nothing to commit, working tree clean".dimmed());
    }

    Ok(())
}
