//! Shared application context and domain services.

mod context;
mod services;

pub use context::{AppContext, AppContextBuilder};
pub use services::*;
