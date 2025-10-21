use crate::StackAction;
use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: StackAction) -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    match action {
        StackAction::List => {
            let stacks = wind_core::stack::list_stacks(&repo)?;
            if stacks.is_empty() {
                println!("{}", "No stacks found".dimmed());
            } else {
                for stack in stacks {
                    println!(
                        "{} {} ({} branches)",
                        "→".blue(),
                        stack.name.bold(),
                        stack.branches.len()
                    );
                    for branch in &stack.branches {
                        println!("  - {}", branch);
                    }
                }
            }
        }
        StackAction::Create { name } => {
            wind_core::stack::create_stack(&repo, &name)?;
            println!("{} Created stack {}", "✓".green(), name.bold());
        }
        StackAction::Rebase => {
            wind_core::stack::rebase_stack(&repo)?;
            println!("{} Rebased entire stack", "✓".green());
        }
        StackAction::Land => {
            wind_core::stack::land_stack(&repo)?;
            println!("{} Landed stack to main", "✓".green());
        }
    }

    Ok(())
}
