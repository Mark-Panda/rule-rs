use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use rquickjs::{Context, Runtime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct JsFunctionConfig {
    pub functions: HashMap<String, String>, // 函数名 -> 函数定义
    pub main: String,                       // 主函数名
}

pub struct JsFunctionNode {
    config: JsFunctionConfig,
    runtime: Runtime,
}

impl JsFunctionNode {
    pub fn new(config: JsFunctionConfig) -> Self {
        Self {
            config,
            runtime: Runtime::new().unwrap(),
        }
    }

    fn register_functions<'js>(&self, ctx: &rquickjs::Ctx<'js>) -> Result<(), RuleError> {
        // 注册所有函数
        for (name, code) in &self.config.functions {
            let js_code = format!(
                r#"
                function {}(msg) {{
                    {}
                }}
                "#,
                name, code
            );
            println!("js_code {}", js_code);
            ctx.eval::<(), _>(js_code)
                .map_err(|e| RuleError::NodeExecutionError(format!("函数注册失败: {}", e)))?;
        }
        Ok(())
    }

    fn execute_js<'js>(&self, ctx: &rquickjs::Ctx<'js>, msg: &Message) -> Result<Value, RuleError> {
        // 注册函数
        self.register_functions(ctx)?;

        // 注入消息对象
        let msg_json = serde_json::to_string(&msg).unwrap();
        ctx.eval::<(), _>(format!("const msg = {};", msg_json))
            .map_err(|e| RuleError::NodeExecutionError(e.to_string()))?;

        // 调用主函数并获取结果
        let result = ctx
            .eval::<String, _>(format!("JSON.stringify({}(msg));", self.config.main))
            .map_err(|e| RuleError::NodeExecutionError(format!("函数执行失败: {}", e)))?;

        // 解析JSON结果
        serde_json::from_str(&result)
            .map_err(|e| RuleError::NodeExecutionError(format!("结果解析失败: {}", e)))
    }
}

#[async_trait]
impl NodeHandler for JsFunctionNode {
    async fn handle<'a>(&self, _ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        let ctx = Context::full(&self.runtime).unwrap();

        let result = ctx.with(|ctx| self.execute_js(&ctx, &msg))?;

        // 构造返回消息
        Ok(Message {
            id: msg.id,
            msg_type: "js_function_result".to_string(),
            metadata: msg.metadata,
            data: result,
            timestamp: msg.timestamp,
        })
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "js_function".to_string(),
            name: "JS函数节点".to_string(),
            description: "执行自定义JS函数".to_string(),
        }
    }
}
