use crate::DaemonId;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSummary {
    #[serde(default)]
    pub uuid: DaemonId,
    #[serde(default)]
    pub ip: String,
    #[serde(default)]
    pub port: u16,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub remarks: String,
    #[serde(default)]
    pub available: bool,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNode {
    pub ip: String,
    pub port: u16,
    pub apiKey: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub remarks: String,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditNode {
    #[serde(flatten)]
    pub fields: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceListQuery {
    pub daemonId: String,
    pub page: u32,
    pub page_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstancePage {
    #[serde(default)]
    pub data: Vec<Value>,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub pageSize: u32,
    #[serde(default)]
    pub maxPage: u32,
    #[serde(default)]
    pub allTags: Vec<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalInstanceQuery {
    pub page: u32,
    pub page_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}
