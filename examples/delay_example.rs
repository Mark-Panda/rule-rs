use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "延迟示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "delay",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "delay_ms": 2000,
                "periodic": true,
                "period_count": 3,
                "common": {
                    "node_type": "Head"
                }
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "延迟消息: ${msg.data.value}",
                "common": {
                    "node_type": "Tail"
                }
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 加载规则链
    engine.load_chain(RULE_CHAIN).await?;
    info!(
        "规则链加载成功, 版本: {}",
        engine.get_current_version().await
    );

    // 创建测试消息
    let msg = Message::new(
        "test",
        json!({
            "value": "hello"
        }),
    );

    // 处理消息
    match engine.process_msg(msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => info!("处理失败: {:?}", e),
    }

    // 等待一段时间以便观察周期性消息
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    Ok(())
}
