use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

/// 延迟节点配置
#[derive(Debug, Deserialize)]
pub struct DelayConfig {
    /// 延迟时间(毫秒)
    pub delay_ms: u64,

    /// 是否周期性延迟
    pub periodic: bool,

    /// 周期执行次数,0表示无限循环
    pub period_count: u32,
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            delay_ms: 1000,
            periodic: false,
            period_count: 0,
        }
    }
}

/// 延迟处理节点
#[derive(Debug)]
pub struct DelayNode {
    config: DelayConfig,
}

impl DelayNode {
    pub fn new(config: DelayConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for DelayNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        if self.config.periodic {
            let mut count = 0;
            loop {
                sleep(Duration::from_millis(self.config.delay_ms)).await;
                // 发送到下一个节点
                ctx.send_next(msg.clone()).await?;

                count += 1;
                if self.config.period_count > 0 && count >= self.config.period_count {
                    break;
                }
            }
        } else {
            sleep(Duration::from_millis(self.config.delay_ms)).await;
            // 发送到下一个节点
            ctx.send_next(msg.clone()).await?;
        }
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "delay".to_string(),
            name: "延时节点".to_string(),
            description: "延迟处理消息,支持一次性延迟和周期性延迟".to_string(),
            node_type: NodeType::Head,
        }
    }
}
