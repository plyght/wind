use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use wind::UnifiedRepository;

pub async fn execute(target: String) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Switching to {}", target));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let current_dir = std::env::current_dir()?;
    let mut repo = UnifiedRepository::open(current_dir)?;
    repo.checkout(&target)?;

    pb.finish_with_message(format!(
        "{} Switched to branch {}",
        "✓".green(),
        target.bold()
    ));

    Ok(())
}
