use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait NodeHandler: Send + Sync {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError>;
    fn get_descriptor(&self) -> NodeDescriptor;
}

pub type NodeFactory =
    Arc<dyn Fn(serde_json::Value) -> Result<Arc<dyn NodeHandler>, Box<dyn Error>> + Send + Sync>;

pub struct NodeRegistry {
    handlers: RwLock<HashMap<String, NodeFactory>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, type_name: &str, factory: NodeFactory) {
        self.handlers
            .write()
            .await
            .insert(type_name.to_string(), factory);
    }

    pub async fn create_handler(
        &self,
        type_name: &str,
        config: serde_json::Value,
    ) -> Option<Arc<dyn NodeHandler>> {
        if let Some(factory) = self.handlers.read().await.get(type_name) {
            factory(config).ok()
        } else {
            None
        }
    }
}
