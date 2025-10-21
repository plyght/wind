use crate::ConfigAction;
use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: ConfigAction) -> Result<()> {
    let repo = wind_core::repository::Repository::open(".")?;

    match action {
        ConfigAction::Get { key } => {
            let value = repo.config_get(&key)?;
            println!("{}", value);
        }
        ConfigAction::Set { key, value } => {
            repo.config_set(&key, &value)?;
            println!("{} Set {} = {}", "✓".green(), key.bold(), value);
        }
        ConfigAction::List => {
            let config = repo.config_list()?;
            for (key, value) in config {
                println!("{} = {}", key.bold(), value);
            }
        }
    }

    Ok(())
}
