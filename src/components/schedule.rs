use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use chrono::{DateTime, Local, Utc};
use cron::Schedule;
use serde::Deserialize;
use std::str::FromStr;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
pub struct ScheduleConfig {
    /// Cron表达式
    pub cron: String,
    /// 时区偏移(小时)
    pub timezone_offset: i32,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            cron: "*/1 * * * * *".to_string(), // 默认每秒执行
            timezone_offset: 0,
            common: CommonConfig {
                node_type: NodeType::Head,
            },
        }
    }
}

#[derive(Debug)]
pub struct ScheduleNode {
    #[allow(dead_code)]
    config: ScheduleConfig,
    schedule: Schedule,
}

impl ScheduleNode {
    pub fn new(config: ScheduleConfig) -> Self {
        let schedule = Schedule::from_str(&config.cron).unwrap();
        Self { config, schedule }
    }

    fn next_schedule_time(&self) -> Option<DateTime<Local>> {
        let now = Utc::now();
        self.schedule
            .after(&now)
            .next()
            .map(|utc| utc.with_timezone(&Local))
    }
}

#[async_trait]
impl NodeHandler for ScheduleNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        loop {
            if let Some(next_time) = self.next_schedule_time() {
                let now = Local::now();
                let delay = next_time.signed_duration_since(now);

                if delay.num_milliseconds() > 0 {
                    sleep(std::time::Duration::from_millis(
                        delay.num_milliseconds() as u64
                    ))
                    .await;
                }

                // 发送到下一个节点
                ctx.send_next(msg.clone()).await?;
            }
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "schedule".to_string(),
            name: "定时节点".to_string(),
            description: "按Cron表达式定时执行".to_string(),
        }
    }
}
