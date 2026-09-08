use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overview {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelSetting {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuditQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditPage {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutData {
    #[serde(flatten)]
    pub raw: Value,
}
