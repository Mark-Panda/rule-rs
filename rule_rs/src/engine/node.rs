use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait NodeHandler: Send + Sync + std::fmt::Debug {
    async fn handle<'a>(&'a self, ctx: NodeContext<'a>, msg: Message)
        -> Result<Message, RuleError>;

    fn get_descriptor(&self) -> NodeDescriptor;
}

// 包装工厂函数的结构体
pub struct NodeFactoryWrapper {
    factory: NodeFactory,
}

impl fmt::Debug for NodeFactoryWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeFactoryWrapper")
            .field("factory", &"<factory function>")
            .finish()
    }
}

impl NodeFactoryWrapper {
    pub fn new(factory: NodeFactory) -> Self {
        Self { factory }
    }

    pub fn create(
        &self,
        config: serde_json::Value,
    ) -> Result<Arc<dyn NodeHandler>, Box<dyn Error>> {
        (self.factory)(config)
    }
}

pub type NodeFactory =
    Arc<dyn Fn(serde_json::Value) -> Result<Arc<dyn NodeHandler>, Box<dyn Error>> + Send + Sync>;

pub struct NodeRegistry {
    factories: RwLock<HashMap<String, NodeFactory>>,
    descriptors: RwLock<HashMap<String, NodeDescriptor>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
            descriptors: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, type_name: &str, factory: NodeFactory) {
        let mut factories = self.factories.write().await;
        let mut descriptors = self.descriptors.write().await;

        // 创建一个临时配置来获取节点描述符
        let empty_config = serde_json::json!({});
        match factory(empty_config.clone()) {
            Ok(node) => {
                let descriptor = node.get_descriptor();
                descriptors.insert(type_name.to_string(), descriptor);
                factories.insert(type_name.to_string(), factory);
            }
            Err(e) => {
                tracing::error!("Failed to register node type {}: {}", type_name, e);
            }
        }
    }

    pub async fn get_descriptors(&self) -> Vec<NodeDescriptor> {
        let descriptors = self.descriptors.read().await;
        descriptors.values().cloned().collect()
    }

    pub async fn get_descriptor(&self, type_name: &str) -> Option<NodeDescriptor> {
        let factories = self.factories.read().await;
        if let Some(factory) = factories.get(type_name) {
            let empty_config = serde_json::Value::Object(serde_json::Map::new());
            if let Ok(node) = factory(empty_config) {
                let descriptor = node.get_descriptor();
                Some(NodeDescriptor {
                    type_name: type_name.to_string(),
                    name: descriptor.name,
                    description: descriptor.description,
                    node_type: descriptor.node_type,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub async fn create_handler(
        &self,
        type_name: &str,
        config: serde_json::Value,
    ) -> Option<Arc<dyn NodeHandler>> {
        let factories = self.factories.read().await;
        if let Some(factory) = factories.get(type_name) {
            match factory(config) {
                Ok(handler) => Some(handler),
                Err(e) => {
                    tracing::error!("Failed to create handler for {}: {}", type_name, e);
                    None
                }
            }
        } else {
            tracing::error!("No factory found for node type: {}", type_name);
            None
        }
    }

    pub async fn get_factory(&self, type_name: &str) -> Option<NodeFactory> {
        let factories = self.factories.read().await;
        factories.get(type_name).cloned()
    }

    pub async fn get_registered_types(&self) -> Vec<String> {
        let factories = self.factories.read().await;
        factories.keys().cloned().collect()
    }
}

impl fmt::Debug for NodeRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeRegistry")
            .field("factories", &"<node factories>")
            .field("descriptors", &"<node descriptors>")
            .finish()
    }
}
