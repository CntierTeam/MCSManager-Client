//! Shared protocol types for MCSManager Panel HTTP and Daemon Socket.IO.
//!
//! This crate is the **only** data contract between `mcsm-panel`, `mcsm-daemon`,
//! `mcsm-core`, `mcsm-cli`, and `mcsm-tui`.

#![allow(non_snake_case)]

pub mod auth;
pub mod daemon;
pub mod environment;
pub mod error;
pub mod exchange;
pub mod files;
pub mod ids;
pub mod instance;
pub mod java;
pub mod mod_mgr;
pub mod overview;
pub mod schedule;
pub mod service;
pub mod stream;

pub use error::{McsmError, McsmResult};
pub use ids::{DaemonId, InstanceUuid, UserUuid};

use serde::{Deserialize, Serialize};

/// Standard Panel HTTP response envelope: `{ status, data, time }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEnvelope<T> {
    pub status: i32,
    pub data: T,
    pub time: u64,
}

impl<T> ApiEnvelope<T> {
    pub fn is_ok(&self) -> bool {
        self.status == 200
    }

    pub fn into_result(self) -> McsmResult<T>
    where
        T: Serialize,
    {
        if self.is_ok() {
            Ok(self.data)
        } else {
            let msg = match serde_json::to_value(&self.data) {
                Ok(v) => match v {
                    serde_json::Value::String(s) => s,
                    other => other.to_string(),
                },
                Err(_) => format!("API status {}", self.status),
            };
            Err(McsmError::Api {
                status: self.status,
                message: msg,
            })
        }
    }
}

impl ApiEnvelope<serde_json::Value> {
    pub fn into_result_value(self) -> McsmResult<serde_json::Value> {
        if self.is_ok() {
            Ok(self.data)
        } else {
            let msg = match &self.data {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            Err(McsmError::Api {
                status: self.status,
                message: msg,
            })
        }
    }
}

/// Role levels from MCSManager panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct Permission(pub i32);

impl Permission {
    pub const BAN: i32 = -1;
    pub const GUEST: i32 = 0;
    pub const USER: i32 = 1;
    pub const ADMIN: i32 = 10;

    pub fn is_admin(self) -> bool {
        self.0 >= Self::ADMIN
    }

    pub fn is_user(self) -> bool {
        self.0 >= Self::USER
    }
}
