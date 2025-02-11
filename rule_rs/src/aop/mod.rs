use crate::types::{Message, NodeContext, RuleError};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

/// 节点拦截器
#[async_trait]
pub trait NodeInterceptor: Send + Sync + std::fmt::Debug {
    /// 节点执行前
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError>;

    /// 节点执行后
    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError>;

    /// 节点执行出错时
    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError>;
}

/// 消息拦截器
#[async_trait]
pub trait MessageInterceptor: Send + Sync + std::fmt::Debug {
    /// 消息处理前
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError>;

    /// 消息处理后
    async fn after_process(&self, msg: &Message) -> Result<(), RuleError>;
}

/// 拦截器管理器
#[derive(Debug)]
pub struct InterceptorManager {
    node_interceptors: Vec<Arc<dyn NodeInterceptor>>,
    msg_interceptors: Vec<Arc<dyn MessageInterceptor>>,
}

impl InterceptorManager {
    pub fn new() -> Self {
        Self {
            node_interceptors: Vec::new(),
            msg_interceptors: Vec::new(),
        }
    }

    pub fn register_node_interceptor(&mut self, interceptor: Arc<dyn NodeInterceptor>) {
        self.node_interceptors.push(interceptor);
    }

    pub fn register_msg_interceptor(&mut self, interceptor: Arc<dyn MessageInterceptor>) {
        self.msg_interceptors.push(interceptor);
    }

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

    pub async fn before_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("执行消息前置拦截器");
        for interceptor in &self.msg_interceptors {
            interceptor.before_process(msg).await?;
        }
        Ok(())
    }

    pub async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("执行消息后置拦截器");
        for interceptor in &self.msg_interceptors {
            interceptor.after_process(msg).await?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct LoggingInterceptor;

#[async_trait]
impl NodeInterceptor for LoggingInterceptor {
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        info!(
            "开始执行节点 [{}], 类型: {}, 输入消息: {:?}",
            ctx.node.id, ctx.node.type_name, msg
        );
        Ok(())
    }

    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        info!(
            "节点 [{}] 执行成功, 输出消息: {:?}, 下一个分支: {:?}",
            ctx.node.id,
            msg.data,
            msg.metadata.get("branch_name")
        );
        Ok(())
    }

    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError> {
        info!("节点 [{}] 执行出错: {:?}", ctx.node.id, error);
        Ok(())
    }
}

#[derive(Debug)]
pub struct MessageLoggingInterceptor;

#[async_trait]
impl MessageInterceptor for MessageLoggingInterceptor {
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("开始处理消息: {:?}", msg);
        Ok(())
    }

    async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        debug!("消息处理完成: {:?}", msg);
        Ok(())
    }
}
