use async_trait::async_trait;
use rulego_rs::engine::NodeHandler;
use rulego_rs::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpperConfig {
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for UpperConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

pub struct UpperNode {
    config: UpperConfig,
}

impl UpperNode {
    pub fn new(config: UpperConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for UpperNode {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        // 将消息内容转换为大写
        if let Some(text) = msg.data.as_str() {
            let upper_text = text.to_uppercase();
            let mut new_msg = msg.clone();
            new_msg.data = serde_json::Value::String(upper_text);
            ctx.send_next(new_msg).await?;
        }
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "custom/upper".to_string(),
            name: "大写转换节点".to_string(),
            description: "将文本转换为大写".to_string(),
        }
    }
}
