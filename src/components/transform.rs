use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

#[derive(Debug)]
pub struct TransformNode {
    pub(crate) config: TransformConfig,
}

#[derive(Debug, Deserialize)]
pub struct TransformConfig {
    pub template: Value,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            template: json!({}),
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

impl TransformNode {
    pub fn new(config: TransformConfig) -> Self {
        Self { config }
    }

    fn get_value_by_path(data: &Value, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = data;

        for part in parts {
            match current.get(part) {
                Some(value) => current = value,
                None => return None,
            }
        }

        match current {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => Some(current.to_string()),
        }
    }

    fn apply_template(&self, msg: &Message) -> Result<Value, RuleError> {
        let mut result = self.config.template.clone();

        if let Value::Object(obj) = &mut result {
            for (_key, value) in obj.iter_mut() {
                if let Value::String(template) = value {
                    let mut new_value = template.clone();

                    // 查找所有 ${...} 模板变量
                    while let Some(start) = new_value.find("${") {
                        if let Some(end) = new_value[start..].find('}') {
                            let var_path = &new_value[start + 2..start + end];
                            let replacement = if var_path.starts_with("msg.") {
                                // 处理消息数据中的变量
                                Self::get_value_by_path(&msg.data, &var_path[4..])
                                    .unwrap_or_else(|| "".to_string())
                            } else {
                                // 处理其他类型的变量
                                match var_path {
                                    "msg.type" => msg.msg_type.clone(),
                                    "msg.id" => msg.id.to_string(),
                                    _ => "".to_string(),
                                }
                            };

                            new_value.replace_range(start..start + end + 1, &replacement);
                        } else {
                            break;
                        }
                    }

                    *value = Value::String(new_value);
                }
            }
        }

        Ok(result)
    }
}

#[async_trait]
impl NodeHandler for TransformNode {
    async fn handle<'a>(&'a self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        // 执行转换
        let new_data = self.apply_template(&msg)?;
        let transformed_msg = Message {
            id: msg.id,
            msg_type: msg.msg_type,
            metadata: msg.metadata,
            data: new_data,
            timestamp: msg.timestamp,
        };

        // 发送到下一个节点
        ctx.send_next(transformed_msg.clone()).await?;
        
        Ok(transformed_msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "transform".to_string(),
            name: "消息转换器".to_string(),
            description: "转换消息格式".to_string(),
        }
    }
}
