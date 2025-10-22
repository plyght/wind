use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use wind_core::UnifiedRepository;

pub async fn execute(path: String) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Importing from Git repository...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let git_path = PathBuf::from(&path);
    
    if !git_path.join(".git").exists() {
        anyhow::bail!("Not a Git repository: {}", path);
    }

    let _repo = UnifiedRepository::import_git(git_path)?;

    pb.finish_with_message(format!(
        "{} Imported Git repository from {}",
        "✓".green(),
        path.bold()
    ));

    println!("\n{}", "Wind repository created. You can now use wind commands.".dimmed());

    Ok(())
}
