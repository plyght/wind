use crate::PrAction;
use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: PrAction) -> Result<()> {
    let _repo = wind::repository::Repository::open(".")?;

    match action {
        PrAction::Create { title, body } => {
            let pr = wind_collab::pr::create(title, body).await?;
            println!(
                "{} Created PR #{}: {}",
                "✓".green(),
                pr.number,
                pr.url.bright_blue()
            );
        }
        PrAction::Update { number } => {
            wind_collab::pr::update(number).await?;
            println!("{} Updated PR #{}", "✓".green(), number);
        }
        PrAction::List => {
            let prs = wind_collab::pr::list().await?;
            if prs.is_empty() {
                println!("{}", "No pull requests found".dimmed());
            } else {
                for pr in prs {
                    println!(
                        "#{} {} [{}]",
                        pr.number.to_string().bright_yellow(),
                        pr.title.bold(),
                        pr.state.cyan()
                    );
                }
            }
        }
    }

    Ok(())
}
