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
    async fn handle<'a>(&self, _ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        let chain = self.engine.get_chain(self.config.chain_id).await
            .ok_or_else(|| RuleError::ConfigError("Subchain not found".to_string()))?;

        let mut ctx = ExecutionContext::new(msg);
        let result = self.engine.execute_chain(&chain, &mut ctx).await?;

        Ok(result)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "subchain".to_string(),
            name: "子规则链".to_string(),
            description: "执行子规则链".to_string(),
        }
    }
}
