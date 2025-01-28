use super::*;
use crate::engine::RuleEngine;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NodeContext<'a> {
    pub node: &'a Node,
    pub metadata: HashMap<String, String>,
    pub engine: Arc<RuleEngine>,
    pub msg: Message,
}

impl<'a> NodeContext<'a> {
    pub fn new(node: &'a Node, ctx: &ExecutionContext, engine: Arc<RuleEngine>) -> Self {
        Self {
            node,
            metadata: ctx.metadata.clone(),
            engine,
            msg: ctx.msg.clone(),
        }
    }

    pub fn create_subchain_context(&self) -> ExecutionContext {
        ExecutionContext {
            msg: self.msg.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub async fn send_next(&self, msg: Message) -> Result<(), RuleError> {
        // 获取当前节点的规则链
        let chain = self
            .engine
            .get_chain(self.node.chain_id)
            .await
            .ok_or_else(|| RuleError::ChainNotFound(self.node.chain_id))?;

        // 获取下一个节点
        let next_node = chain.get_next_node(&self.node.id, &self.create_subchain_context())?;

        // 如果有下一个节点，则执行
        if let Some(node) = next_node {
            let ctx = NodeContext::new(node, &self.create_subchain_context(), self.engine.clone());
            self.engine.execute_node(node, &ctx, msg).await?;
        }

        Ok(())
    }
}

pub struct ExecutionContext {
    pub msg: Message,
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(msg: Message) -> Self {
        Self {
            msg,
            metadata: HashMap::new(),
        }
    }
}
