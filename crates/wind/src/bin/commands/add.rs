use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use wind::UnifiedRepository;

pub async fn execute(files: Vec<String>, all: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut repo = UnifiedRepository::open(current_dir)?;

    if all {
        anyhow::bail!("--all not yet implemented, please specify files");
    } else if files.is_empty() {
        anyhow::bail!("No files specified. Use -a/--all to add all changes.");
    } else {
        let paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
        repo.add(paths)?;
        println!("{} Added {} file(s)", "✓".green(), files.len());
    }

    Ok(())
}
