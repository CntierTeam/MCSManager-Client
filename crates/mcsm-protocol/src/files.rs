use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListQuery {
    pub target: String,
    pub page: u32,
    pub page_size: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileList {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStatus {
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEditBody {
    pub target: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePathBody {
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCopyBody {
    pub targets: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMoveBody {
    pub targets: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileDeleteBody {
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChmodBody {
    pub target: String,
    pub chmod: u32,
    #[serde(default)]
    pub deep: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCompressBody {
    pub type_code: i32,
    pub source: String,
    pub targets: Vec<String>,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFromUrlBody {
    pub url: String,
    pub filename: String,
    pub path: String,
}
