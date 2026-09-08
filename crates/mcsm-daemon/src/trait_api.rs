use crate::StreamSession;
use async_trait::async_trait;
use mcsm_protocol::error::McsmResult;
use mcsm_protocol::stream::StreamPassport;
use serde_json::Value;
use std::path::Path;

#[async_trait]
pub trait DaemonClient: Send + Sync {
    /// Connect with Daemon API key (ops/debug).
    async fn connect_with_key(addr: &str, key: &str, prefix: &str) -> McsmResult<DaemonConnection>;

    /// Connect stream session using Panel-issued passport.
    async fn connect_stream(passport: StreamPassport) -> McsmResult<StreamSession>;

    async fn upload(passport_url: &str, local_path: &Path) -> McsmResult<()>;
    async fn download(passport_url: &str, dest: &Path) -> McsmResult<()>;
}

/// Authenticated daemon Socket.IO connection for direct ops.
pub struct DaemonConnection {
    pub(crate) inner: crate::socketio::SocketIoClient,
}

impl DaemonConnection {
    pub async fn emit_event(&self, event: &str, data: Value) -> McsmResult<Value> {
        self.inner.request(event, data).await
    }

    pub async fn close(&self) {
        self.inner.close().await;
    }
}
