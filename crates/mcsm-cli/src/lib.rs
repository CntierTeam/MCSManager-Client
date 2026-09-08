//! CLI command definitions and runners.

mod commands;
mod output;

pub use commands::{Cli, Command};
pub use output::print_result;

use anyhow::Result;
use mcsm_config::Config;
use mcsm_core::AppContext;

pub async fn run(cli: Cli) -> Result<()> {
    commands::dispatch(cli).await
}

pub fn load_config(cli: &Cli) -> Result<Config> {
    let mut cfg = if let Some(path) = &cli.config {
        mcsm_config::load_from(path)?
    } else {
        mcsm_config::load()?
    };
    if let Some(url) = &cli.url {
        cfg.panel_url = url.clone();
    }
    if let Some(key) = &cli.apikey {
        cfg.api_key = key.clone();
    }
    Ok(cfg)
}

pub fn ctx_from_cli(cli: &Cli) -> Result<AppContext> {
    let cfg = load_config(cli)?;
    Ok(AppContext::from_config(cfg)?)
}
