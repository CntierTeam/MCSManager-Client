//! MCSManager TUI (ratatui).

mod app;
mod ui;

use anyhow::Result;
use mcsm_config::Config;
use mcsm_core::AppContext;

pub async fn run() -> Result<()> {
    let cfg = Config::load_or_default();
    run_with_config(cfg).await
}

pub async fn run_with_config(cfg: Config) -> Result<()> {
    let ctx = match AppContext::from_config(cfg.clone()) {
        Ok(c) => Some(c),
        Err(_) => None,
    };
    app::run_app(ctx, cfg).await
}

trait ConfigExt {
    fn load_or_default() -> Config;
}

impl ConfigExt for Config {
    fn load_or_default() -> Config {
        mcsm_config::load().unwrap_or_default()
    }
}
