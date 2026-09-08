use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeRequest {
    pub target: String,
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemBody {
    #[serde(flatten)]
    pub raw: Value,
}
