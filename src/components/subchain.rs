use crate::engine::{NodeHandler, RuleEngine};
use crate::types::{ExecutionContext, Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct SubchainConfig {
    pub chain_id: uuid::Uuid,
    pub output_type: Option<String>,
}

pub struct SubchainNode {
    pub(crate) config: SubchainConfig,
    pub(crate) engine: Arc<RuleEngine>,
}

impl SubchainNode {
    pub fn new(config: SubchainConfig, engine: Arc<RuleEngine>) -> Self {
        Self { config, engine }
    }
}

#[async_trait]
impl NodeHandler for SubchainNode {
    async fn handle(&self, _ctx: NodeContext, msg: Message) -> Result<Message, RuleError> {
        // 获取子规则链
        let chain = self
            .engine
            .get_chain(self.config.chain_id)
            .await
            .ok_or_else(|| RuleError::ConfigError("子规则链不存在".to_string()))?;

        // 创建执行上下文
        let ctx = ExecutionContext::new(msg);

        // 执行子规则链
        let result = self.engine.execute_chain(&chain, ctx).await?;

        // 设置输出类型
        Ok(Message {
            id: result.id,
            msg_type: self.config.output_type.clone().unwrap_or(result.msg_type),
            metadata: result.metadata,
            data: result.data,
            timestamp: result.timestamp,
        })
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "subchain".to_string(),
            name: "子规则链".to_string(),
            description: "执行子规则链".to_string(),
        }
    }
}
