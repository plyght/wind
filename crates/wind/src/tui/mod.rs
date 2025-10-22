mod app;
mod commands;
mod config;
mod event;
pub mod lazy_list;
mod state;
mod ui;

use self::config::Config as TuiConfig;
use crate::Repository;
use anyhow::Result;

pub async fn run(repo: &Repository) -> Result<()> {
    let config = TuiConfig::default();
    let mut app = app::App::new(config, repo).await?;
    app.run().await
}
