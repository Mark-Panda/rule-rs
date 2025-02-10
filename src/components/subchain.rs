use crate::engine::NodeHandler;
use crate::types::{
    CommonConfig, ExecutionContext, Message, NodeContext, NodeDescriptor, NodeType, RuleError,
};
use async_trait::async_trait;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SubchainConfig {
    pub chain_id: Uuid,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for SubchainConfig {
    fn default() -> Self {
        Self {
            chain_id: Uuid::nil(),
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

#[derive(Debug)]
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
    async fn handle<'a>(&'a self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        let subchain = ctx
            .engine
            .get_chain(self.config.chain_id)
            .await
            .ok_or_else(|| {
                RuleError::ConfigError(format!("子规则链未找到: {}", self.config.chain_id))
            })?;

        // 创建子规则链上下文
        let mut sub_ctx = ExecutionContext::new(msg.clone());
        let result = ctx.engine.execute_chain(&subchain, &mut sub_ctx).await?;

        // 发送到下一个节点
        ctx.send_next(result.clone()).await?;
        
        Ok(result)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "subchain".to_string(),
            name: "子规则链节点".to_string(),
            description: "执行另一个规则链".to_string(),
        }
    }
}
