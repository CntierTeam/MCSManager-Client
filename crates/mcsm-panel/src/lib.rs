//! MCSManager Panel HTTP client.
//!
//! All Panel `/api` access goes through [`PanelClient`]. CLI/TUI must not call HTTP directly.

mod client;
mod trait_api;

pub use client::HttpPanelClient;
pub use trait_api::PanelClient;
