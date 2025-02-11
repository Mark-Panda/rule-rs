mod context;
mod descriptor;
mod error;
mod message;
mod node;

pub use context::*;
pub use descriptor::*;
pub use error::*;
pub use message::*;
pub use node::*;

use serde::{Deserialize, Serialize};
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

// 连接定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub type_name: String,
}

// 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub version: u64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum NodeType {
    #[serde(rename = "head")]
    Head, // 头节点
    #[serde(rename = "middle")]
    Middle, // 中节点
    #[serde(rename = "tail")]
    Tail, // 尾节点
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommonConfig {
    #[serde(default = "default_node_type")]
    pub node_type: NodeType,
}

fn default_node_type() -> NodeType {
    NodeType::Middle
}
