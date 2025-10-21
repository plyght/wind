use anyhow::Result;
use colored::Colorize;

pub async fn execute(name: Option<String>, delete: bool, list: bool) -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    if list || name.is_none() {
        let branches = repo.list_branches()?;
        let current = repo.current_branch()?;

        for branch in branches {
            if branch == current {
                println!("{} {}", "*".green(), branch.green().bold());
            } else {
                println!("  {}", branch);
            }
        }
    } else if let Some(branch_name) = name {
        if delete {
            repo.delete_branch(&branch_name)?;
            println!("{} Deleted branch {}", "✓".green(), branch_name.bold());
        } else {
            repo.create_branch(&branch_name)?;
            println!("{} Created branch {}", "✓".green(), branch_name.bold());
        }
    }

    Ok(())
}
