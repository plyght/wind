mod app;
mod commands;
mod config;
mod event;
pub mod lazy_list;
mod state;
mod ui;

use anyhow::Result;
use wind_core::Repository;

pub async fn run(repo: &Repository) -> Result<()> {
    let config = config::Config::default();
    let mut app = app::App::new(config, repo).await?;
    app.run().await
}
