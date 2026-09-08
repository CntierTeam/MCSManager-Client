//! Minimal Engine.IO v4 + Socket.IO v4 client for MCSManager Daemon.

use futures::{SinkExt, StreamExt};
use mcsm_protocol::error::{McsmError, McsmResult};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

type WsStream = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;
type WsSink = futures::stream::SplitSink<WsStream, Message>;
type AckMap = Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>;

#[derive(Clone)]
pub struct SocketIoClient {
    writer: Arc<Mutex<Option<WsSink>>>,
    ack_map: AckMap,
    ack_counter: Arc<AtomicU64>,
    event_rx: Arc<Mutex<mpsc::UnboundedReceiver<(String, Value)>>>,
    _reader: Arc<Mutex<Option<JoinHandle<()>>>>,
    closed: Arc<RwLock<bool>>,
}

impl SocketIoClient {
    pub async fn connect(addr: &str, prefix: &str) -> McsmResult<Self> {
        let ws_url = build_ws_url(addr, prefix)?;
        let (ws, _) = connect_async(&ws_url)
            .await
            .map_err(|e| McsmError::Daemon(format!("ws connect {ws_url}: {e}")))?;
        let (write, mut read) = ws.split();
        let ack_map: AckMap = Arc::new(Mutex::new(HashMap::new()));
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let writer = Arc::new(Mutex::new(Some(write)));
        let closed = Arc::new(RwLock::new(false));

        let ack_map_r = ack_map.clone();
        let writer_r = writer.clone();
        let closed_r = closed.clone();
        let reader = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if *closed_r.read().await {
                    break;
                }
                let Ok(msg) = msg else { break };
                let text = match msg {
                    Message::Text(t) => t.to_string(),
                    Message::Ping(p) => {
                        if let Some(w) = writer_r.lock().await.as_mut() {
                            let _ = w.send(Message::Pong(p)).await;
                        }
                        continue;
                    }
                    Message::Close(_) => break,
                    _ => continue,
                };

                if text == "2" {
                    if let Some(w) = writer_r.lock().await.as_mut() {
                        let _ = w.send(Message::Text("3".into())).await;
                    }
                    continue;
                }
                if text.starts_with('0') {
                    continue;
                }
                if !text.starts_with('4') {
                    continue;
                }
                let sio = &text[1..];
                if let Some(rest) = sio.strip_prefix('2') {
                    handle_event_packet(rest, &ack_map_r, &event_tx).await;
                } else if let Some(rest) = sio.strip_prefix('3') {
                    handle_ack_packet(rest, &ack_map_r).await;
                }
            }
        });

        let client = Self {
            writer,
            ack_map,
            ack_counter: Arc::new(AtomicU64::new(1)),
            event_rx: Arc::new(Mutex::new(event_rx)),
            _reader: Arc::new(Mutex::new(Some(reader))),
            closed,
        };

        tokio::time::sleep(Duration::from_millis(50)).await;
        client.send_raw("40").await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(client)
    }

    pub async fn send_raw(&self, packet: &str) -> McsmResult<()> {
        let mut guard = self.writer.lock().await;
        let Some(w) = guard.as_mut() else {
            return Err(McsmError::Daemon("connection closed".into()));
        };
        w.send(Message::Text(packet.to_string().into()))
            .await
            .map_err(|e| McsmError::Daemon(e.to_string()))
    }

    pub async fn emit(&self, event: &str, data: Value) -> McsmResult<()> {
        let payload = serde_json::json!([event, data]);
        self.send_raw(&format!("42{payload}")).await
    }

    pub async fn request(&self, event: &str, data: Value) -> McsmResult<Value> {
        let id = self.ack_counter.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();
        self.ack_map.lock().await.insert(id, tx);
        let payload = serde_json::json!([event, data]);
        let packet = format!("42{id}{payload}");
        if let Err(e) = self.send_raw(&packet).await {
            self.ack_map.lock().await.remove(&id);
            return Err(e);
        }
        match tokio::time::timeout(Duration::from_secs(30), rx).await {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(_)) => Err(McsmError::Daemon("ack channel closed".into())),
            Err(_) => {
                self.ack_map.lock().await.remove(&id);
                Err(McsmError::Daemon(format!(
                    "timeout waiting ack for event {event}"
                )))
            }
        }
    }

    pub async fn auth_key(&self, key: &str) -> McsmResult<()> {
        self.emit(
            mcsm_protocol::stream::DAEMON_AUTH,
            Value::String(key.to_string()),
        )
        .await?;
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    pub async fn next_event(&self) -> Option<(String, Value)> {
        self.event_rx.lock().await.recv().await
    }

    pub async fn close(&self) {
        *self.closed.write().await = true;
        if let Some(mut w) = self.writer.lock().await.take() {
            let _ = w.close().await;
        }
    }
}

async fn handle_event_packet(
    rest: &str,
    ack_map: &AckMap,
    event_tx: &mpsc::UnboundedSender<(String, Value)>,
) {
    let (ack_id, json_part) = split_ack_prefix(rest);
    let Ok(arr) = serde_json::from_str::<Value>(json_part) else {
        return;
    };
    let Some(list) = arr.as_array() else {
        return;
    };
    if list.is_empty() {
        return;
    }
    let event = list[0].as_str().unwrap_or("").to_string();
    let data = if list.len() > 1 {
        list[1].clone()
    } else {
        Value::Null
    };
    if let Some(id) = ack_id {
        if let Some(tx) = ack_map.lock().await.remove(&id) {
            let _ = tx.send(data.clone());
        }
    }
    let _ = event_tx.send((event, data));
}

async fn handle_ack_packet(rest: &str, ack_map: &AckMap) {
    let (ack_id, json_part) = split_ack_prefix(rest);
    let Some(id) = ack_id else {
        return;
    };
    let data = serde_json::from_str::<Value>(json_part).unwrap_or(Value::Null);
    let payload = match data {
        Value::Array(mut a) if !a.is_empty() => a.remove(0),
        other => other,
    };
    if let Some(tx) = ack_map.lock().await.remove(&id) {
        let _ = tx.send(payload);
    }
}

fn split_ack_prefix(s: &str) -> (Option<u64>, &str) {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        (None, s)
    } else {
        let id = digits.parse().ok();
        (id, &s[digits.len()..])
    }
}

fn build_ws_url(addr: &str, prefix: &str) -> McsmResult<String> {
    let mut base = addr.trim_end_matches('/').to_string();
    if !base.contains("://") {
        base = format!("http://{base}");
    }
    let base = base
        .replacen("https://", "wss://", 1)
        .replacen("http://", "ws://", 1);
    let prefix = prefix.trim_end_matches('/');
    let path = if prefix.is_empty() {
        "/socket.io".to_string()
    } else if prefix.ends_with("/socket.io") {
        prefix.to_string()
    } else {
        format!("{prefix}/socket.io")
    };
    Ok(format!("{base}{path}/?EIO=4&transport=websocket"))
}
