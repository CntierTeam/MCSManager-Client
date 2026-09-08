//! Daemon client: stream sessions (Socket.IO over websocket) and passport file transfer.
//!
//! Socket.IO engine is implemented with a lightweight Engine.IO v4 + Socket.IO v4
//! client sufficient for MCSManager stream events. Full admin remote-request
//! multiplexing is available via [`DaemonClient::emit_event`].

mod http_transfer;
mod socketio;
mod stream_session;
mod trait_api;

pub use http_transfer::{download_file, upload_file};
pub use stream_session::{DefaultDaemonClient, StreamEvent, StreamSession};
pub use trait_api::{DaemonClient, DaemonConnection};
