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
    pb.set_message("Exporting to Git repository...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let current_dir = std::env::current_dir()?;
    let repo = UnifiedRepository::open(current_dir)?;
    
    let git_path = PathBuf::from(&path);
    repo.export_git(git_path)?;

    pb.finish_with_message(format!(
        "{} Exported to Git repository at {}",
        "✓".green(),
        path.bold()
    ));

    Ok(())
}
