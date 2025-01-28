use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub template: String,
}

pub struct LogNode {
    config: LogConfig,
}

impl LogNode {
    pub fn new(config: LogConfig) -> Self {
        Self { config }
    }

    fn format_message(&self, msg: &Message) -> String {
        let mut result = self.config.template.clone();

        // 替换消息相关的变量
        result = result.replace("${msg.id}", &msg.id.to_string());
        result = result.replace("${msg.type}", &msg.msg_type);

        // 替换数据中的变量
        if let Some(obj) = msg.data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("${{msg.{}}}", key);
                if result.contains(&placeholder) {
                    result = result.replace(&placeholder, &value.to_string());
                }
            }
        }

        result
    }
}

#[async_trait]
impl NodeHandler for LogNode {
    async fn handle<'a>(&self, _ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        // 格式化并输出日志
        let log_message = self.format_message(&msg);
        info!("{}", log_message);

        // 返回原始消息
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "log".to_string(),
            name: "日志节点".to_string(),
            description: "输出格式化日志".to_string(),
        }
    }
}
