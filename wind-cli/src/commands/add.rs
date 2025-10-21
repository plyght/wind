use anyhow::Result;
use colored::Colorize;

pub async fn execute(files: Vec<String>, all: bool) -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    if all {
        repo.add_all()?;
        println!("{} Added all changes", "✓".green());
    } else if files.is_empty() {
        anyhow::bail!("No files specified. Use -a/--all to add all changes.");
    } else {
        for file in &files {
            repo.add(file)?;
        }
        println!("{} Added {} file(s)", "✓".green(), files.len());
    }

    Ok(())
}
