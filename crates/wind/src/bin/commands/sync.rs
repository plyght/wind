use anyhow::Result;
use colored::Colorize;
use wind::UnifiedRepository;

pub fn handle_sync(quiet: bool, _install: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    if !current_dir.join(".wind").exists() {
        anyhow::bail!("Not a Wind repository. Run 'wind init' first.");
    }

    if !current_dir.join(".git").exists() {
        if !quiet {
            println!("{}", "No .git directory found".yellow());
        }
        return Ok(());
    }

    let mut repo = UnifiedRepository::open(current_dir)?;

    if !quiet {
        println!("{}", "Syncing with Git...".cyan());
    }

    repo.sync_with_git()?;

    if !quiet {
        println!("{}", "Synced with .git".green());
    }

    Ok(())
}
