use crate::types::{Message, NodeContext, RuleError};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

/// 节点拦截器特征,用于在节点执行的不同阶段进行拦截处理
#[async_trait]
pub trait NodeInterceptor: Send + Sync + std::fmt::Debug {
    /// 节点执行前的拦截处理
    ///
    /// # Arguments
    /// * `ctx` - 节点执行上下文
    /// * `msg` - 待处理的消息
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError>;

    /// 节点执行后的拦截处理
    ///
    /// # Arguments
    /// * `ctx` - 节点执行上下文
    /// * `msg` - 处理后的消息
    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError>;

    /// 节点执行出错时的拦截处理
    ///
    /// # Arguments
    /// * `ctx` - 节点执行上下文
    /// * `error` - 错误信息
    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError>;
}

/// 消息拦截器特征,用于在消息处理的不同阶段进行拦截处理
#[async_trait]
pub trait MessageInterceptor: Send + Sync + std::fmt::Debug {
    /// 消息处理前的拦截处理
    ///
    /// # Arguments
    /// * `msg` - 待处理的消息
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError>;

    /// 消息处理后的拦截处理
    ///
    /// # Arguments
    /// * `msg` - 处理后的消息
    async fn after_process(&self, msg: &Message) -> Result<(), RuleError>;
}

/// 拦截器管理器,用于管理和执行所有注册的拦截器
#[derive(Debug)]
pub struct InterceptorManager {
    /// 已注册的节点拦截器列表
    node_interceptors: Vec<Arc<dyn NodeInterceptor>>,
    /// 已注册的消息拦截器列表
    msg_interceptors: Vec<Arc<dyn MessageInterceptor>>,
}

impl InterceptorManager {
    /// 创建新的拦截器管理器实例
    pub fn new() -> Self {
        Self {
            node_interceptors: Vec::new(),
            msg_interceptors: Vec::new(),
        }
    }

    /// 注册节点拦截器
    pub fn register_node_interceptor(&mut self, interceptor: Arc<dyn NodeInterceptor>) {
        self.node_interceptors.push(interceptor);
    }

    /// 注册消息拦截器
    pub fn register_msg_interceptor(&mut self, interceptor: Arc<dyn MessageInterceptor>) {
        self.msg_interceptors.push(interceptor);
    }

    /// 执行所有节点前置拦截器
    pub async fn before_node<'a>(
        &self,
        ctx: &NodeContext<'a>,
        msg: &Message,
    ) -> Result<(), RuleError> {
        for interceptor in &self.node_interceptors {
            interceptor.before(ctx, msg).await?;
        }
        Ok(())
    }

    /// 执行所有节点后置拦截器
    pub async fn after_node<'a>(
        &self,
        ctx: &NodeContext<'a>,
        msg: &Message,
    ) -> Result<(), RuleError> {
        for interceptor in &self.node_interceptors {
            interceptor.after(ctx, msg).await?;
        }
        Ok(())
    }

    /// 执行所有节点错误拦截器
    pub async fn node_error<'a>(
        &self,
        ctx: &NodeContext<'a>,
        error: &RuleError,
    ) -> Result<(), RuleError> {
        for interceptor in &self.node_interceptors {
            interceptor.error(ctx, error).await?;
        }
        Ok(())
    }

    /// 执行所有消息前置拦截器
    pub async fn before_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("执行消息前置拦截器");
        for interceptor in &self.msg_interceptors {
            interceptor.before_process(msg).await?;
        }
        Ok(())
    }

    /// 执行所有消息后置拦截器
    pub async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("执行消息后置拦截器");
        for interceptor in &self.msg_interceptors {
            interceptor.after_process(msg).await?;
        }
        Ok(())
    }
}

/// 日志节点拦截器,用于记录节点执行的关键信息
#[derive(Debug)]
pub struct LoggingInterceptor;

#[async_trait]
impl NodeInterceptor for LoggingInterceptor {
    /// 记录节点开始执行的日志
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        info!(
            "开始执行节点 [{}], 类型: {}, 输入消息: {:?}",
            ctx.node.id, ctx.node.type_name, msg
        );
        Ok(())
    }

    /// 记录节点执行成功的日志
    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        info!(
            "节点 [{}] 执行成功, 输出消息: {:?}, 下一个分支: {:?}",
            ctx.node.id,
            msg.data,
            msg.metadata.get("branch_name")
        );
        Ok(())
    }

    /// 记录节点执行错误的日志
    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError> {
        info!("节点 [{}] 执行出错: {:?}", ctx.node.id, error);
        Ok(())
    }
}

/// 消息日志拦截器,用于记录消息处理的关键信息
#[derive(Debug)]
pub struct MessageLoggingInterceptor;

#[async_trait]
impl MessageInterceptor for MessageLoggingInterceptor {
    /// 记录消息开始处理的日志
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("开始处理消息: {:?}", msg);
        Ok(())
    }

    /// 记录消息处理完成的日志
    async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("消息处理完成: {:?}", msg);
        Ok(())
    }
}
