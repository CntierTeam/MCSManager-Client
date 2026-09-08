use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSearchQuery {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDownloadBody {
    #[serde(flatten)]
    pub raw: Value,
}
