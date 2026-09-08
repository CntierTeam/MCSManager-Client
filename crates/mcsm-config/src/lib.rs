//! Local config: `~/.config/mcsm/config.toml`

use directories::ProjectDirs;
use mcsm_protocol::{McsmError, McsmResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub panel_url: String,
    #[serde(default)]
    pub api_key: String,
    #[serde(default)]
    pub default_daemon_id: Option<String>,
    #[serde(default)]
    pub session_token: Option<String>,
    #[serde(default)]
    pub username: Option<String>,
}

impl Config {
    pub fn is_ready(&self) -> bool {
        !self.panel_url.is_empty() && (!self.api_key.is_empty() || self.session_token.is_some())
    }

    pub fn require_url(&self) -> McsmResult<&str> {
        if self.panel_url.is_empty() {
            Err(McsmError::Config(
                "panel_url not set; run `mcsm config set-url <url>`".into(),
            ))
        } else {
            Ok(&self.panel_url)
        }
    }
}

pub fn config_dir() -> McsmResult<PathBuf> {
    let dirs = ProjectDirs::from("com", "MCSManager", "mcsm").ok_or_else(|| {
        McsmError::Config("cannot resolve config directory".into())
    })?;
    Ok(dirs.config_dir().to_path_buf())
}

pub fn config_path() -> McsmResult<PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}

pub fn load() -> McsmResult<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(&path)?;
    toml::from_str(&text).map_err(|e| McsmError::Config(e.to_string()))
}

pub fn load_from(path: &std::path::Path) -> McsmResult<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let text = fs::read_to_string(path)?;
    toml::from_str(&text).map_err(|e| McsmError::Config(e.to_string()))
}

pub fn save(cfg: &Config) -> McsmResult<()> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;
    let path = dir.join("config.toml");
    let text = toml::to_string_pretty(cfg).map_err(|e| McsmError::Config(e.to_string()))?;
    fs::write(path, text)?;
    Ok(())
}

pub fn save_to(path: &std::path::Path, cfg: &Config) -> McsmResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let text = toml::to_string_pretty(cfg).map_err(|e| McsmError::Config(e.to_string()))?;
    fs::write(path, text)?;
    Ok(())
}
