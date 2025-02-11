use crate::aop::{
    InterceptorManager, LoggingInterceptor, MessageInterceptor, MessageLoggingInterceptor,
    NodeInterceptor,
};
use crate::components::{
    DelayConfig, DelayNode, FilterConfig, FilterNode, ForkNode, JoinConfig, JoinNode,
    JsFunctionConfig, JsFunctionNode, LogConfig, LogNode, RedisConfig, RedisNode, RedisOperation,
    RestClientConfig, RestClientNode, ScheduleConfig, ScheduleNode, ScriptConfig, ScriptNode,
    StartConfig, StartNode, SubchainConfig, SubchainNode, SwitchConfig, SwitchNode,
    TransformConfig, TransformJsConfig, TransformJsNode, TransformNode, WeatherConfig, WeatherNode,
};
use crate::engine::{NodeFactory, NodeHandler, NodeRegistry, VersionManager};
use crate::types::{
    ExecutionContext, Message, Node, NodeContext, NodeDescriptor, NodeType, RuleChain, RuleError,
};
use async_trait::async_trait;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

pub type DynRuleEngine = Arc<dyn RuleEngineTrait + Send + Sync>;

#[async_trait]
pub trait RuleEngineTrait: Debug + Send + Sync {
    async fn check_circular_dependency(&self, chain: &RuleChain) -> Result<(), RuleError>;
    async fn check_circular_dependency1(&self, chain: &RuleChain) -> Result<(), RuleError>;
    async fn load_chain(&self, content: &str) -> Result<Uuid, RuleError>;
    async fn load_chain_from_file(&self, path: &str) -> Result<(), RuleError>;
    async fn add_node_interceptor(&self, interceptor: Arc<dyn NodeInterceptor>);
    async fn add_msg_interceptor(&self, interceptor: Arc<dyn MessageInterceptor>);
    async fn process_msg(&self, chain_id: Uuid, msg: Message) -> Result<Message, RuleError>;
    async fn execute_chain(
        &self,
        chain: &RuleChain,
        ctx: &mut ExecutionContext,
    ) -> Result<Message, RuleError>;
    async fn execute_node<'a>(
        &self,
        node: &'a Node,
        ctx: &NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError>;
    async fn get_current_version(&self) -> u64;
    async fn get_registered_components(&self) -> Vec<NodeDescriptor>;
    async fn get_loaded_chains(&self) -> Vec<Arc<RuleChain>>;
    async fn get_chain(&self, id: Uuid) -> Option<Arc<RuleChain>>;
    async fn remove_chain(&self, id: Uuid) -> Result<(), RuleError>;
    async fn register_node_type(&self, type_name: &str, factory: NodeFactory);
    async fn get_component_descriptor(&self, type_name: &str) -> Option<NodeDescriptor>;
}

#[derive(Debug, Clone)]
pub struct RuleEngine {
    pub(crate) chains: Arc<RwLock<HashMap<Uuid, Arc<RuleChain>>>>,
    node_registry: Arc<NodeRegistry>,
    version_manager: Arc<VersionManager>,
    interceptor_manager: Arc<RwLock<InterceptorManager>>,
    execution_counters: Arc<RwLock<HashMap<Uuid, Arc<Mutex<usize>>>>>,
}

