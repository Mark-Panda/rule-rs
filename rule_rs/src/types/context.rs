use crate::engine::DynRuleEngine;
use crate::types::{Connection, Message, Node, RuleError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

/// 节点执行上下文,包含节点执行所需的所有信息
#[derive(Debug, Clone)]
pub struct NodeContext<'a> {
    /// 当前执行的节点
    pub node: &'a Node,
    /// 上下文元数据,用于在节点间传递信息
    pub metadata: HashMap<String, String>,
    /// 规则引擎实例
    pub engine: DynRuleEngine,
    /// 当前处理的消息
    pub msg: Message,
    /// 分支执行结果,用于存储并行分支的执行结果
    branch_results: Arc<Mutex<HashMap<String, Message>>>,
}

/// 规则链执行上下文,包含规则链执行过程中的状态信息
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// 当前处理的消息
    pub msg: Message,
    /// 上下文元数据,用于在规则链执行过程中传递信息
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    /// 创建新的执行上下文
    ///
    /// # Arguments
    /// * `msg` - 待处理的消息
    pub fn new(msg: Message) -> Self {
        Self {
            msg,
            metadata: HashMap::new(),
        }
    }
}

impl<'a> NodeContext<'a> {
    /// 创建新的节点上下文
    ///
    /// # Arguments
    /// * `node` - 当前节点
    /// * `ctx` - 执行上下文
    /// * `engine` - 规则引擎实例
    pub fn new(node: &'a Node, ctx: &ExecutionContext, engine: DynRuleEngine) -> Self {
        Self {
            node,
            metadata: ctx.metadata.clone(),
            engine,
            msg: ctx.msg.clone(),
            branch_results: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建子规则链的执行上下文
    pub fn create_subchain_context(&self) -> ExecutionContext {
        ExecutionContext {
            msg: self.msg.clone(),
            metadata: self.metadata.clone(),
        }
    }

    /// 发送消息到下一个节点
    ///
    /// # Arguments
    /// * `msg` - 要发送的消息
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
            let ctx = NodeContext::new(node, &exec_ctx, self.engine.clone());
            self.engine.execute_node(node, &ctx, exec_ctx.msg).await?;
        }

        Ok(())
    }

    /// 设置下一个要执行的分支名称
    ///
    /// # Arguments
    /// * `branch` - 分支名称
    pub fn set_next_branch(&mut self, branch: &str) {
        self.metadata
            .insert("branch_name".to_string(), branch.to_string());
    }

    /// 获取指定类型的下一个连接
    ///
    /// # Arguments
    /// * `type_name` - 连接类型名称
    ///
    /// # Returns
    /// * `Result<Vec<Connection>, RuleError>` - 符合条件的连接列表
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

    /// 发送消息到指定节点
    ///
    /// # Arguments
    /// * `node_id` - 目标节点ID
    /// * `msg` - 要发送的消息
    ///
    /// # Returns
    /// * `Result<Message, RuleError>` - 节点处理后的消息或错误
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

    /// 获取所有分支的执行结果
    ///
    /// # Returns
    /// * `Vec<Result<Message, RuleError>>` - 分支执行结果列表
    pub async fn get_branch_results(&self) -> Vec<Result<Message, RuleError>> {
        let results = self.branch_results.lock().await;
        let mut branch_msgs = Vec::new();

        // 按branch_id排序获取结果
        let mut branch_ids: Vec<_> = results.keys().collect();
        branch_ids.sort_by(|a, b| {
            a.parse::<usize>()
                .unwrap()
                .cmp(&b.parse::<usize>().unwrap())
        });

        for branch_id in branch_ids {
            if let Some(msg) = results.get(branch_id) {
                branch_msgs.push(Ok(msg.clone()));
            }
        }

        branch_msgs
    }

    /// 添加分支执行结果
    ///
    /// # Arguments
    /// * `branch_id` - 分支ID
    /// * `msg` - 分支执行结果消息
    pub async fn add_branch_result(&self, branch_id: String, msg: Message) {
        let mut results = self.branch_results.lock().await;
        results.insert(branch_id, msg);
    }
}
