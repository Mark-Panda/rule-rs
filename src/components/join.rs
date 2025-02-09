use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::timeout;

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinConfig {
    #[serde(default)]
    pub timeout: u64, // 超时时间(秒)
}

#[derive(Debug)]
pub struct JoinNode {
    config: JoinConfig,
    processed_msgs: Arc<Mutex<HashSet<String>>>,
    max_iterations: u32,
}

#[derive(Debug, Serialize)]
struct WrapperMsg {
    msg: Message,
    err: Option<String>,
    node_id: String,
}

impl JoinNode {
    pub fn new(config: JoinConfig) -> Self {
        Self {
            config,
            processed_msgs: Arc::new(Mutex::new(HashSet::new())),
            max_iterations: 100, // 设置最大迭代次数
        }
    }

    async fn process_message(&self, msg: Message) -> Result<Message, RuleError> {
        let msg_id = msg.id.to_string();
        let mut processed = self.processed_msgs.lock().await;

        // 检查消息是否已处理过
        if processed.contains(&msg_id) {
            return Ok(msg);
        }

        // 添加到已处理集合
        processed.insert(msg_id);

        // 检查迭代次数
        if processed.len() > self.max_iterations as usize {
            return Err(RuleError::NodeExecutionError(
                "超过最大迭代次数限制".to_string(),
            ));
        }

        Ok(msg)
    }
}

#[async_trait]
impl NodeHandler for JoinNode {
    async fn handle<'a>(
        &'a self,
        mut ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 检查是否是分支消息
        if !msg.metadata.contains_key("is_branch") {
            return Ok(msg);
        }

        // 获取所有前置节点的结果
        let mut results = vec![];
        let mut has_error = false;

        // 设置超时
        let timeout_duration = if self.config.timeout > 0 {
            Duration::from_secs(self.config.timeout)
        } else {
            Duration::from_secs(30) // 默认30秒
        };

        // 等待所有分支执行完成
        match timeout(timeout_duration, async {
            for result in ctx.get_branch_results().await {
                match result {
                    Ok(branch_msg) => {
                        // 只处理分支消息
                        if branch_msg.metadata.contains_key("is_branch") {
                            results.push(WrapperMsg {
                                msg: branch_msg.clone(),
                                err: None,
                                node_id: ctx.node.id.to_string(),
                            });
                        }
                    }
                    Err(e) => {
                        has_error = true;
                        results.push(WrapperMsg {
                            msg: msg.clone(),
                            err: Some(e.to_string()),
                            node_id: ctx.node.id.to_string(),
                        });
                    }
                }
            }
        })
        .await
        {
            Ok(_) => {
                // 移除分支标记
                let mut result_msg = msg.clone();
                result_msg.metadata.remove("is_branch");
                result_msg.metadata.remove("branch_id");

                // 合并结果
                let merged_data = serde_json::json!({
                    "results": results,
                });
                result_msg.data = merged_data;

                if has_error {
                    ctx.set_next_branch("Failure");
                }

                Ok(result_msg)
            }
            Err(_) => {
                ctx.set_next_branch("Failure");
                Err(RuleError::NodeExecutionError(
                    "Join节点执行超时".to_string(),
                ))
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
