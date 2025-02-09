use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use rquickjs::Context;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Deserialize)]
pub struct JsFunctionConfig {
    pub functions: HashMap<String, String>, // 函数名 -> 函数定义
    pub main: String,                       // 主函数名
    #[serde(default)]
    pub chain_id: String,
    #[serde(default)]
    pub node_id: String,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for JsFunctionConfig {
    fn default() -> Self {
        Self {
            functions: HashMap::new(),
            main: "main".to_string(),
            chain_id: String::new(),
            node_id: String::new(),
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

#[derive(Debug)]
pub struct JsFunctionNode {
    config: JsFunctionConfig,
}

impl JsFunctionNode {
    pub fn new(config: JsFunctionConfig) -> Self {
        Self { config }
    }

    fn replace_function_names(&self, code: &str, config: &JsFunctionConfig) -> String {
        let mut modified_code = code.to_string();
        // 遍历所有已定义的函数名
        for name in config.functions.keys() {
            // 检查是否包含 "name("
            let search_pattern = format!("{}(", name);
            if code.contains(&search_pattern) {
                // 构造完整的函数名
                let full_name = format!("{}_{}_{}", name, config.chain_id, config.node_id);
                // 直接替换函数调用
                modified_code = modified_code.replace(&search_pattern, &format!("{}(", full_name));
            }
        }
        modified_code
    }

    fn register_functions<'js>(&self, ctx: &rquickjs::Ctx<'js>) -> Result<(), RuleError> {
        // 注册所有函数
        let config = self.config.functions.clone();

        // 先处理所有函数的代码
        let mut processed_functions: HashMap<String, String> = HashMap::new();
        for (name, code) in &config {
            let func_name = format!("{}_{}_{}", name, self.config.chain_id, self.config.node_id);
            // 替换函数体中的其他函数调用
            let modified_code = self.replace_function_names(code, &self.config);
            processed_functions.insert(func_name, modified_code);
        }

        // 然后注册处理后的函数
        for (func_name, modified_code) in processed_functions {
            let js_code = format!(
                r#"
                function {}(msg) {{
                    {}
                }}
                "#,
                func_name, modified_code
            );
            // println!("js_code: {}", js_code); // 添加调试日志
            ctx.eval::<(), _>(js_code)
                .map_err(|e| RuleError::NodeExecutionError(format!("函数注册失败: {}", e)))?;
        }
        Ok(())
    }

    fn execute_js<'js>(&self, ctx: &rquickjs::Ctx<'js>, msg: &Message) -> Result<Value, RuleError> {
        // 注册函数
        self.register_functions(ctx)?;

        // 注入消息对象
        let main_name = format!(
            "{}_{}_{}",
            self.config.main, self.config.chain_id, self.config.node_id
        );
        let msg_json = serde_json::to_string(&msg).unwrap();
        ctx.eval::<(), _>(format!("const msg = {};", msg_json))
            .map_err(|e| RuleError::NodeExecutionError(e.to_string()))?;

        // 调用主函数并获取结果
        let result = ctx
            .eval::<String, _>(format!("JSON.stringify({}(msg));", main_name))
            .map_err(|e| RuleError::NodeExecutionError(format!("函数执行失败: {}", e)))?;
        // 解析JSON结果
        serde_json::from_str(&result)
            .map_err(|e| RuleError::NodeExecutionError(format!("结果解析失败: {}", e)))
    }
}

#[async_trait]
impl NodeHandler for JsFunctionNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 创建新的运行时
        let runtime = rquickjs::Runtime::new().unwrap();
        let ctx_js = Context::full(&runtime).unwrap();
        let result = ctx_js.with(|ctx| self.execute_js(&ctx, &msg))?;

        // 构造返回消息
        let new_msg = Message {
            id: msg.id,
            msg_type: "js_function_result".to_string(),
            metadata: msg.metadata,
            data: result,
            timestamp: msg.timestamp,
        };

        // 发送到下一个节点
        ctx.send_next(new_msg.clone()).await?;

        Ok(new_msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "js_function".to_string(),
            name: "JS函数节点".to_string(),
            description: "执行自定义JS函数".to_string(),
        }
    }
}
