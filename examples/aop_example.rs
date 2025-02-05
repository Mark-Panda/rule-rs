use async_trait::async_trait;
use rule_rs::aop::{MessageInterceptor, NodeInterceptor};
use rule_rs::{engine::rule::RuleEngineTrait, Message, NodeContext, RuleEngine, RuleError};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use tracing::{info, Level};
use tracing_subscriber;

// 日志拦截器
#[derive(Debug)]
struct LoggingInterceptor;

#[async_trait]
impl NodeInterceptor for LoggingInterceptor {
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        info!(
            "节点执行前 - 节点类型: {}, 节点ID: {}, 消息ID: {}, 消息类型: {}, 数据: {:?}",
            ctx.node.type_name, ctx.node.id, msg.id, msg.msg_type, msg.data
        );
        Ok(())
    }

    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        info!(
            "节点执行后 - 节点类型: {}, 节点ID: {}, 消息ID: {}, 消息类型: {}, 数据: {:?}",
            ctx.node.type_name, ctx.node.id, msg.id, msg.msg_type, msg.data
        );
        Ok(())
    }

    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError> {
        info!(
            "节点执行错误 - 节点类型: {}, 节点ID: {}, 错误: {:?}",
            ctx.node.type_name, ctx.node.id, error
        );
        Ok(())
    }
}

// 性能监控拦截器
#[derive(Debug)]
struct MetricsInterceptor {
    start_time: std::time::Instant,
}

impl MetricsInterceptor {
    fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
}

#[async_trait]
impl MessageInterceptor for MetricsInterceptor {
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError> {
        info!(
            "开始处理消息 - ID: {}, 类型: {}, 时间: {}, 数据: {:?}",
            msg.id, msg.msg_type, msg.timestamp, msg.data
        );
        Ok(())
    }

    async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        let duration = self.start_time.elapsed();
        info!(
            "消息处理完成 - ID: {}, 类型: {}, 耗时: {:?}, 结果数据: {:?}",
            msg.id, msg.msg_type, duration, msg.data
        );
        Ok(())
    }
}

// 性能分析拦截器
#[derive(Debug)]
struct ProfilingInterceptor {
    start_times: Arc<Mutex<HashMap<String, Instant>>>,
    call_chains: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ProfilingInterceptor {
    fn new() -> Self {
        Self {
            start_times: Arc::new(Mutex::new(HashMap::new())),
            call_chains: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn format_duration(duration: std::time::Duration) -> String {
        if duration.as_micros() < 1000 {
            format!("{}μs", duration.as_micros())
        } else if duration.as_millis() < 1000 {
            format!("{}ms", duration.as_millis())
        } else {
            format!("{}s", duration.as_secs_f32())
        }
    }
}

#[async_trait]
impl NodeInterceptor for ProfilingInterceptor {
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        let node_id = ctx.node.id.to_string();
        let msg_id = msg.id.to_string();

        // 更新开始时间
        self.start_times
            .lock()
            .unwrap()
            .insert(node_id.clone(), Instant::now());

        // 更新调用链
        let mut call_chains = self.call_chains.lock().unwrap();
        let chain = call_chains.entry(msg_id.clone()).or_insert_with(Vec::new);
        chain.push(format!("{}({})", ctx.node.type_name, node_id));

        info!("调用链路 [{}]: {}", msg_id, chain.join(" -> "));

        Ok(())
    }

    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        let node_id = ctx.node.id.to_string();
        if let Some(start_time) = self.start_times.lock().unwrap().get(&node_id) {
            let duration = start_time.elapsed();
            info!(
                "节点性能 [{}] - 类型: {}, ID: {}, 耗时: {}",
                msg.id,
                ctx.node.type_name,
                ctx.node.id,
                Self::format_duration(duration)
            );

            // 清理完成节点的调用链
            let mut call_chains = self.call_chains.lock().unwrap();
            if let Some(chain) = call_chains.get_mut(&msg.id.to_string()) {
                if let Some(pos) = chain.iter().position(|x| x.contains(&node_id)) {
                    chain.truncate(pos + 1);
                }
            }
        }
        Ok(())
    }

    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError> {
        let node_id = ctx.node.id.to_string();
        if let Some(start_time) = self.start_times.lock().unwrap().get(&node_id) {
            let duration = start_time.elapsed();
            info!(
                "节点错误 - 类型: {}, ID: {}, 耗时: {}, 错误: {:?}",
                ctx.node.type_name,
                ctx.node.id,
                Self::format_duration(duration),
                error
            );
        }
        Ok(())
    }
}

// 统计拦截器
#[derive(Debug)]
struct StatsInterceptor {
    node_stats: Arc<Mutex<HashMap<String, NodeStats>>>,
    start_times: Arc<Mutex<HashMap<String, Instant>>>,
}

#[derive(Debug, Default)]
struct NodeStats {
    total_calls: u64,
    total_errors: u64,
    total_duration: Duration,
    min_duration: Option<Duration>,
    max_duration: Option<Duration>,
}

impl StatsInterceptor {
    fn new() -> Self {
        Self {
            node_stats: Arc::new(Mutex::new(HashMap::new())),
            start_times: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn update_stats(&self, node_type: &str, duration: Duration, is_error: bool) {
        let mut stats = self.node_stats.lock().unwrap();
        let node_stats = stats.entry(node_type.to_string()).or_default();

        node_stats.total_calls += 1;
        if is_error {
            node_stats.total_errors += 1;
        }
        node_stats.total_duration += duration;

        if let Some(min_dur) = node_stats.min_duration {
            if duration < min_dur {
                node_stats.min_duration = Some(duration);
            }
        } else {
            node_stats.min_duration = Some(duration);
        }

        if let Some(max_dur) = node_stats.max_duration {
            if duration > max_dur {
                node_stats.max_duration = Some(duration);
            }
        } else {
            node_stats.max_duration = Some(duration);
        }
    }

    fn print_stats(&self) {
        let stats = self.node_stats.lock().unwrap();
        info!("\n=== 节点统计信息 ===");
        for (node_type, stats) in stats.iter() {
            let avg_duration = stats.total_duration.as_micros() as f64 / stats.total_calls as f64;
            info!(
                "节点类型: {}\n  总调用: {}\n  错误数: {}\n  平均耗时: {:.2}μs\n  最小耗时: {}μs\n  最大耗时: {}μs",
                node_type,
                stats.total_calls,
                stats.total_errors,
                avg_duration,
                stats.min_duration.unwrap().as_micros(),
                stats.max_duration.unwrap().as_micros()
            );
        }
    }
}

#[async_trait]
impl NodeInterceptor for StatsInterceptor {
    async fn before<'a>(&self, ctx: &NodeContext<'a>, _msg: &Message) -> Result<(), RuleError> {
        // 记录节点开始时间
        self.start_times
            .lock()
            .unwrap()
            .insert(ctx.node.id.to_string(), Instant::now());
        Ok(())
    }

    async fn after<'a>(&self, ctx: &NodeContext<'a>, _msg: &Message) -> Result<(), RuleError> {
        if let Some(start_time) = self
            .start_times
            .lock()
            .unwrap()
            .get(&ctx.node.id.to_string())
        {
            let duration = start_time.elapsed();
            self.update_stats(&ctx.node.type_name, duration, false);
        }
        Ok(())
    }

