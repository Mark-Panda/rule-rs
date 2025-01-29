use crate::aop::{InterceptorManager, MessageInterceptor, NodeInterceptor};
use crate::components::{
    DelayConfig, DelayNode, FilterConfig, FilterNode, LogConfig, LogNode, RestClientConfig,
    RestClientNode, ScriptConfig, ScriptNode, SubchainConfig, SubchainNode, SwitchConfig,
    SwitchNode, TransformConfig, TransformJsConfig, TransformJsNode, TransformNode, WeatherConfig,
    WeatherNode,
};
use crate::engine::{NodeFactory, NodeHandler, NodeRegistry, VersionManager};
use crate::types::{
    ExecutionContext, Message, Node, NodeContext, NodeDescriptor, RuleChain, RuleError,
};
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct RuleEngine {
    pub(crate) chains: Arc<RwLock<HashMap<Uuid, Arc<RuleChain>>>>,
    node_registry: Arc<NodeRegistry>,
    version_manager: Arc<VersionManager>,
    interceptor_manager: Arc<RwLock<InterceptorManager>>,
}

impl RuleEngine {
    pub async fn new() -> Self {
        let node_registry = Arc::new(NodeRegistry::new());
        let registry = node_registry.clone();

        // 注册内置组件
        let factories: Vec<(&str, NodeFactory)> = vec![
            (
                "log",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        // 用于获取描述符时使用默认配置
                        Ok(Arc::new(LogNode::new(LogConfig {
                            template: "".to_string(),
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: LogConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(LogNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "delay",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        let node = DelayNode::new(DelayConfig {
                            delay_ms: 0,
                            periodic: false,
                            period_count: 0,
                            cron: None,
                            timezone_offset: 0,
                        })?;
                        Ok(Arc::new(node) as Arc<dyn NodeHandler>)
                    } else {
                        let config: DelayConfig = serde_json::from_value(config)?;
                        let node = DelayNode::new(config)?;
                        Ok(Arc::new(node) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "filter",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(FilterNode::new(FilterConfig {
                            condition: "true".to_string(),
                            js_script: None,
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: FilterConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(FilterNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "transform",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(TransformNode::new(TransformConfig {
                            template: serde_json::json!({}), // 使用 json! 宏创建空对象
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: TransformConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(TransformNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "transform_js",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(TransformJsNode::new(TransformJsConfig {
                            script: "return msg;".to_string(),
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: TransformJsConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(TransformJsNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "script",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(ScriptNode::new(ScriptConfig {
                            script: "return msg;".to_string(),
                            output_type: None,
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: ScriptConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(ScriptNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "switch",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(SwitchNode::new(SwitchConfig {
                            cases: Vec::new(),
                            default_next: None,
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: SwitchConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(SwitchNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "rest_client",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(RestClientNode::new(RestClientConfig {
                            url: "http://localhost".to_string(),
                            method: "GET".to_string(),
                            headers: None,
                            timeout_ms: None,
                            output_type: None,
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: RestClientConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(RestClientNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "weather",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(WeatherNode::new(WeatherConfig {
                            api_key: "demo".to_string(),
                            city: "".to_string(),
                            language: "zh".to_string(),
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: WeatherConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(WeatherNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "subchain",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(SubchainNode::new(SubchainConfig {
                            chain_id: Uuid::nil(),
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: SubchainConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(SubchainNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
        ];

        // 直接注册所有工厂
        for (type_name, factory) in factories {
            registry.register(type_name, factory).await;
        }

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

    pub async fn load_chain(&self, content: &str) -> Result<Uuid, RuleError> {
        let chain: RuleChain =
            serde_json::from_str(content).map_err(|e| RuleError::ConfigError(e.to_string()))?;

        // 检查循环依赖
        self.check_circular_dependency(&chain).await?;

        // 创建新版本
        let version = self.version_manager.create_version(&chain);

        // 更新规则链元数据
        let mut chain = chain;
        chain.metadata.version = version.version;
        chain.metadata.updated_at = version.timestamp;

        let id = chain.id;
        self.chains.write().await.insert(id, Arc::new(chain));

        Ok(id)
    }

    pub async fn load_chain_from_file(&self, path: &str) -> Result<(), RuleError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| RuleError::ConfigError(e.to_string()))?;

        self.load_chain(&content).await?;
        Ok(())
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
            let node_ctx = NodeContext::new(node, ctx, Arc::new(self.clone()));

            // 执行节点逻辑
            ctx.msg = self.execute_node(node, &node_ctx, ctx.msg.clone()).await?;

            // 获取下一个节点
            current_node = chain.get_next_node(&node.id, &ctx)?;
        }

        Ok(ctx.msg.clone())
    }

    pub async fn execute_node<'a>(
        &self,
        node: &'a Node,
        ctx: &NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        let manager = self.interceptor_manager.read().await;

        // 获取节点处理器
        let handler = self
            .node_registry
            .create_handler(&node.type_name, node.config.clone())
            .await
            .ok_or_else(|| RuleError::HandlerNotFound(node.type_name.clone()))?;

        // 节点执行前拦截
        manager.before_node(ctx, &msg).await?;

        // 执行节点
        let result = match handler.handle(ctx.clone(), msg.clone()).await {
            Ok(result) => {
                // 节点执行后拦截
                manager.after_node(ctx, &result).await?;
                Ok(result)
            }
            Err(e) => {
                // 节点错误拦截
                manager.node_error(ctx, &e).await?;
                Err(e)
            }
        };

        result
    }

    pub async fn get_current_version(&self) -> u64 {
        self.version_manager.get_current_version()
    }

    /// 获取所有已注册的组件类型
    pub async fn get_registered_components(&self) -> Vec<NodeDescriptor> {
        self.node_registry.get_descriptors().await
    }

    /// 获取所有已加载的规则链
    pub async fn get_loaded_chains(&self) -> Vec<Arc<RuleChain>> {
        self.chains.read().await.values().cloned().collect()
    }

    /// 获取指定ID的规则链
    pub async fn get_chain(&self, id: Uuid) -> Option<Arc<RuleChain>> {
        self.chains.read().await.get(&id).cloned()
    }

    /// 删除规则链
    pub async fn remove_chain(&self, id: Uuid) -> Result<(), RuleError> {
        let mut chains = self.chains.write().await;

        // 检查是否为根规则链
        if let Some(chain) = chains.get(&id) {
            if chain.root {
                return Err(RuleError::ConfigError(
                    "Cannot delete root chain".to_string(),
                ));
            }
        }

        // 检查是否被其他规则链引用
        for chain in chains.values() {
            for node in &chain.nodes {
                if let Ok(config) = serde_json::from_value::<SubchainConfig>(node.config.clone()) {
                    if config.chain_id == id {
                        return Err(RuleError::ConfigError(format!(
                            "Chain {} is referenced by chain {}",
                            id, chain.id
                        )));
                    }
                }
            }
        }

        chains
            .remove(&id)
            .ok_or_else(|| RuleError::ChainNotFound(id))?;
        Ok(())
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