impl RuleEngine {
    pub async fn new() -> Self {
        let node_registry = Arc::new(NodeRegistry::new());
        let registry = node_registry.clone();

        // 注册内置组件
        let factories: Vec<(&str, NodeFactory)> = vec![
            (
                "redis",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(RedisNode::new(RedisConfig {
                            url: "redis://localhost:6379".to_string(),
                            operation: RedisOperation::Raw {
                                command: "PING".to_string(),
                                args: vec![],
                            },
                            key: String::new(),
                            field: None,
                            value: None,
                            values: None,
                            score: None,
                            ttl: None,
                            start: None,
                            stop: None,
                            success_branch: None,
                            error_branch: None,
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: RedisConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(RedisNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "log",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
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
                "start",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(StartNode::new(StartConfig::default()))
                            as Arc<dyn NodeHandler>)
                    } else {
                        let config: StartConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(StartNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "delay",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(DelayNode::new(DelayConfig::default()))
                            as Arc<dyn NodeHandler>)
                    } else {
                        let config: DelayConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(DelayNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "schedule",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(ScheduleNode::new(ScheduleConfig::default()))
                            as Arc<dyn NodeHandler>)
                    } else {
                        let config: ScheduleConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(ScheduleNode::new(config)) as Arc<dyn NodeHandler>)
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
                            template: json!({}),
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
                            success_branch: None,
                            error_branch: None,
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
            (
                "js_function",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(JsFunctionNode::new(JsFunctionConfig {
                            functions: HashMap::new(),
                            main: "main".to_string(),
                            chain_id: String::new(),
                            node_id: String::new(),
                        })) as Arc<dyn NodeHandler>)
                    } else {
                        let config: JsFunctionConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(JsFunctionNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
            (
                "fork",
                Arc::new(|_config| Ok(Arc::new(ForkNode::new()) as Arc<dyn NodeHandler>)),
            ),
            (
                "join",
                Arc::new(|config| {
                    if config.is_object() && config.as_object().unwrap().is_empty() {
                        Ok(Arc::new(JoinNode::new(JoinConfig {})) as Arc<dyn NodeHandler>)
                    } else {
                        let config: JoinConfig = serde_json::from_value(config)?;
                        Ok(Arc::new(JoinNode::new(config)) as Arc<dyn NodeHandler>)
                    }
                }),
            ),
        ];

        // 直接注册所有工厂
        for (type_name, factory) in factories {
            registry.register(type_name, factory).await;
        }

        let engine = Self {
            chains: Arc::new(RwLock::new(HashMap::new())),
            node_registry,
            version_manager: Arc::new(VersionManager::new()),
            interceptor_manager: Arc::new(RwLock::new(InterceptorManager::new())),
            execution_counters: Arc::new(RwLock::new(HashMap::new())),
        };

        // 注册默认拦截器
        {
            let mut manager = engine.interceptor_manager.write().await;
            // 注册节点日志拦截器
            manager.register_node_interceptor(Arc::new(LoggingInterceptor));
            // 注册消息日志拦截器
            manager.register_msg_interceptor(Arc::new(MessageLoggingInterceptor));
        }

        engine
    }

    // 增加执行计数
    async fn increment_counter(&self, chain_id: Uuid) {
        let counter = {
            let mut counters = self.execution_counters.write().await;
            counters
                .entry(chain_id)
                .or_insert_with(|| Arc::new(Mutex::new(0)))
                .clone()
        };
        let mut count = counter.lock().await;
        *count += 1;
    }

    // 减少执行计数
    async fn decrement_counter(&self, chain_id: Uuid) {
        if let Some(counter) = self.execution_counters.read().await.get(&chain_id) {
            let mut count = counter.lock().await;
            *count = count.saturating_sub(1);
        }
    }
}

#[async_trait]
impl RuleEngineTrait for RuleEngine {
    async fn get_component_descriptor(&self, type_name: &str) -> Option<NodeDescriptor> {
        self.node_registry.get_descriptor(type_name).await
    }
    async fn get_registered_components(&self) -> Vec<NodeDescriptor> {
        self.node_registry.get_descriptors().await
    }
    async fn check_circular_dependency(&self, chain: &RuleChain) -> Result<(), RuleError> {
        let chains = self.chains.read().await;

        // 全局节点访问记录（链ID + 节点ID）
        let mut global_visited = HashSet::new();
        // 当前节点调用栈（链ID + 节点ID）
        let mut node_stack = Vec::new();
        // 规则链调用栈
        let mut chain_stack = Vec::new();

        async fn check_dependencies<'a>(
            chain: &'a Arc<RuleChain>,
            node_id: &Uuid,
            chains: &'a HashMap<Uuid, Arc<RuleChain>>,
            global_visited: &mut HashSet<(Uuid, Uuid)>,
            node_stack: &mut Vec<(Uuid, Uuid)>,
            chain_stack: &mut Vec<Uuid>,
        ) -> Result<(), RuleError> {
            let node_key = (chain.id, *node_id);

            // 检查节点级循环
            if node_stack.contains(&node_key) {
                let cycle_path: Vec<String> = node_stack
                    .iter()
                    .skip_while(|&x| x != &node_key)
                    .chain(std::iter::once(&node_key))
                    .map(|(c, n)| {
                        let current_chain = chains.get(c).unwrap_or(chain);
                        let node = current_chain
                            .nodes
                            .iter()
                            .find(|nd| &nd.id == n)
                            .map_or_else(
                                || format!("未知节点({})", n),
                                |nd| format!("{}[{}]", current_chain.name, nd.type_name),
                            );
                        node
                    })
                    .collect();

                return Err(RuleError::CircularDependency(format!(
                    "节点循环依赖: {}",
                    cycle_path.join(" -> ")
                )));
            }

            // 检查链级循环
            if chain_stack.contains(&chain.id) {
                let chain_cycle: Vec<String> = chain_stack
                    .iter()
                    .skip_while(|&x| x != &chain.id)
                    .chain(std::iter::once(&chain.id))
                    .map(|id| {
                        chains
                            .get(id)
                            .map_or_else(|| format!("未加载链({})", id), |c| c.name.clone())
                    })
                    .collect();

                return Err(RuleError::CircularDependency(format!(
                    "规则链循环依赖: {}",
                    chain_cycle.join(" -> ")
                )));
            }

            // 已检查过该节点
            if global_visited.contains(&node_key) {
                return Ok(());
            }

            // 记录访问状态
            global_visited.insert(node_key);
            node_stack.push(node_key);
            chain_stack.push(chain.id);

            // 处理当前节点
            let node = chain
                .nodes
                .iter()
                .find(|n| &n.id == node_id)
                .ok_or_else(|| RuleError::ConfigError("节点不存在".to_string()))?;

            // 处理子链节点
            if node.type_name == "subchain" {
                let config: SubchainConfig = serde_json::from_value(node.config.clone())
                    .map_err(|e| RuleError::ConfigError(format!("子链配置解析失败: {}", e)))?;

                if let Some(subchain) = chains.get(&config.chain_id) {
                    // 获取子链的起始节点
                    let start_node = subchain.get_start_node()?.ok_or_else(|| {
                        RuleError::ConfigError(format!("子链 {} 没有起始节点", subchain.id))
                    })?;

                    // 递归检查子链
                    Box::pin(check_dependencies(
                        subchain,
                        &start_node.id,
                        chains,
                        global_visited,
                        node_stack,
                        chain_stack,
                    ))
                    .await?;
                }
            }

            // 处理后续连接
            for conn in chain.connections.iter().filter(|c| &c.from_id == node_id) {
                Box::pin(check_dependencies(
                    chain,
                    &conn.to_id,
                    chains,
                    global_visited,
                    node_stack,
                    chain_stack,
                ))
                .await?;
            }

            // 回溯状态
            node_stack.pop();
            chain_stack.pop();

            Ok(())
        }

        // 从起始节点开始检查
        let start_node = chain
            .get_start_node()?
            .ok_or(RuleError::ConfigError("规则链没有起始节点".to_string()))?;

        let current_chain = self
            .get_chain(chain.id)
            .await
            .ok_or_else(|| RuleError::ConfigError(format!("规则链 {} 未正确加载", chain.id)))?;

        Box::pin(check_dependencies(
            &current_chain,
            &start_node.id,
            &chains,
            &mut global_visited,
            &mut node_stack,
            &mut chain_stack,
        ))
        .await
    }

    async fn check_circular_dependency1(&self, chain: &RuleChain) -> Result<(), RuleError> {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let mut chain_stack = HashSet::new();

        async fn check_subchain_node<'a>(
            node: &'a Node,
            chain_stack: &'a mut HashSet<Uuid>,
            chain_id: Uuid,
            chains: &'a HashMap<Uuid, Arc<RuleChain>>,
        ) -> Result<(), RuleError> {
            if let "subchain" = node.type_name.as_str() {
                if let Ok(config) = serde_json::from_value::<SubchainConfig>(node.config.clone()) {
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
                            Box::pin(check_subchain_node(node, chain_stack, subchain.id, chains))
                                .await?;
                        }
                    }
                }
            }
            Ok(())
        }

        async fn dfs<'a>(
            node_id: &'a Uuid,
            chain: &'a RuleChain,
            visited: &'a mut HashSet<Uuid>,
            stack: &'a mut HashSet<Uuid>,
            chain_stack: &'a mut HashSet<Uuid>,
            chains: &'a HashMap<Uuid, Arc<RuleChain>>,
        ) -> Result<(), RuleError> {
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
                Box::pin(check_subchain_node(node, chain_stack, chain.id, chains)).await?;
            }

