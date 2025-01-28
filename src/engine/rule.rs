use crate::aop::{InterceptorManager, MessageInterceptor, NodeInterceptor};
use crate::components::{
    FilterConfig, FilterNode, RestClientConfig, RestClientNode, ScriptConfig, ScriptNode,
    SubchainConfig, SubchainNode, SwitchConfig, SwitchNode, TransformConfig, TransformJsConfig,
    TransformJsNode, TransformNode,
};
use crate::engine::{NodeRegistry, VersionManager};
use crate::types::{ExecutionContext, Message, Node, NodeContext, RuleChain, RuleError};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct RuleEngine {
    chains: Arc<RwLock<HashMap<Uuid, Arc<RuleChain>>>>,
    node_registry: Arc<NodeRegistry>,
    version_manager: Arc<VersionManager>,
    interceptor_manager: Arc<RwLock<InterceptorManager>>,
}

impl RuleEngine {
    pub fn new() -> Self {
        let node_registry = Arc::new(NodeRegistry::new());
        Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            node_registry,
            version_manager: Arc::new(VersionManager::new()),
            interceptor_manager: Arc::new(RwLock::new(InterceptorManager::new())),
        }
    }

    async fn check_circular_dependency(&self, chain: &RuleChain) -> Result<(), RuleError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut chain_stack = HashSet::new();

        async fn check_subchain_node<'a>(
            node: &'a Node,
            chain_stack: &'a mut HashSet<Uuid>,
            chain_id: Uuid,
            chains: &'a HashMap<Uuid, Arc<RuleChain>>,
        ) -> Pin<Box<dyn Future<Output = Result<(), RuleError>> + 'a>> {
            Box::pin(async move {
                if let "subchain" = node.type_name.as_str() {
                    if let Ok(config) =
                        serde_json::from_value::<SubchainConfig>(node.config.clone())
                    {
                        // 检查子规则链是否形成循环
                        if chain_stack.contains(&config.chain_id) {
                            let chain_names = chain_stack
                                .iter()
                                .map(|id| id.to_string())
                                .collect::<Vec<_>>()
                                .join(" -> ");
                            return Err(RuleError::CircularDependency(format!(
                                "检测到规则链循环依赖: {} -> {}",
                                chain_names, config.chain_id
                            )));
                        }
                        chain_stack.insert(chain_id);

                        // 递归检查已加载的子规则链
                        if let Some(subchain) = chains.get(&config.chain_id) {
                            for node in &subchain.nodes {
                                check_subchain_node(node, chain_stack, subchain.id, chains)
                                    .await
                                    .await?;
                            }
                        }
                    }
                }
                Ok(())
            })
        }

        async fn dfs<'a>(
            node_id: &'a Uuid,
            chain: &'a RuleChain,
            visited: &'a mut HashSet<Uuid>,
            stack: &'a mut HashSet<Uuid>,
            chain_stack: &'a mut HashSet<Uuid>,
            chains: &'a HashMap<Uuid, Arc<RuleChain>>,
        ) -> Pin<Box<dyn Future<Output = Result<(), RuleError>> + 'a>> {
            Box::pin(async move {
                if stack.contains(node_id) {
                    let node_names: Vec<_> = stack
                        .iter()
                        .chain(std::iter::once(node_id))
                        .filter_map(|id| {
                            chain
                                .nodes
                                .iter()
                                .find(|n| &n.id == id)
                                .map(|n| n.type_name.clone())
                        })
                        .collect();

                    return Err(RuleError::CircularDependency(format!(
                        "检测到节点循环依赖: {}",
                        node_names.join(" -> ")
                    )));
                }

                if visited.contains(node_id) {
                    return Ok(());
                }

                visited.insert(*node_id);
                stack.insert(*node_id);

                // 检查当前节点是否是子规则链节点
                if let Some(node) = chain.nodes.iter().find(|n| &n.id == node_id) {
                    check_subchain_node(node, chain_stack, chain.id, chains)
                        .await
                        .await?;
                }

                // 遍历所有后继节点
                for conn in chain.connections.iter().filter(|c| &c.from_id == node_id) {
                    dfs(&conn.to_id, chain, visited, stack, chain_stack, chains)
                        .await
                        .await?;
                }

                stack.remove(node_id);
                Ok(())
            })
        }

        // 获取所有已加载的规则链
        let chains = self.chains.read().await;

        // 从每个节点开始检查
        for node in &chain.nodes {
            dfs(
                &node.id,
                chain,
                &mut visited,
                &mut stack,
                &mut chain_stack,
                &chains,
            )
            .await
            .await?;
        }

        Ok(())
    }

    pub async fn load_chain(&self, content: &str) -> Result<(), RuleError> {
        let chain: RuleChain =
            serde_json::from_str(content).map_err(|e| RuleError::ConfigError(e.to_string()))?;

        // 检查循环依赖
        self.check_circular_dependency(&chain).await?;

        // 创建新版本
        let version = self.version_manager.create_version(&chain);

        // 注册节点处理器
        for node in &chain.nodes {
            match node.type_name.as_str() {
                "filter" => {
                    let config: FilterConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(FilterNode { config });
                    self.node_registry.register(&node.type_name, handler).await;
                }
                "transform" => {
                    let config: TransformConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(TransformNode { config });
                    self.node_registry.register(&node.type_name, handler).await;
                }
                "transform_js" => {
                    let config: TransformJsConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(TransformJsNode { config });
                    self.node_registry.register(&node.type_name, handler).await;
                }
                "script" => {
                    let config: ScriptConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(ScriptNode { config });
                    self.node_registry.register(&node.type_name, handler).await;
                }
                "switch" => {
                    let config: SwitchConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(SwitchNode { config });
                    self.node_registry.register(&node.type_name, handler).await;
                }
                "rest_client" => {
                    let config: RestClientConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(RestClientNode::new(config));
                    self.node_registry.register(&node.type_name, handler).await;
                }
                "subchain" => {
                    let config: SubchainConfig = serde_json::from_value(node.config.clone())
                        .map_err(|e| RuleError::ConfigError(e.to_string()))?;
                    let handler = Arc::new(SubchainNode::new(config, Arc::new(self.clone())));
                    self.node_registry.register(&node.type_name, handler).await;
                }
                _ => {
                    return Err(RuleError::ConfigError(format!(
                        "Unknown node type: {}",
                        node.type_name
                    )))
                }
            }
        }

        // 更新规则链元数据
        let mut chain = chain;
        chain.metadata.version = version.version;
        chain.metadata.updated_at = version.timestamp;

        self.chains.write().await.insert(chain.id, Arc::new(chain));
        Ok(())
    }

    pub async fn load_chain_from_file(&self, path: &str) -> Result<(), RuleError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| RuleError::ConfigError(e.to_string()))?;

        self.load_chain(&content).await
    }

    pub async fn add_node_interceptor(&self, interceptor: Arc<dyn NodeInterceptor>) {
        self.interceptor_manager
            .write()
            .await
            .add_node_interceptor(interceptor);
    }

    pub async fn add_msg_interceptor(&self, interceptor: Arc<dyn MessageInterceptor>) {
        self.interceptor_manager
            .write()
            .await
            .add_msg_interceptor(interceptor);
    }

    pub async fn process_msg(&self, msg: Message) -> Result<Message, RuleError> {
        let manager = self.interceptor_manager.read().await;

        // 消息处理前拦截
        manager.before_process(&msg).await?;

        let chains = self.chains.read().await;

        // 查找根规则链
        let root_chain = chains
            .values()
            .find(|c| c.root)
            .ok_or(RuleError::NoRootChain)?;

        // 创建执行上下文
        let mut ctx = ExecutionContext::new(msg.clone());

        // 执行规则链
        let result = self.execute_chain(root_chain, &mut ctx).await?;

        // 消息处理后拦截
        manager.after_process(&msg).await?;

        Ok(result)
    }

    pub async fn execute_chain(
        &self,
        chain: &RuleChain,
        ctx: &mut ExecutionContext,
    ) -> Result<Message, RuleError> {
        let mut current_node = chain.get_start_node()?;

        while let Some(node) = current_node {
            // 创建节点上下文
            let node_ctx = NodeContext::new(node, ctx);

            // 执行节点逻辑
            ctx.msg = self.execute_node(node, node_ctx, ctx.msg.clone()).await?;

            // 获取下一个节点
            current_node = chain.get_next_node(&node.id, &ctx)?;
        }

        Ok(ctx.msg.clone())
    }

    async fn execute_node<'a>(
        &self,
        node: &'a Node,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        let manager = self.interceptor_manager.read().await;

        // 获取节点处理器
        let handler = self
            .node_registry
            .get_handler(&node.type_name)
            .await
            .ok_or_else(|| RuleError::HandlerNotFound(node.type_name.clone()))?;

        // 节点执行前拦截
        manager.before_node(&ctx, &msg).await?;

        // 执行节点
        let result = match handler.handle(ctx.clone(), msg.clone()).await {
            Ok(result) => {
                // 节点执行后拦截
                manager.after_node(&ctx, &result).await?;
                Ok(result)
            }
            Err(e) => {
                // 节点错误拦截
                manager.node_error(&ctx, &e).await?;
                Err(e)
            }
        };

        result
    }

    pub async fn get_current_version(&self) -> u64 {
        self.version_manager.get_current_version()
    }

    pub async fn get_chain(&self, id: Uuid) -> Option<Arc<RuleChain>> {
        self.chains.read().await.get(&id).cloned()
    }
}

impl RuleChain {
    pub fn get_start_node(&self) -> Result<Option<&Node>, RuleError> {
        self.nodes
            .first()
            .ok_or(RuleError::ConfigError("Empty rule chain".to_string()))
            .map(Some)
    }

    pub fn get_next_node(
        &self,
        current_id: &Uuid,
        _ctx: &ExecutionContext,
    ) -> Result<Option<&Node>, RuleError> {
        let next_conn = self
            .connections
            .iter()
            .find(|conn| &conn.from_id == current_id);

        if let Some(conn) = next_conn {
            self.nodes
                .iter()
                .find(|node| node.id == conn.to_id)
                .ok_or_else(|| RuleError::ConfigError("Invalid connection".to_string()))
                .map(Some)
        } else {
            Ok(None)
        }
    }
}
