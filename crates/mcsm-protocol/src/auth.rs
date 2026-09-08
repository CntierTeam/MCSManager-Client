use crate::Permission;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Login returns a session token string (when using cookie session).
pub type LoginOut = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    #[serde(default)]
    pub isInstall: bool,
    #[serde(default)]
    pub versionChange: bool,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPageInfo {
    #[serde(default)]
    pub loginInfo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub userName: String,
    #[serde(default)]
    pub permission: Permission,
    #[serde(default)]
    pub instances: Vec<Value>,
    #[serde(default)]
    pub apiKey: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePasswordRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyToggle {
    pub enable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
    pub permission: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditUserRequest {
    pub uuid: String,
    #[serde(flatten)]
    pub fields: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userName: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPage {
    #[serde(default)]
    pub data: Vec<UserInfo>,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub pageSize: u32,
    #[serde(default)]
    pub maxPage: u32,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bind2faConfirm {
    pub code: String,
}
