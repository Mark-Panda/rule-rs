use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 节点处理器特征,定义了节点的核心处理逻辑
#[async_trait]
pub trait NodeHandler: Send + Sync + std::fmt::Debug {
    /// 处理消息
    ///
    /// # Arguments
    /// * `ctx` - 节点执行上下文
    /// * `msg` - 输入消息
    ///
    /// # Returns
    /// * `Result<Message, RuleError>` - 处理结果或错误
    async fn handle<'a>(&'a self, ctx: NodeContext<'a>, msg: Message)
        -> Result<Message, RuleError>;

    /// 获取节点描述符,包含节点的元数据信息
    fn get_descriptor(&self) -> NodeDescriptor;
}

/// 节点工厂函数的包装器,用于创建节点实例
pub struct NodeFactoryWrapper {
    /// 节点工厂函数
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
    /// 创建新的工厂包装器实例
    pub fn new(factory: NodeFactory) -> Self {
        Self { factory }
    }

    /// 使用工厂函数创建节点实例
    pub fn create(
        &self,
        config: serde_json::Value,
    ) -> Result<Arc<dyn NodeHandler>, Box<dyn Error>> {
        (self.factory)(config)
    }
}

/// 节点工厂函数类型,用于根据配置创建节点实例
pub type NodeFactory =
    Arc<dyn Fn(serde_json::Value) -> Result<Arc<dyn NodeHandler>, Box<dyn Error>> + Send + Sync>;

/// 节点注册表,管理所有已注册的节点类型
pub struct NodeRegistry {
    /// 存储节点工厂函数,key为节点类型名称
    factories: RwLock<HashMap<String, NodeFactory>>,
    /// 存储节点描述符,key为节点类型名称
    descriptors: RwLock<HashMap<String, NodeDescriptor>>,
}

impl NodeRegistry {
    /// 创建新的节点注册表实例
    pub fn new() -> Self {
        Self {
            factories: RwLock::new(HashMap::new()),
            descriptors: RwLock::new(HashMap::new()),
        }
    }

    /// 注册新的节点类型
    ///
    /// # Arguments
    /// * `type_name` - 节点类型名称
    /// * `factory` - 节点工厂函数
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

    /// 获取所有已注册节点的描述符
    pub async fn get_descriptors(&self) -> Vec<NodeDescriptor> {
        let descriptors = self.descriptors.read().await;
        descriptors.values().cloned().collect()
    }

    /// 获取指定节点类型的描述符
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

    /// 根据节点类型和配置创建节点处理器实例
    ///
    /// # Arguments
    /// * `type_name` - 节点类型名称
    /// * `config` - 节点配置
    ///
    /// # Returns
    /// * `Option<Arc<dyn NodeHandler>>` - 节点处理器实例或None
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

    /// 获取指定节点类型的工厂函数
    pub async fn get_factory(&self, type_name: &str) -> Option<NodeFactory> {
        let factories = self.factories.read().await;
        factories.get(type_name).cloned()
    }

    /// 获取所有已注册的节点类型名称
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
