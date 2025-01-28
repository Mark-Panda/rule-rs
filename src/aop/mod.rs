use crate::types::{Message, NodeContext, RuleError};
use async_trait::async_trait;
use std::sync::Arc;

/// 节点拦截器
#[async_trait]
pub trait NodeInterceptor: Send + Sync {
    /// 节点执行前
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError>;

    /// 节点执行后
    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError>;

    /// 节点执行出错时
    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError>;
}

/// 消息拦截器
#[async_trait]
pub trait MessageInterceptor: Send + Sync {
    /// 消息处理前
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError>;

    /// 消息处理后
    async fn after_process(&self, msg: &Message) -> Result<(), RuleError>;
}

/// 拦截器管理器
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

    pub fn add_node_interceptor(&mut self, interceptor: Arc<dyn NodeInterceptor>) {
        self.node_interceptors.push(interceptor);
    }

    pub fn add_msg_interceptor(&mut self, interceptor: Arc<dyn MessageInterceptor>) {
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
        for interceptor in &self.msg_interceptors {
            interceptor.before_process(msg).await?;
        }
        Ok(())
    }

    pub async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        for interceptor in &self.msg_interceptors {
            interceptor.after_process(msg).await?;
        }
        Ok(())
    }
}