    async fn error<'a>(&self, ctx: &NodeContext<'a>, _error: &RuleError) -> Result<(), RuleError> {
        if let Some(start_time) = self
            .start_times
            .lock()
            .unwrap()
            .get(&ctx.node.id.to_string())
        {
            let duration = start_time.elapsed();
            self.update_stats(&ctx.node.type_name, duration, true);
        }
        Ok(())
    }
}

// 在 main 函数前添加规则链配置
const CHAIN_A: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "测试规则链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "script",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "script": "return { value: msg.data.value + 1 };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "filter",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "condition": "value < 10",
                "js_script": null
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "success"
        }
    ],
    "metadata": {
        "version": 1,
        "created_at": 1679800000,
        "updated_at": 1679800000
    }
}"#;

// 添加一个会产生错误的规则链配置
const ERROR_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
    "name": "错误测试链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3402",
            "type_name": "script",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
            "config": {
                "script": "return { value: msg.data.invalid_field.value };"
            },
            "layout": { "x": 100, "y": 100 }
        }
    ],
    "connections": [],
    "metadata": {
        "version": 1,
        "created_at": 1679800000,
        "updated_at": 1679800000
    }
}"#;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 创建统计拦截器
    let stats_interceptor = Arc::new(StatsInterceptor::new());

    // 注册节点拦截器
    engine
        .add_node_interceptor(Arc::new(LoggingInterceptor))
        .await;

    engine
        .add_node_interceptor(Arc::new(ProfilingInterceptor::new()))
        .await;

    engine.add_node_interceptor(stats_interceptor.clone()).await;

    // 注册消息拦截器
    engine
        .add_msg_interceptor(Arc::new(MetricsInterceptor::new()))
        .await;

    // 加载并执行正常规则链
    let msg = Message::new("test", json!({"value": 1}));
    let chain_id = engine.load_chain(CHAIN_A).await?;
    engine.process_msg(chain_id, msg).await?;

    // 加载并执行错误规则链
    let msg = Message::new("test", json!({"value": 1}));
    let chain_id1 = engine.load_chain(ERROR_CHAIN).await?;
    let _ = engine.process_msg(chain_id1, msg).await; // 忽略错误结果

    // 打印统计信息
    stats_interceptor.print_stats();

    Ok(())
}
