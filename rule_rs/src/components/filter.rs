use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::Deserialize;

#[derive(Debug)]
pub struct FilterNode {
    pub(crate) config: FilterConfig,
}

#[derive(Debug, Deserialize)]
pub struct FilterConfig {
    pub condition: String,
    pub js_script: Option<String>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            condition: "true".to_string(),
            js_script: None,
        }
    }
}

impl FilterNode {
    pub fn new(config: FilterConfig) -> Self {
        Self { config }
    }

    fn eval_condition(&self, _ctx: &NodeContext, msg: &Message) -> Result<bool, RuleError> {
        if let Some(value) = msg.data.get("value") {
            if let Some(num) = value.as_f64() {
                match self.config.condition.as_str() {
                    "value < 10" => Ok(num < 10.0),
                    _ => Ok(true),
                }
            } else {
                Err(RuleError::NodeExecutionError(
                    "Value must be a number".to_string(),
                ))
            }
        } else {
            Err(RuleError::NodeExecutionError(
                "Missing value field".to_string(),
            ))
        }
    }
}

#[async_trait]
impl NodeHandler for FilterNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        if self.eval_condition(&ctx, &msg)? {
            // 条件满足,发送到下一个节点
            ctx.send_next(msg.clone()).await?;
            Ok(msg)
        } else {
            Err(RuleError::FilterReject)
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "filter".to_string(),
            name: "消息过滤器".to_string(),
            description: "根据条件过滤消息".to_string(),
            node_type: NodeType::Middle,
        }
    }
}
