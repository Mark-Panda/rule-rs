mod context;
mod descriptor;
mod error;
mod message;

pub use context::*;
pub use descriptor::*;
pub use error::*;
pub use message::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// 规则链定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleChain {
    pub id: Uuid,
    pub name: String,
    pub root: bool,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub metadata: Metadata,
}

// 节点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub type_name: String,
    pub config: serde_json::Value,
    pub layout: Position,
}

// 连接定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub type_name: String,
}

// 位置信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

// 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub version: u64,
    pub created_at: i64,
    pub updated_at: i64,
} 