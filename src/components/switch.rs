use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use rquickjs::{Context, Runtime};
use serde::{Deserialize, Serialize};

// 分支条件配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwitchCase {
    pub name: String,        // 分支名称
    pub condition: String,   // JS条件表达式
    pub description: String, // 分支描述
}

#[derive(Debug, Deserialize)]
pub struct SwitchConfig {
    pub cases: Vec<SwitchCase>,
    pub default_next: Option<String>,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for SwitchConfig {
    fn default() -> Self {
        Self {
            cases: Vec::new(),
            default_next: None,
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

pub struct SwitchNode {
    config: SwitchConfig,
}

impl SwitchNode {
    pub fn new(config: SwitchConfig) -> Self {
        Self { config }
    }

    // 执行条件表达式
    fn evaluate_condition(&self, case: &SwitchCase, msg: &Message) -> Result<bool, RuleError> {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();

        ctx.with(|ctx| {
            // 注入消息数据
            let msg_json = serde_json::to_string(&msg).unwrap();
            let js_code = format!(
                r#"
                const msg = {};
                const condition = () => {{ 
                    return {}; 
                }};
                condition();
                "#,
                msg_json, case.condition
            );

            // 执行条件表达式
            let result: bool = ctx
                .eval(js_code)
                .map_err(|e: rquickjs::Error| RuleError::NodeExecutionError(e.to_string()))?;

            Ok(result)
        })
    }
}

#[async_trait]
impl NodeHandler for SwitchNode {
    async fn handle<'a>(
        &self,
        ctx: NodeContext<'a>,
        mut msg: Message,
    ) -> Result<Message, RuleError> {
        // 遍历所有分支条件
        for case in &self.config.cases {
            if self.evaluate_condition(case, &msg)? {
                msg.metadata.insert("branch_name".into(), case.name.clone());
                // 发送到对应分支的下一个节点
                ctx.send_next(msg.clone()).await?;
                return Ok(msg);
            }
        }

        // 没有匹配的条件,使用默认分支
        if let Some(default) = &self.config.default_next {
            msg.metadata.insert("branch_name".into(), default.clone());
            ctx.send_next(msg.clone()).await?;
        }
        
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "switch".to_string(),
            name: "条件分支节点".to_string(),
            description: "根据条件选择不同的处理分支".to_string(),
        }
    }
}
