use mcsm_config::Config;
use mcsm_panel::{HttpPanelClient, PanelClient};
use mcsm_protocol::error::{McsmError, McsmResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppContext {
    pub panel: Arc<dyn PanelClient>,
    pub config: Config,
}

pub struct AppContextBuilder {
    config: Config,
}

impl AppContextBuilder {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn build(self) -> McsmResult<AppContext> {
        let url = self.config.require_url()?;
        let mut client = HttpPanelClient::new(url)?;
        if !self.config.api_key.is_empty() {
            client = client.with_api_key(&self.config.api_key);
        }
        if let Some(token) = &self.config.session_token {
            client = client.with_session_token(token);
        }
        Ok(AppContext {
            panel: Arc::new(client),
            config: self.config,
        })
    }

    pub fn build_unauthenticated(self) -> McsmResult<AppContext> {
        let url = if self.config.panel_url.is_empty() {
            return Err(McsmError::Config(
                "panel_url not set; run `mcsm config set-url <url>`".into(),
            ));
        } else {
            self.config.panel_url.as_str()
        };
        let client = HttpPanelClient::new(url)?;
        Ok(AppContext {
            panel: Arc::new(client),
            config: self.config,
        })
    }
}

impl AppContext {
    pub fn from_config(config: Config) -> McsmResult<Self> {
        AppContextBuilder::new(config).build()
    }

    pub fn daemon_or<'a>(&'a self, override_id: Option<&'a str>) -> McsmResult<&'a str> {
        if let Some(id) = override_id {
            return Ok(id);
        }
        self.config.default_daemon_id.as_deref().ok_or_else(|| {
            McsmError::Config("daemon id required (pass --daemon or set default)".into())
        })
    }
}
