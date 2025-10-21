use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn execute(message: Option<String>, ai: bool) -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    let commit_message = if ai {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Generating commit message with AI...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let ai_message = wind_ai::commit_message::generate(&repo).await?;
        pb.finish_and_clear();

        println!("{}", "Suggested commit message:".cyan().bold());
        println!("{}\n", ai_message.dimmed());
        println!("Use this message? [Y/n]: ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input.is_empty() || input == "y" || input == "yes" {
            ai_message
        } else {
            message.ok_or_else(|| anyhow::anyhow!("No commit message provided"))?
        }
    } else {
        message.ok_or_else(|| anyhow::anyhow!("No commit message provided. Use -m or --ai"))?
    };

    let commit_id = repo.commit(&commit_message)?;

    println!(
        "{} Created commit {}",
        "✓".green(),
        commit_id[..8].bright_yellow()
    );

    Ok(())
}
