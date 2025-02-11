use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub msg_type: String,
    pub metadata: HashMap<String, String>,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

impl Message {
    pub fn new(msg_type: &str, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            msg_type: msg_type.to_string(),
            metadata: HashMap::new(),
            data,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}
