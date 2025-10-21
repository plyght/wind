use anyhow::Result;
use colored::Colorize;

pub async fn execute() -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    println!("{}", "Launching Wind TUI...".cyan());
    wind_tui::run(&repo).await?;

    Ok(())
}
