use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::info;
use crate::utils::resolve_placeholders_in_str;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub template: String,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            template: String::new(),
        }
    }
}

#[derive(Debug)]
pub struct LogNode {
    config: LogConfig,
}

impl LogNode {
    pub fn new(config: LogConfig) -> Self {
        Self { config }
    }

    fn format_message(&self, ctx: &NodeContext, msg: &Message) -> String {
        resolve_placeholders_in_str(&self.config.template, ctx, msg)
    }
}

#[async_trait]
impl NodeHandler for LogNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 格式化并输出日志
        let log_message = self.format_message(&ctx, &msg);
        info!("log组件输出: {}", log_message);
        // 返回原始消息
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "log".to_string(),
            name: "日志节点".to_string(),
            description: "输出日志消息".to_string(),
            node_type: NodeType::Tail,
        }
    }
}
