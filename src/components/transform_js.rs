use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use rquickjs::{Context, Runtime};
use serde::Deserialize;
use serde_json::Value;

pub struct TransformJsNode {
    pub(crate) config: TransformJsConfig,
}

#[derive(Debug, Deserialize)]
pub struct TransformJsConfig {
    pub script: String,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for TransformJsConfig {
    fn default() -> Self {
        Self {
            script: "return msg;".to_string(),
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

impl TransformJsNode {
    pub fn new(config: TransformJsConfig) -> Self {
        Self { config }
    }

    fn execute_js(&self, msg: &Message) -> Result<Value, RuleError> {
        // 创建 JS 运行时和上下文
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        // 在 JS 上下文中执行代码
        ctx.with(|ctx| {
            // 将消息数据注入到 JS 上下文
            let msg_data = serde_json::to_string(&msg.data).unwrap();
            let js_code = format!(
                r#"
                const msg = {};
                const transform = (msg) => {{
                    {}
                }};
                JSON.stringify(transform(msg));
                "#,
                msg_data, self.config.script
            );
            // 执行转换脚本
            let result: String = ctx
                .eval(js_code)
                .map_err(|e| RuleError::NodeExecutionError(e.to_string()))?;
            // 解析结果
            serde_json::from_str(&result).map_err(|e| RuleError::NodeExecutionError(e.to_string()))
        })
        .map_err(|e| RuleError::NodeExecutionError(e.to_string()))
    }
}

#[async_trait]
impl NodeHandler for TransformJsNode {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        let new_data = self.execute_js(&msg)?;
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
            type_name: "transform_js".to_string(),
            name: "JS转换器".to_string(),
            description: "使用JavaScript转换消息".to_string(),
        }
    }
}
