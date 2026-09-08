use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTask {
    #[serde(default)]
    pub name: String,
    #[serde(flatten)]
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleAction {
    #[serde(rename = "type")]
    pub action_type: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchedule {
    pub name: String,
    pub count: i32,
    pub time: String,
    pub actions: Vec<ScheduleAction>,
    #[serde(rename = "type")]
    pub type_code: i32,
}
