use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

pub struct TransformNode {
    pub(crate) config: TransformConfig,
}

#[derive(Deserialize)]
pub struct TransformConfig {
    pub template: Value,
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
    async fn handle(&self, _ctx: NodeContext, msg: Message) -> Result<Message, RuleError> {
        let new_data = self.apply_template(&msg)?;

        Ok(Message {
            id: msg.id,
            msg_type: "alert".to_string(),
            metadata: msg.metadata,
            data: new_data,
            timestamp: msg.timestamp,
        })
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "transform".to_string(),
            name: "消息转换器".to_string(),
            description: "转换消息格式".to_string(),
        }
    }
}
