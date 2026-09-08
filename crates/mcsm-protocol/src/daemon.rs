use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Daemon request packet: `{ uuid, data }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IRequestPacket {
    pub uuid: String,
    pub data: Value,
}

/// Daemon response packet: `{ uuid, status, event, data }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPacket {
    pub uuid: String,
    pub status: i32,
    #[serde(default)]
    pub event: String,
    pub data: Value,
}

impl IPacket {
    pub fn is_ok(&self) -> bool {
        self.status == 200
    }
}
