use anyhow::Result;
use colored::Colorize;

pub async fn execute() -> Result<()> {
    let repo = wind::repository::Repository::open(".")?;

    println!("{}", "Launching Wind TUI...".cyan());
    wind::tui::run(&repo).await?;

    Ok(())
}
