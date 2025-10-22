use crate::Repository;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stack {
    pub name: String,
    pub branches: Vec<String>,
    pub base: String,
}

pub fn list_stacks(_repo: &Repository) -> Result<Vec<Stack>> {
    Ok(vec![])
}

pub fn create_stack(_repo: &Repository, _name: &str) -> Result<()> {
    Ok(())
}

pub fn rebase_stack(_repo: &Repository) -> Result<()> {
    Ok(())
}

pub fn land_stack(_repo: &Repository) -> Result<()> {
    Ok(())
}
