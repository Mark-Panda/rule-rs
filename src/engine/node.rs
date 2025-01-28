use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait NodeHandler: Send + Sync {
    async fn handle(&self, ctx: NodeContext, msg: Message) -> Result<Message, RuleError>;
    fn get_descriptor(&self) -> NodeDescriptor;
}

pub struct NodeRegistry {
    handlers: RwLock<HashMap<String, Arc<dyn NodeHandler>>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, type_name: &str, handler: Arc<dyn NodeHandler>) {
        self.handlers
            .write()
            .await
            .insert(type_name.to_string(), handler);
    }

    pub async fn get_handler(&self, type_name: &str) -> Option<Arc<dyn NodeHandler>> {
        self.handlers.read().await.get(type_name).cloned()
    }
}
