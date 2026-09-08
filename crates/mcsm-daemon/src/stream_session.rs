use crate::socketio::SocketIoClient;
use mcsm_protocol::error::{McsmError, McsmResult};
use mcsm_protocol::stream::{
    StreamPassport, INSTANCE_OPENED, INSTANCE_STDOUT, INSTANCE_STOPPED, STREAM_AUTH, STREAM_DETAIL,
    STREAM_INPUT, STREAM_RESIZE, STREAM_WRITE,
};
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Stdout(String),
    Detail(Value),
    Opened,
    Stopped,
    Auth(Value),
    Other(String, Value),
}

pub struct StreamSession {
    client: SocketIoClient,
    events: mpsc::UnboundedReceiver<StreamEvent>,
    _pump: tokio::task::JoinHandle<()>,
}

impl StreamSession {
    pub async fn connect(passport: StreamPassport) -> McsmResult<Self> {
        let client = SocketIoClient::connect(&passport.addr, &passport.prefix).await?;
        // stream/auth with password
        client
            .emit(STREAM_AUTH, Value::String(passport.password.clone()))
            .await?;

        let (tx, rx) = mpsc::unbounded_channel();
        let client_c = client.clone();
        let pump = tokio::spawn(async move {
            while let Some((event, data)) = client_c.next_event().await {
                let ev = match event.as_str() {
                    e if e == INSTANCE_STDOUT => {
                        let text = match data {
                            Value::String(s) => s,
                            other => other.to_string(),
                        };
                        StreamEvent::Stdout(text)
                    }
                    e if e == STREAM_DETAIL => StreamEvent::Detail(data),
                    e if e == INSTANCE_OPENED => StreamEvent::Opened,
                    e if e == INSTANCE_STOPPED => StreamEvent::Stopped,
                    e if e == STREAM_AUTH => StreamEvent::Auth(data),
                    _ => StreamEvent::Other(event, data),
                };
                if tx.send(ev).is_err() {
                    break;
                }
            }
        });

        // Request initial detail
        let _ = client.emit(STREAM_DETAIL, Value::Null).await;

        Ok(Self {
            client,
            events: rx,
            _pump: pump,
        })
    }

    pub async fn send_input(&self, line: &str) -> McsmResult<()> {
        self.client
            .emit(STREAM_INPUT, Value::String(line.to_string()))
            .await
    }

    pub async fn write_raw(&self, data: &str) -> McsmResult<()> {
        self.client
            .emit(STREAM_WRITE, Value::String(data.to_string()))
            .await
    }

    pub async fn resize(&self, cols: u16, rows: u16) -> McsmResult<()> {
        self.client
            .emit(STREAM_RESIZE, serde_json::json!({ "w": cols, "h": rows }))
            .await
    }

    pub async fn next(&mut self) -> Option<StreamEvent> {
        self.events.recv().await
    }

    pub async fn close(self) {
        self.client.close().await;
    }
}

/// Default DaemonClient facade used by core.
pub struct DefaultDaemonClient;

#[async_trait::async_trait]
impl crate::DaemonClient for DefaultDaemonClient {
    async fn connect_with_key(
        addr: &str,
        key: &str,
        prefix: &str,
    ) -> McsmResult<crate::DaemonConnection> {
        let inner = SocketIoClient::connect(addr, prefix).await?;
        inner.auth_key(key).await?;
        Ok(crate::DaemonConnection { inner })
    }

    async fn connect_stream(passport: StreamPassport) -> McsmResult<StreamSession> {
        StreamSession::connect(passport).await
    }

    async fn upload(passport_url: &str, local_path: &std::path::Path) -> McsmResult<()> {
        crate::http_transfer::upload_file(passport_url, local_path).await
    }

    async fn download(passport_url: &str, dest: &std::path::Path) -> McsmResult<()> {
        crate::http_transfer::download_file(passport_url, dest).await
    }
}

// silence unused import warning helper
#[allow(dead_code)]
fn _err(msg: &str) -> McsmError {
    McsmError::Stream(msg.into())
}
