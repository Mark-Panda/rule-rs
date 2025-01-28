use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::Deserialize;

pub struct SwitchNode {
    pub(crate) config: SwitchConfig,
}

#[derive(Deserialize)]
pub struct SwitchCase {
    pub condition: String,
    pub output: String,
}

#[derive(Deserialize)]
pub struct SwitchConfig {
    pub cases: Vec<SwitchCase>,
    pub default: Option<String>,
}

impl SwitchNode {
    pub fn new(config: SwitchConfig) -> Self {
        Self { config }
    }

    fn evaluate_case(&self, case: &SwitchCase, msg: &Message) -> Result<bool, RuleError> {
        // 简单实现，实际应该使用表达式引擎
        match case.condition.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => {
                if let Some(value) = msg.data.get("value") {
                    if let Some(num) = value.as_f64() {
                        let expr = case.condition.replace("value", &num.to_string());
                        // 这里应该使用表达式引擎进行求值
                        Ok(expr.contains(">") || expr.contains("<"))
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }
}

#[async_trait]
impl NodeHandler for SwitchNode {
    async fn handle(&self, _ctx: NodeContext, msg: Message) -> Result<Message, RuleError> {
        // 遍历所有 case
        for case in &self.config.cases {
            if self.evaluate_case(case, &msg)? {
                return Ok(Message {
                    id: msg.id,
                    msg_type: case.output.clone(),
                    metadata: msg.metadata,
                    data: msg.data,
                    timestamp: msg.timestamp,
                });
            }
        }

        // 如果没有匹配的 case，使用默认输出
        if let Some(default_output) = &self.config.default {
            Ok(Message {
                id: msg.id,
                msg_type: default_output.clone(),
                metadata: msg.metadata,
                data: msg.data,
                timestamp: msg.timestamp,
            })
        } else {
            Err(RuleError::NodeExecutionError(
                "No matching case".to_string(),
            ))
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "switch".to_string(),
            name: "分支节点".to_string(),
            description: "根据条件选择不同的处理分支".to_string(),
        }
    }
}