            // 遍历所有后继节点
            for conn in chain.connections.iter().filter(|c| &c.from_id == node_id) {
                Box::pin(dfs(&conn.to_id, chain, visited, stack, chain_stack, chains)).await?;
            }

            stack.remove(node_id);
            Ok(())
        }

        // 获取所有已加载的规则链
        let chains = self.chains.read().await;

        // 从每个节点开始检查
        for node in &chain.nodes {
            Box::pin(dfs(
                &node.id,
                chain,
                &mut visited,
                &mut stack,
                &mut chain_stack,
                &chains,
            ))
            .await?;
        }

        Ok(())
    }

    async fn load_chain(&self, content: &str) -> Result<Uuid, RuleError> {
        let chain: RuleChain =
            serde_json::from_str(content).map_err(|e| RuleError::ConfigError(e.to_string()))?;

        let start_node = chain
            .get_start_node()?
            .ok_or_else(|| RuleError::ConfigError("规则链没有起始节点".to_string()))?;

        let descriptor = self
            .get_component_descriptor(&start_node.type_name)
            .await
            .ok_or_else(|| {
                RuleError::ConfigError(format!("未找到节点类型: {}", start_node.type_name))
            })?;

        let node_type = descriptor.node_type;
        if node_type != NodeType::Head {
            return Err(RuleError::ConfigError(
                "规则链必须以header节点开始".to_string(),
            ));
        }
        chain.validate(self).await?;

        // 启用循环依赖检查
        self.check_circular_dependency1(&chain).await?;

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

    async fn load_chain_from_file(&self, path: &str) -> Result<(), RuleError> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| RuleError::ConfigError(e.to_string()))?;

        self.load_chain(&content).await?;
        Ok(())
    }

    async fn add_node_interceptor(&self, interceptor: Arc<dyn NodeInterceptor>) {
        self.interceptor_manager
            .write()
            .await
            .register_node_interceptor(interceptor);
    }

    async fn add_msg_interceptor(&self, interceptor: Arc<dyn MessageInterceptor>) {
        self.interceptor_manager
            .write()
            .await
            .register_msg_interceptor(interceptor);
    }

    async fn process_msg(&self, chain_id: Uuid, msg: Message) -> Result<Message, RuleError> {
        let manager = self.interceptor_manager.read().await;

        // 消息处理前拦截
        manager.before_process(&msg).await?;

        // 查找指定的规则链
        let chain = self
            .get_chain(chain_id)
            .await
            .ok_or(RuleError::ChainNotFound(chain_id))?;

        // 检查是否为根规则链
        if !chain.root {
            return Err(RuleError::ConfigError(format!(
                "Chain {} is not a root chain",
                chain_id
            )));
        }

        // 创建执行上下文并执行规则链
        let mut ctx = ExecutionContext::new(msg.clone());
        let result = self.execute_chain(&chain, &mut ctx).await?;

        // 消息处理后拦截
        manager.after_process(&msg).await?;

        Ok(result)
    }

    async fn execute_chain(
        &self,
        chain: &RuleChain,
        ctx: &mut ExecutionContext,
    ) -> Result<Message, RuleError> {
        // 增加计数
        self.increment_counter(chain.id).await;

        // 使用 defer 模式确保计数器一定会减少
        let result = async {
            let start_node = chain
                .get_start_node()?
                .ok_or_else(|| RuleError::ConfigError("规则链没有起始节点".to_string()))?;

            let node_ctx = NodeContext::new(start_node, ctx, Arc::new(self.clone()));
            self.execute_node(start_node, &node_ctx, ctx.msg.clone())
                .await
        }
        .await;

        // 减少计数
        self.decrement_counter(chain.id).await;

        result
    }

    async fn execute_node<'a>(
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

    async fn get_current_version(&self) -> u64 {
        self.version_manager.get_current_version()
    }

    /// 获取所有已加载的规则链
    async fn get_loaded_chains(&self) -> Vec<Arc<RuleChain>> {
        self.chains.read().await.values().cloned().collect()
    }

    /// 获取指定ID的规则链
    async fn get_chain(&self, id: Uuid) -> Option<Arc<RuleChain>> {
        self.chains.read().await.get(&id).cloned()
    }

    /// 删除规则链
    async fn remove_chain(&self, id: Uuid) -> Result<(), RuleError> {
        // 先用读锁检查规则链是否存在
        {
            let chains = self.chains.read().await;
            if !chains.contains_key(&id) {
                return Err(RuleError::ChainNotFound(id));
            }

            // 检查引用关系
            for chain in chains.values() {
                for node in &chain.nodes {
                    if let Ok(config) =
                        serde_json::from_value::<SubchainConfig>(node.config.clone())
                    {
                        if config.chain_id == id {
                            return Err(RuleError::ConfigError(format!(
                                "规则链 {} 被规则链 {} 引用,无法删除",
                                id, chain.id
                            )));
                        }
                    }
                }
            }
        }

        // 等待执行完成，最多等待 5 秒
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(5);

        while start_time.elapsed() < timeout {
            let has_running_instances = {
                let counters = self.execution_counters.read().await;
                if let Some(counter) = counters.get(&id) {
                    let count = counter.lock().await;
                    *count > 0
                } else {
                    false
                }
            };

            if !has_running_instances {
                break;
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // 再次检查是否还有运行实例
        let has_running_instances = {
            let counters = self.execution_counters.read().await;
            if let Some(counter) = counters.get(&id) {
                let count = counter.lock().await;
                *count > 0
            } else {
                false
            }
        };

        if has_running_instances {
            return Err(RuleError::ConfigError(format!(
                "无法删除正在执行的规则链 {}, 等待超时",
                id
            )));
        }

        // 获取写锁并删除
        {
            let mut chains = self.chains.write().await;
            chains.remove(&id);
        }

        // 清理计数器
        {
            let mut counters = self.execution_counters.write().await;
            counters.remove(&id);
        }

        Ok(())
    }

    /// 注册自定义节点类型
    async fn register_node_type(&self, type_name: &str, factory: NodeFactory) {
        self.node_registry.register(type_name, factory).await;
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
        ctx: &ExecutionContext,
    ) -> Result<Option<&Node>, RuleError> {
        // 获取所有从当前节点出发的连接
        let next_conns: Vec<_> = self
            .connections
            .iter()
            .filter(|conn| &conn.from_id == current_id)
            .collect();

        // 如果没有连接，返回 None
        if next_conns.is_empty() {
            return Ok(None);
        }

        // 检查消息元数据中的分支名称
        if let Some(branch) = ctx.msg.metadata.get("branch_name") {
            // 查找匹配分支名称的连接
            if let Some(conn) = next_conns
                .iter()
                .find(|conn| conn.type_name == branch.to_string())
            {
                return self
                    .nodes
                    .iter()
                    .find(|node| node.id == conn.to_id)
                    .ok_or_else(|| RuleError::ConfigError("Invalid connection".to_string()))
                    .map(Some);
            }
        }

        // 如果没有匹配的分支，使用第一个连接
        let conn = &next_conns[0];
        self.nodes
            .iter()
            .find(|node| node.id == conn.to_id)
            .ok_or_else(|| RuleError::ConfigError("Invalid connection".to_string()))
            .map(Some)
    }

    pub async fn validate(&self, engine: &RuleEngine) -> Result<(), RuleError> {
        for node in &self.nodes {
            let node_type = Self::get_node_type(engine, node).await?;
            // 检查头节点不能被指向
            if node_type == NodeType::Head {
                let has_incoming = self.connections.iter().any(|conn| conn.to_id == node.id);
                if has_incoming {
                    return Err(RuleError::ConfigError(format!(
                        "头结点 {} 不能被其他节点指向",
                        node.type_name
                    )));
                }
            }

            // 检查尾节点不能指向其他节点
            if node_type == NodeType::Tail {
                let has_outgoing = self.connections.iter().any(|conn| conn.from_id == node.id);
                if has_outgoing {
                    return Err(RuleError::ConfigError(format!(
                        "尾节点 {} 不能指向其他节点",
                        node.type_name
                    )));
                }
            }
        }
        Ok(())
    }

    async fn get_node_type(engine: &RuleEngine, node: &Node) -> Result<NodeType, RuleError> {
        let descriptor = engine
            .get_component_descriptor(&node.type_name)
            .await
            .ok_or_else(|| RuleError::ConfigError(format!("未找到节点类型: {}", node.type_name)))?;
        Ok(descriptor.node_type)
    }
}
