use async_trait::async_trait;
use rule_rs::engine::NodeHandler;
use rule_rs::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use rule_rs::{engine::rule::RuleEngineTrait, RuleEngine};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "自定义组件示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "type_name": "start",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {},
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "custom/upper",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {},
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "转换结果: ${msg.data}"
            },
            "layout": { "x": 500, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "success"
        }
    ],
    "metadata": {
        "version": 1,
        "created_at": 1679800000,
        "updated_at": 1679800000
    }
}"#;

#[derive(Debug, Deserialize)]
pub struct UpperConfig {}

impl Default for UpperConfig {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct UpperNode {
    #[allow(dead_code)]
    config: UpperConfig,
}

impl UpperNode {
    pub fn new(config: UpperConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for UpperNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
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
            node_type: NodeType::Middle,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = RuleEngine::new().await;

    // 注册自定义组件
    engine
        .register_node_type(
            "custom/upper",
            Arc::new(|config| {
                if config.is_object() && config.as_object().unwrap().is_empty() {
                    Ok(Arc::new(UpperNode::new(UpperConfig::default())) as Arc<dyn NodeHandler>)
                } else {
                    let config: UpperConfig = serde_json::from_value(config)?;
                    Ok(Arc::new(UpperNode::new(config)) as Arc<dyn NodeHandler>)
                }
            }),
        )
        .await;
    info!("已注册的组件:");
    for desc in engine.get_registered_components().await {
        info!("- {}: {}", desc.type_name, desc.description);
    }

    let chain_id = engine.load_chain(RULE_CHAIN).await?;

    let msg = Message::new("test", json!("hello world"));

    match engine.process_msg(chain_id, msg).await {
        Ok(_) => info!("处理成功"),
        Err(e) => info!("处理失败: {:?}", e),
    }

    Ok(())
}
