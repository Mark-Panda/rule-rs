use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use rquickjs::{Context, Function, Runtime};
use serde::Deserialize;
use serde_json::Value;

pub struct ScriptNode {
    pub(crate) config: ScriptConfig,
}

#[derive(Deserialize)]
pub struct ScriptConfig {
    pub script: String,
    pub output_type: Option<String>,
}

impl ScriptNode {
    pub fn new(config: ScriptConfig) -> Self {
        Self { config }
    }

    fn execute_script(&self, node_ctx: &NodeContext, msg: &Message) -> Result<Value, RuleError> {
        let rt = Runtime::new().unwrap();
        let js_ctx = Context::full(&rt).unwrap();

        js_ctx
            .with(|ctx| {
                // 注入上下文变量
                let msg_json = serde_json::to_string(&msg).unwrap();
                // 创建简化的上下文对象
                let ctx_obj = serde_json::json!({
                    "node_id": node_ctx.node.id.to_string(),
                    "node_type": node_ctx.node.type_name,
                    "metadata": node_ctx.metadata
                });

                // 添加 console.log 支持
                {
                    let console = rquickjs::Object::new(ctx.clone()).unwrap();
                    let log_fn =
                        Function::new(ctx.clone(), |s: String| println!("JS log: {}", s)).unwrap();
                    let error_fn =
                        Function::new(ctx.clone(), |s: String| eprintln!("JS error: {}", s))
                            .unwrap();
                    console.set("log", log_fn).unwrap();
                    console.set("error", error_fn).unwrap();
                    ctx.globals().set("console", console).unwrap();
                }

                let js_code = format!(
                    r#"
                const msg = {};
                const ctx = {};
                const execute = () => {{
                    {}
                }};
                JSON.stringify(execute());
                "#,
                    msg_json, ctx_obj, self.config.script
                );
                println!("js code: {}", js_code);
                let result: String = ctx.eval(js_code).map_err(|e| {
                    RuleError::NodeExecutionError(format!("JavaScript执行错误: {}", e))
                })?;
                println!("script result: {}", result);
                serde_json::from_str(&result)
                    .map_err(|e| RuleError::NodeExecutionError(format!("JSON解析错误: {}", e)))
            })
            .map_err(|e| RuleError::NodeExecutionError(format!("节点执行失败: {}", e)))
    }
}

#[async_trait]
impl NodeHandler for ScriptNode {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        let new_data = self.execute_script(&ctx, &msg)?;
        Ok(Message {
            id: msg.id,
            msg_type: self.config.output_type.clone().unwrap_or(msg.msg_type),
            metadata: msg.metadata,
            data: new_data,
            timestamp: msg.timestamp,
        })
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "script".to_string(),
            name: "脚本节点".to_string(),
            description: "执行自定义脚本".to_string(),
        }
    }
}
