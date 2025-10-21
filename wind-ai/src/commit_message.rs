use anyhow::Result;
use wind_core::Repository;

pub async fn generate(_repo: &Repository) -> Result<String> {
    Ok("feat: implement feature".to_string())
}
