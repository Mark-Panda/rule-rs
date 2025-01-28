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
