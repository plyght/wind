use anyhow::Result;
use colored::Colorize;
use wind_core::UnifiedRepository;

pub async fn execute(name: Option<String>, delete: bool, list: bool) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let repo = UnifiedRepository::open(current_dir)?;

    if list || name.is_none() {
        let branches = repo.branches()?;

        for branch in branches {
            println!("  {} (head: {})", branch.name.green(), &branch.head[..8]);
        }
    } else if let Some(_branch_name) = name {
        if delete {
            println!("{}", "Branch deletion not yet implemented".yellow());
        } else {
            println!("{}", "Branch creation not yet implemented".yellow());
        }
    }

    Ok(())
}
