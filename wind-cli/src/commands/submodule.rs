use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: crate::SubmoduleAction) -> Result<()> {
    use crate::SubmoduleAction;

    match action {
        SubmoduleAction::List => list().await,
        SubmoduleAction::Status => status().await,
        SubmoduleAction::Init { name } => init(name).await,
        SubmoduleAction::Update { name } => update(name).await,
    }
}

async fn list() -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;
    let submodules = repo.list_submodules()?;

    if submodules.is_empty() {
        println!("{}", "No submodules found".dimmed());
        return Ok(());
    }

    println!("{}", "Submodules:".bold());
    for sub in submodules {
        let status = if sub.initialized {
            "initialized".green()
        } else {
            "not initialized".yellow()
        };

        println!("  {} {} ({})", sub.name.cyan(), sub.path.display(), status);
        println!("    {}: {}", "url".dimmed(), sub.url.dimmed());
    }

    Ok(())
}

async fn status() -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    if repo.is_inside_submodule()? {
        println!(
            "{}",
            "Warning: You are inside a submodule. Navigate to the root repository.".yellow()
        );
    }

    let submodules = repo.list_submodules()?;

    if submodules.is_empty() {
        println!("{}", "No submodules configured".dimmed());
        return Ok(());
    }

    println!("{}", "Submodule status:".bold());
    for sub in submodules {
        let status_str = if sub.initialized {
            "✓ initialized".green()
        } else {
            "✗ not initialized".yellow()
        };

        println!(
            "  {} {} - {}",
            sub.name.cyan(),
            sub.path.display(),
            status_str
        );
    }

    Ok(())
}

async fn init(name: Option<String>) -> Result<()> {
    println!("{}", "Submodule init requires git CLI integration".yellow());
    if let Some(n) = name {
        println!("Use: git submodule init {}", n);
    } else {
        println!("Use: git submodule init");
    }
    Ok(())
}

async fn update(name: Option<String>) -> Result<()> {
    println!(
        "{}",
        "Submodule update requires git CLI integration".yellow()
    );
    if let Some(n) = name {
        println!("Use: git submodule update {}", n);
    } else {
        println!("Use: git submodule update");
    }
    Ok(())
}
