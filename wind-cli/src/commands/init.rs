use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use wind_core::UnifiedRepository;

pub async fn execute(path: Option<String>) -> Result<()> {
    let target_path = path.unwrap_or_else(|| ".".to_string());
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Initializing Wind repository in {}", target_path));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let path = PathBuf::from(&target_path);
    UnifiedRepository::init(path)?;

    pb.finish_with_message(format!(
        "{} Initialized Wind repository in {}",
        "✓".green(),
        target_path.bold()
    ));

    println!("\n{}", "Next steps:".bold());
    println!("  wind add <files>");
    println!("  wind commit -m \"Initial commit\"");

    Ok(())
}
