use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn execute(onto: String) -> Result<()> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.magenta} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Rebasing onto {}", onto));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let repo = wind_core::repository::Repository::open(".")?;
    repo.rebase(&onto)?;

    pb.finish_with_message(format!("{} Successfully rebased", "✓".green()));

    Ok(())
}
