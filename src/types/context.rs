use super::*;
use crate::engine::DynRuleEngine;
use crate::types::{Connection, Message, Node, RuleError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct NodeContext<'a> {
    pub node: &'a Node,
    pub metadata: HashMap<String, String>,
    pub engine: DynRuleEngine,
    pub msg: Message,
    branch_results: Arc<Mutex<HashMap<String, Message>>>,
}

#[derive(Debug, Clone)]
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

impl<'a> NodeContext<'a> {
    pub fn new(node: &'a Node, ctx: &ExecutionContext, engine: DynRuleEngine) -> Self {
        Self {
            node,
            metadata: ctx.metadata.clone(),
            engine,
            msg: ctx.msg.clone(),
            branch_results: Arc::new(Mutex::new(HashMap::new())),
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

        // 创建执行上下文
        let mut exec_ctx = ExecutionContext::new(msg);
        exec_ctx.metadata = self.metadata.clone();

        // 获取下一个节点
        let next_node = chain.get_next_node(&self.node.id, &exec_ctx)?;

        // 如果有下一个节点，则执行
        if let Some(node) = next_node {
            println!("发送给下一节点: {}", node.id);
            let ctx = NodeContext::new(node, &exec_ctx, self.engine.clone());
            self.engine.execute_node(node, &ctx, exec_ctx.msg).await?;
        }

        Ok(())
    }

    pub fn set_next_branch(&mut self, branch: &str) {
        self.metadata
            .insert("branch_name".to_string(), branch.to_string());
    }

    // 获取指定类型的下一个连接
    pub async fn get_next_connections(
        &self,
        type_name: &str,
    ) -> Result<Vec<Connection>, RuleError> {
        let chain = self
            .engine
            .get_chain(self.node.chain_id)
            .await
            .ok_or_else(|| RuleError::ChainNotFound(self.node.chain_id))?;

        Ok(chain
            .connections
            .iter()
            .filter(|conn| conn.from_id == self.node.id && conn.type_name == type_name)
            .cloned()
            .collect())
    }

    // 发送消息到指定节点
    pub async fn send_to_node(&self, node_id: &Uuid, msg: Message) -> Result<Message, RuleError> {
        let chain = self
            .engine
            .get_chain(self.node.chain_id)
            .await
            .ok_or_else(|| RuleError::ChainNotFound(self.node.chain_id))?;

        let target_node = chain
            .nodes
            .iter()
            .find(|n| n.id == *node_id)
            .ok_or_else(|| RuleError::ConfigError(format!("节点 {} 不存在", node_id)))?;

        let ctx = NodeContext::new(
            target_node,
            &ExecutionContext::new(msg.clone()),
            self.engine.clone(),
        );
        self.engine.execute_node(target_node, &ctx, msg).await
    }

    // 获取分支执行结果
    pub async fn get_branch_results(&self) -> Vec<Result<Message, RuleError>> {
        let results = self.branch_results.lock().await;
        let mut branch_msgs = Vec::new();

        // 按branch_id排序获取结果
        let mut branch_ids: Vec<_> = results.keys().collect();
        branch_ids.sort();

        for branch_id in branch_ids {
            if let Some(msg) = results.get(branch_id) {
                branch_msgs.push(Ok(msg.clone()));
            }
        }

        branch_msgs
    }

    pub async fn add_branch_result(&self, branch_id: String, msg: Message) {
        let mut results = self.branch_results.lock().await;
        results.insert(branch_id, msg);
    }
}
