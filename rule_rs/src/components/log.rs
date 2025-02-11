use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use tracing::info;

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

    fn format_message(&self, msg: &Message) -> String {
        let mut result = self.config.template.clone();
        // 替换消息相关的变量
        result = result.replace("${msg.id}", &msg.id.to_string());
        result = result.replace("${msg.type}", &msg.msg_type);
        // 替换数据中的变量
        if let Some(obj) = msg.data.as_object() {
            for (key, value) in obj {
                let placeholder = format!("${{msg.data.{}}}", key);
                let placeholder_without_markers = format!("msg.data.{}", key);
                let result_without_markers = result.replace("${", "").replace("}", ""); // 去掉结果中的 ${}

                if result_without_markers.contains(&placeholder_without_markers) {
                    let value_str = if value.is_object() || value.is_array() {
                        // 检查是否有更深层的路径
                        let parts: Vec<&str> = key.split('.').collect();
                        if parts.len() > 1 {
                            // 递归查找嵌套值
                            let mut current_value = value;
                            for &part in &parts[1..] {
                                if let Some(obj) = current_value.as_object() {
                                    current_value = obj.get(part).unwrap_or(value);
                                }
                            }
                            current_value.to_string()
                        } else {
                            value.to_string()
                        }
                    } else {
                        value.to_string()
                    };
                    result = result.replace(&placeholder, &value_str);
                }
            }
        } else if let Some(value) = msg.data.as_str() {
            result = result.replace("${msg.data}", value);
        } else if let Some(value) = msg.data.as_f64() {
            result = result.replace("${msg.data}", &value.to_string());
        } else {
            result = result.replace("${msg.data}", &msg.data.to_string());
        }
        result
    }
}

#[async_trait]
impl NodeHandler for LogNode {
    async fn handle<'a>(
        &'a self,
        _ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 格式化并输出日志
        let log_message = self.format_message(&msg);
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
