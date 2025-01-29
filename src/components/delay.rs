use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use cron::Schedule;
use serde::Deserialize;
use std::str::FromStr;
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

    /// Cron表达式
    pub cron: Option<String>,

    /// 时区偏移(小时)
    pub timezone_offset: i32,

    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self {
            delay_ms: 0,
            periodic: false,
            period_count: 0,
            cron: None,
            timezone_offset: 0,
            common: CommonConfig {
                node_type: NodeType::Head,
            },
        }
    }
}

/// 延迟处理节点
pub struct DelayNode {
    config: DelayConfig,
    schedule: Option<Schedule>,
}

impl DelayNode {
    pub fn new(config: DelayConfig) -> Result<Self, RuleError> {
        let schedule =
            if let Some(cron) = &config.cron {
                Some(Schedule::from_str(cron).map_err(|e| {
                    RuleError::ConfigError(format!("Invalid cron expression: {}", e))
                })?)
            } else {
                None
            };

        Ok(Self { config, schedule })
    }

    /// 获取下一次执行时间
    fn next_schedule_time(&self) -> Option<DateTime<Local>> {
        self.schedule.as_ref().and_then(|schedule| {
            let now = Utc::now();
            schedule
                .after(&now)
                .next()
                .map(|utc| utc.with_timezone(&Local))
        })
    }
}

#[async_trait]
impl NodeHandler for DelayNode {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        if let Some(_schedule) = &self.schedule {
            // 处理 cron 定时执行
            loop {
                if let Some(next_time) = self.next_schedule_time() {
                    let now = Local::now();
                    let delay = next_time.signed_duration_since(now);

                    // 等待到执行时间
                    if delay.num_milliseconds() > 0 {
                        sleep(Duration::from_millis(delay.num_milliseconds() as u64)).await;
                    }

                    // 发送消息副本
                    let msg_clone = msg.clone();
                    ctx.send_next(msg_clone).await?;
                }

                if !self.config.periodic {
                    break;
                }
            }
            Ok(msg)
        } else if self.config.periodic {
            // 周期性延迟处理
            let mut count = 0;
            loop {
                sleep(Duration::from_millis(self.config.delay_ms)).await;

                let msg_clone = msg.clone();
                ctx.send_next(msg_clone).await?;

                count += 1;
                if self.config.period_count > 0 && count >= self.config.period_count {
                    break;
                }
            }
            Ok(msg)
        } else {
            // 一次性延迟
            sleep(Duration::from_millis(self.config.delay_ms)).await;
            Ok(msg)
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "delay".to_string(),
            name: "延迟节点".to_string(),
            description: "延迟处理消息,支持一次性延迟、周期性延迟和Cron定时执行".to_string(),
        }
    }
}
