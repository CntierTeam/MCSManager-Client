//! Stream / passport constants and types (Panel → Daemon bypass).

use serde::{Deserialize, Serialize};

pub const STREAM_AUTH: &str = "stream/auth";
pub const STREAM_DETAIL: &str = "stream/detail";
pub const STREAM_INPUT: &str = "stream/input";
pub const STREAM_WRITE: &str = "stream/write";
pub const STREAM_RESIZE: &str = "stream/resize";
pub const INSTANCE_STDOUT: &str = "instance/stdout";
pub const INSTANCE_OPENED: &str = "instance/opened";
pub const INSTANCE_STOPPED: &str = "instance/stopped";
pub const DAEMON_AUTH: &str = "auth";

/// Returned by `POST /api/protected_instance/stream_channel` and file upload/download.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamPassport {
    pub addr: String,
    pub password: String,
    #[serde(default)]
    pub prefix: String,
    #[serde(default)]
    pub key: Option<String>,
}

/// Passport from Panel file upload/download endpoints.
///
/// Panel returns `{ password, addr, prefix?, remoteMappings? }` — not a full URL.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilePassport {
    /// Optional prebuilt full URL (rare). Prefer [`Self::http_base`] + path helpers.
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub addr: Option<String>,
    #[serde(default)]
    pub prefix: Option<String>,
}

impl FilePassport {
    /// Normalize daemon addr into `http(s)://host:port/prefix` (no trailing slash).
    pub fn http_base(&self) -> String {
        if !self.url.is_empty()
            && (self.url.starts_with("http://") || self.url.starts_with("https://"))
        {
            return self.url.trim_end_matches('/').to_string();
        }
        let addr = self
            .addr
            .clone()
            .or_else(|| {
                if !self.url.is_empty() {
                    Some(self.url.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let prefix = self.prefix.clone().unwrap_or_default();
        let mut base = if addr.starts_with("http://") || addr.starts_with("https://") {
            addr
        } else if addr.starts_with("ws://") {
            addr.replacen("ws://", "http://", 1)
        } else if addr.starts_with("wss://") {
            addr.replacen("wss://", "https://", 1)
        } else {
            format!("http://{addr}")
        };
        base = base.trim_end_matches('/').to_string();
        let prefix = prefix.trim_matches('/');
        if !prefix.is_empty() {
            format!("{base}/{prefix}")
        } else {
            base
        }
    }

    pub fn upload_url(&self) -> crate::McsmResult<String> {
        let password = self
            .password
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| crate::McsmError::Message("file passport missing password".into()))?;
        Ok(format!("{}/upload/{}", self.http_base(), password))
    }

    pub fn download_url(&self, file_name: &str) -> crate::McsmResult<String> {
        let password = self
            .password
            .as_deref()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| crate::McsmError::Message("file passport missing password".into()))?;
        let name = file_name.rsplit('/').next().unwrap_or(file_name);
        let name = urlencoding_minimal(name);
        Ok(format!("{}/download/{}/{}", self.http_base(), password, name))
    }
}

fn urlencoding_minimal(s: &str) -> String {
    // Enough for typical instance filenames.
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}
