use crate::AiAction;
use anyhow::Result;
use colored::Colorize;

pub async fn execute(action: AiAction) -> Result<()> {
    match action {
        AiAction::Enable => {
            wind_ai::config::enable()?;
            println!("{} AI features enabled", "✓".green());
        }
        AiAction::Disable => {
            wind_ai::config::disable()?;
            println!("{} AI features disabled", "✓".green());
        }
        AiAction::Configure { api_key, provider } => {
            if let Some(key) = api_key {
                wind_ai::config::set_api_key(&key)?;
                println!("{} API key configured", "✓".green());
            }
            if let Some(prov) = provider {
                wind_ai::config::set_provider(&prov)?;
                println!("{} Provider set to {}", "✓".green(), prov.bold());
            }
        }
    }

    Ok(())
}
