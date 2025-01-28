use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{types::*, NodeRegistry};

pub struct RuleEngine {
    chains: RwLock<HashMap<Uuid, Arc<RuleChain>>>,
    node_registry: Arc<NodeRegistry>,
    version_manager: Arc<VersionManager>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            chains: RwLock::new(HashMap::new()),
            node_registry: Arc::new(NodeRegistry::new()),
            version_manager: Arc::new(VersionManager::new()),
        }
    }

    pub async fn process_msg(&self, msg: Message) -> Result<Message, RuleError> {
        let chains = self.chains.read().await;
        
        // 查找根规则链
        let root_chain = chains.values()
            .find(|c| c.root)
            .ok_or(RuleError::NoRootChain)?;

        // 创建执行上下文
        let ctx = ExecutionContext::new(msg);
        
        // 执行规则链
        self.execute_chain(root_chain, ctx).await
    }

    async fn execute_chain(&self, chain: &RuleChain, mut ctx: ExecutionContext) 
        -> Result<Message, RuleError> {
        
        let mut current_node = chain.get_start_node()?;
        
        while let Some(node) = current_node {
            // 获取节点处理器
            let handler = self.node_registry
                .get_handler(&node.type_name)
                .await
                .ok_or(RuleError::HandlerNotFound)?;
            
            // 创建节点上下文
            let node_ctx = NodeContext::new(&node, &ctx);
            
            // 执行节点逻辑
            ctx.msg = handler.handle(node_ctx, ctx.msg).await?;
            
            // 获取下一个节点
            current_node = chain.get_next_node(&node.id, &ctx)?;
        }
        
        Ok(ctx.msg)
    }
} 