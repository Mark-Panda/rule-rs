use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

#[derive(Debug, Deserialize)]
pub struct JoinConfig {
    #[serde(default)]
    pub timeout: u64, // 超时时间(秒)
    pub success_branch: Option<String>, // 成功分支名称
    pub error_branch: Option<String>,   // 失败分支名称
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for JoinConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
            timeout: 30,
            success_branch: None,
            error_branch: None,
        }
    }
}

#[derive(Debug)]
pub struct JoinNode {
    config: JoinConfig,
}

#[derive(Debug, Serialize)]
struct WrapperMsg {
    msg: Message,
    err: Option<String>,
    node_id: String,
}

impl JoinNode {
    pub fn new(config: JoinConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for JoinNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 检查是否是分支消息
        if !msg.metadata.contains_key("is_branch") {
            return Ok(msg);
        }
        let mut msg = msg;

        // 获取所有前置节点的连接数量
        let expected_branches = {
            let chain = ctx
                .engine
                .get_chain(ctx.node.chain_id)
                .await
                .ok_or_else(|| RuleError::ChainNotFound(ctx.node.chain_id))?;

            let incoming_count = chain
                .connections
                .iter()
                .filter(|conn| conn.to_id == ctx.node.id)
                .count();

            println!("Join节点 {} 的前置连接数: {}", ctx.node.id, incoming_count);
            incoming_count
        };
        println!("expected_branches: {}", expected_branches);

        // 获取所有前置节点的结果
        let mut results = vec![];

        // 设置超时
        let timeout_duration = if self.config.timeout > 0 {
            Duration::from_secs(self.config.timeout)
        } else {
            Duration::from_secs(30) // 默认30秒
        };

        // 等待所有分支执行完成
        let mut remaining_branches = expected_branches;
        match timeout(timeout_duration, async {
            while remaining_branches > 0 {
                let branch_msg = msg.clone();
                results.push(WrapperMsg {
                    msg: branch_msg.clone(),
                    err: None,
                    node_id: ctx.node.id.to_string(),
                });
                remaining_branches -= 1;
            }
            Ok::<(), RuleError>(())
        })
        .await
        {
            Ok(_) => {
                // 所有分支都收集完成
                let mut result_msg = msg.clone();

                let merged_data = serde_json::json!({
                    "results": results,
                });
                result_msg.data = merged_data;
                // 设置成功分支
                if let Some(branch) = &self.config.success_branch {
                    result_msg
                        .metadata
                        .insert("branch_name".into(), branch.clone());
                }
                ctx.send_next(result_msg.clone()).await?;
                Ok(result_msg)
            }
            Err(_) => {
                // 设置失败分支
                if let Some(branch) = &self.config.error_branch {
                    msg.metadata.insert("branch_name".into(), branch.clone());
                }
                // 超时处理
                ctx.send_next(msg.clone()).await?;
                Ok(msg)
            }
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "join".to_string(),
            name: "汇聚节点".to_string(),
            description: "汇聚并合并多个并行分支的执行结果".to_string(),
        }
    }
}
