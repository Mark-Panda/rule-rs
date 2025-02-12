mod context;
mod descriptor;
mod error;
mod message;
mod node;

pub use context::*;
pub use descriptor::*;
pub use error::*;
pub use message::*;
pub use node::*;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 规则链定义,描述了一个完整的规则处理流程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleChain {
    /// 规则链唯一标识
    pub id: Uuid,
    /// 规则链名称
    pub name: String,
    /// 是否为根规则链
    pub root: bool,
    /// 规则链中的所有节点
    pub nodes: Vec<Node>,
    /// 节点之间的连接关系
    pub connections: Vec<Connection>,
    /// 规则链元数据
    pub metadata: Metadata,
}

/// 节点之间的连接定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    /// 起始节点ID
    pub from_id: Uuid,
    /// 目标节点ID
    pub to_id: Uuid,
    /// 连接类型名称,用于条件路由
    pub type_name: String,
}

/// 规则链元数据信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// 版本号
    pub version: u64,
    /// 创建时间戳
    pub created_at: i64,
    /// 最后更新时间戳
    pub updated_at: i64,
}

/// 节点类型枚举
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub enum NodeType {
    /// 头节点 - 规则链的入口节点
    #[serde(rename = "head")]
    Head,
    /// 中间节点 - 处理业务逻辑的节点
    #[serde(rename = "middle")]
    Middle,
    /// 尾节点 - 规则链的出口节点
    #[serde(rename = "tail")]
    Tail,
}

/// 通用节点配置
#[derive(Debug, Clone, Deserialize)]
pub struct CommonConfig {
    /// 节点类型,默认为中间节点
    #[serde(default = "default_node_type")]
    pub node_type: NodeType,
}

/// 默认节点类型为中间节点
fn default_node_type() -> NodeType {
    NodeType::Middle
}
