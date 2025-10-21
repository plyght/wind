use anyhow::Result;
use colored::Colorize;

pub async fn execute(n: Option<usize>, graph: bool) -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;
    let commits = repo.log(n)?;

    for commit in commits {
        if graph {
            print!("* ");
        }

        println!("{} {}", "commit".yellow(), commit.id.bright_yellow());
        println!("{} {}", "Author:".dimmed(), commit.author);
        println!("{} {}", "Date:".dimmed(), commit.date);
        println!("\n    {}\n", commit.message);
    }

    Ok(())
}
