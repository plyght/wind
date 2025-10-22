use crate::StackAction;
use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: StackAction) -> Result<()> {
    match action {
        StackAction::List => {
            println!("{}", "No stacks found".dimmed());
            println!(
                "{}",
                "Stack management not yet implemented for Wind VCS".yellow()
            );
        }
        StackAction::Create { name } => {
            println!(
                "{}",
                format!("Stack creation '{}' not yet implemented", name).yellow()
            );
        }
        StackAction::Rebase => {
            println!("{}", "Stack rebase not yet implemented".yellow());
        }
        StackAction::Land => {
            println!("{}", "Stack landing not yet implemented".yellow());
        }
    }

    Ok(())
}
