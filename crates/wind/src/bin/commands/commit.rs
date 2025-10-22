use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use wind::UnifiedRepository;

pub async fn execute(message: Option<String>, ai: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let mut repo = UnifiedRepository::open(current_dir)?;

    let commit_message = if ai {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Generating commit message with AI...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        pb.finish_and_clear();
        println!(
            "{}",
            "AI commit message generation not yet implemented".yellow()
        );

        message.ok_or_else(|| anyhow::anyhow!("No commit message provided"))?
    } else {
        message.ok_or_else(|| anyhow::anyhow!("No commit message provided. Use -m or --ai"))?
    };

    let oid = repo.commit(&commit_message)?;

    println!(
        "{} Created changeset {}",
        "✓".green(),
        oid[..16].bright_yellow()
    );

    Ok(())
}
