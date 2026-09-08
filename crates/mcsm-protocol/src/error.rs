use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type McsmResult<T> = Result<T, McsmError>;

#[derive(Debug, Error)]
pub enum McsmError {
    #[error("API error {status}: {message}")]
    Api { status: i32, message: String },

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("not configured: {0}")]
    Config(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("serialization error: {0}")]
    Serde(String),

    #[error("daemon error: {0}")]
    Daemon(String),

    #[error("stream error: {0}")]
    Stream(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("{0}")]
    Message(String),
}

impl From<serde_json::Error> for McsmError {
    fn from(e: serde_json::Error) -> Self {
        McsmError::Serde(e.to_string())
    }
}

impl From<std::io::Error> for McsmError {
    fn from(e: std::io::Error) -> Self {
        McsmError::Io(e.to_string())
    }
}

/// Serializable error for JSON CLI output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorBody {
    pub status: i32,
    pub message: String,
}
