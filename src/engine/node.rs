use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[async_trait]
pub trait NodeHandler: Send + Sync {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError>;
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

#[derive(Debug)]
pub struct NodeRegistry {
    handlers: RwLock<HashMap<String, Arc<NodeFactoryWrapper>>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
        }
    }

    pub async fn register(&self, type_name: &str, factory: NodeFactory) {
        self.handlers.write().await.insert(
            type_name.to_string(),
            Arc::new(NodeFactoryWrapper::new(factory)),
        );
    }

    pub async fn create_handler(
        &self,
        type_name: &str,
        config: serde_json::Value,
    ) -> Option<Arc<dyn NodeHandler>> {
        if let Some(wrapper) = self.handlers.read().await.get(type_name) {
            wrapper.create(config).ok()
        } else {
            None
        }
    }

    /// 获取所有已注册组件的描述信息
    pub async fn get_descriptors(&self) -> Vec<NodeDescriptor> {
        let mut descriptors = Vec::new();

        // 创建一个空配置来初始化每种类型的节点
        let empty_config = serde_json::json!({});

        for (_type_name, wrapper) in self.handlers.read().await.iter() {
            if let Ok(handler) = wrapper.create(empty_config.clone()) {
                descriptors.push(handler.get_descriptor());
            }
        }

        descriptors
    }
}
