use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum RuleError {
    #[error("找不到根规则链")]
    NoRootChain,

    #[error("找不到节点处理器: {0}")]
    HandlerNotFound(String),

    #[error("节点执行失败: {0}")]
    NodeExecutionError(String),

    #[error("消息被过滤")]
    FilterReject,

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("循环依赖: {0}")]
    CircularDependency(String),

    #[error("规则链未找到: {0}")]
    ChainNotFound(Uuid),
}
