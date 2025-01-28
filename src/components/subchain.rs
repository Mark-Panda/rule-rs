use crate::engine::NodeHandler;
use crate::types::ExecutionContext;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SubchainConfig {
    pub chain_id: uuid::Uuid,
}

pub struct SubchainNode {
    config: SubchainConfig,
}

impl SubchainNode {
    pub fn new(config: SubchainConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for SubchainNode {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        // 获取子规则链
        let chains = ctx.engine.chains.read().await;
        let subchain = chains.get(&self.config.chain_id).ok_or_else(|| {
            RuleError::ConfigError(format!("子规则链未找到: {}", self.config.chain_id))
        })?;

        // 创建子规则链上下文
        let mut sub_ctx = ExecutionContext::new(msg);
        ctx.engine.execute_chain(subchain, &mut sub_ctx).await
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "subchain".to_string(),
            name: "子规则链节点".to_string(),
            description: "执行另一个规则链".to_string(),
        }
    }
}
