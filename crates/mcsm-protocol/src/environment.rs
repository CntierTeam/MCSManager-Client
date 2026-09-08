use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewImageBody {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressMap {
    #[serde(flatten)]
    pub raw: Value,
}
