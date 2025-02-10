use crate::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: Uuid,
    pub type_name: String,
    pub config: Value,
    pub layout: Position,
    pub chain_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[async_trait]
pub trait NodeHandler: Send + Sync + std::fmt::Debug {
    async fn handle(&self, ctx: NodeContext<'_>, msg: Message) -> Result<Message, RuleError>;
    fn get_descriptor(&self) -> NodeDescriptor;
}

impl Node {
    pub fn get_node_type(&self) -> Result<NodeType, RuleError> {
        match self.type_name.as_str() {
            "log" => Ok(NodeType::Tail),
            "delay" => Ok(NodeType::Head),
            "schedule" => Ok(NodeType::Head),
            "start" => Ok(NodeType::Head),
            _ => Ok(NodeType::Middle),
        }
    }
}
