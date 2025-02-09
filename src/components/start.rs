use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StartConfig {
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for StartConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                node_type: NodeType::Head,
            },
        }
    }
}

#[derive(Debug)]
pub struct StartNode {
    #[allow(dead_code)]
    config: StartConfig,
}

impl StartNode {
    pub fn new(config: StartConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for StartNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 开始节点只做消息转发
        ctx.send_next(msg.clone()).await?;
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "start".to_string(),
            name: "开始节点".to_string(),
            description: "规则链的起始节点".to_string(),
        }
    }
}
