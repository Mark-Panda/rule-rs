use crate::engine::NodeHandler;
use crate::types::{ExecutionContext, Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;

#[derive(Debug)]
pub struct ForkNode;

impl ForkNode {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl NodeHandler for ForkNode {
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

        let connections = chain
            .connections
            .iter()
            .filter(|conn| conn.from_id == ctx.node.id)
            .collect::<Vec<_>>();

        // 创建分支消息
        let mut branch_msgs = Vec::with_capacity(connections.len());
        for (i, _) in connections.iter().enumerate() {
            let branch_id = i.to_string();
            // 保存分支消息
            ctx.add_branch_result(branch_id, msg.clone()).await;
            branch_msgs.push(msg.clone());
        }

        // 并行发送消息到所有分支
        let mut handles = vec![];
        for (conn, branch_msg) in connections.into_iter().zip(branch_msgs) {
            let engine = ctx.engine.clone();
            let chain_id = ctx.node.chain_id;
            let to_id = conn.to_id;

            let handle = tokio::spawn(async move {
                if let Some(chain) = engine.get_chain(chain_id).await {
                    if let Some(target_node) = chain.nodes.iter().find(|n| n.id == to_id) {
                        let ctx = NodeContext::new(
                            target_node,
                            &ExecutionContext::new(branch_msg.clone()),
                            engine.clone(),
                        );
                        engine.execute_node(target_node, &ctx, branch_msg).await
                    } else {
                        Err(RuleError::ConfigError(format!("节点 {} 不存在", to_id)))
                    }
                } else {
                    Err(RuleError::ChainNotFound(chain_id))
                }
            });
            handles.push(handle);
        }

        // 等待所有分支执行完成
        for handle in handles {
            match handle.await {
                Ok(result) => result?,
                Err(e) => return Err(RuleError::NodeExecutionError(e.to_string())),
            };
        }

        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "fork".to_string(),
            name: "并行网关".to_string(),
            description: "将消息并行发送到多个分支进行处理".to_string(),
        }
    }
}
