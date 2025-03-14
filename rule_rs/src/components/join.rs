use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

lazy_static! {
    static ref GLOBAL_JOIN_STATE: Arc<Mutex<HashMap<String, Vec<Message>>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[derive(Debug, Deserialize)]
pub struct JoinConfig {}

impl Default for JoinConfig {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug)]
pub struct JoinNode {
    #[allow(dead_code)]
    config: JoinConfig,
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
        let chain = ctx
            .engine
            .get_chain(ctx.node.chain_id)
            .await
            .ok_or(RuleError::ChainNotFound(ctx.node.chain_id))?;

        let expected_branches = chain
            .connections
            .iter()
            .filter(|conn| conn.to_id == ctx.node.id)
            .count();

        // 使用全局状态存储
        let mut global_state = GLOBAL_JOIN_STATE.lock().await;
        let messages = global_state
            .entry(msg.id.to_string())
            .or_insert_with(Vec::new);

        messages.push(msg.clone());
        if messages.len() >= expected_branches {
            let branch_messages = messages.drain(..).collect::<Vec<_>>();
            drop(global_state);

            let result_msg = Message {
                id: msg.id,
                msg_type: msg.msg_type,
                metadata: msg.metadata.clone(),
                data: json!({
                    "branches": branch_messages.iter().map(|msg| json!({
                        "data": msg.data,
                    })).collect::<Vec<_>>()
                }),
                timestamp: msg.timestamp,
            };

            debug!(
                "Join节点 {} 合并完成，发送结果消息: {:?}",
                ctx.node.id, result_msg
            );
            ctx.send_next(result_msg.clone()).await?;
            Ok(result_msg)
        } else {
            debug!(
                "Join节点 {} 等待更多分支消息 ({}/{})",
                ctx.node.id,
                messages.len(),
                expected_branches
            );
            Ok(msg)
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "join".to_string(),
            name: "汇聚节点".to_string(),
            description: "汇聚并合并多个并行分支的执行结果".to_string(),
            node_type: NodeType::Middle,
        }
    }
}
