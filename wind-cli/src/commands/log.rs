use anyhow::Result;
use colored::Colorize;
use wind_core::UnifiedRepository;

pub async fn execute(n: Option<usize>, graph: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let repo = UnifiedRepository::open(current_dir)?;
    let changesets = repo.log(n.unwrap_or(10))?;

    for changeset in changesets {
        if graph {
            print!("* ");
        }

        println!("{} {}", "changeset".yellow(), changeset.id[..16].bright_yellow());
        println!("{} {}", "Author:".dimmed(), changeset.author);
        println!("{} {}", "Timestamp:".dimmed(), changeset.timestamp);
        println!("\n    {}\n", changeset.commit_message);
    }

    Ok(())
}
