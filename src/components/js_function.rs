use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use rquickjs::{Context, Runtime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct JsFunctionConfig {
    pub functions: HashMap<String, String>, // 函数名 -> 函数定义
    pub main: String,                       // 主函数名
    #[serde(default)]
    pub chain_id: String,
    #[serde(default)]
    pub node_id: String,
}

impl Default for JsFunctionConfig {
    fn default() -> Self {
        Self {
            functions: HashMap::new(),
            main: "main".to_string(),
            chain_id: String::new(),
            node_id: String::new(),
        }
    }
}

pub struct JsFunctionNode {
    config: RwLock<JsFunctionConfig>,
    runtime: Arc<Mutex<Runtime>>,
}

impl JsFunctionNode {
    pub fn new(config: JsFunctionConfig) -> Self {
        Self {
            config: RwLock::new(config),
            runtime: Arc::new(Mutex::new(Runtime::new().unwrap())),
        }
    }

    fn register_functions<'js>(&self, ctx: &rquickjs::Ctx<'js>) -> Result<(), RuleError> {
        // 注册所有函数
        let config = self.config.read().unwrap();
        for (name, code) in &config.functions {
            let func_name = format!("{}_{}_{}", name, config.chain_id, config.node_id);
            let js_code = format!(
                r#"
                function {}(msg) {{
                    {}
                }}
                "#,
                func_name, code
            );
            ctx.eval::<(), _>(js_code)
                .map_err(|e| RuleError::NodeExecutionError(format!("函数注册失败: {}", e)))?;
        }
        Ok(())
    }

    fn execute_js<'js>(&self, ctx: &rquickjs::Ctx<'js>, msg: &Message) -> Result<Value, RuleError> {
        // 注册函数
        self.register_functions(ctx)?;

        // 注入消息对象
        let config = self.config.read().unwrap();
        let main_name = format!("{}_{}_{}", config.main, config.chain_id, config.node_id);
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
        &self,
        node_ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 获取当前节点和规则链信息
        let node = node_ctx.node;
        let chain_id = node.chain_id.to_string().replace("-", "_");
        let node_id = node.id.to_string().replace("-", "_");

        // 更新配置并立即释放锁
        {
            let mut config = self.config.write().unwrap();
            config.chain_id = chain_id;
            config.node_id = node_id;
        } // 锁在这里被释放

        // 获取运行时的互斥锁
        let runtime = self.runtime.lock().await;
        let ctx = Context::full(&runtime).unwrap();
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
