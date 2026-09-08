use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    #[serde(default)]
    pub instanceUuid: String,
    #[serde(default)]
    pub started: i32,
    #[serde(default)]
    pub status: i32,
    #[serde(default)]
    pub config: Value,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstance {
    #[serde(flatten)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstance {
    #[serde(flatten)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteInstances {
    pub uuids: Vec<String>,
    #[serde(default)]
    pub deleteFile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiInstanceOp {
    /// Map daemonId -> instance uuids
    #[serde(flatten)]
    pub targets: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncTaskQuery {
    pub task_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfigFileQuery {
    pub fileName: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
}
