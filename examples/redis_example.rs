use rule_rs::{engine::rule::RuleEngineTrait, Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "Redis Hash JSON示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "redis",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "url": "redis://localhost:6379",
                "operation": {
                    "type": "Command",
                    "config": "HSET"
                },
                "key": "user:1",
                "field": "profile",
                "value": "{\"name\":\"张三\",\"age\":25,\"email\":\"zhangsan@example.com\"}",
                "success_branch": "get",
                "error_branch": "error"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "redis",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "url": "redis://localhost:6379",
                "operation": {
                    "type": "Command",
                    "config": "HGET"
                },
                "key": "user:1",
                "field": "profile",
                "success_branch": "log",
                "error_branch": "error"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "用户信息: ${msg.data.value}",
                "common": {
                    "node_type": "Tail"
                }
            },
            "layout": { "x": 500, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "get"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log"
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
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例
    let engine = RuleEngine::new().await;

    // 加载规则链
    let chain_id = engine.load_chain(RULE_CHAIN).await?;

    info!(
        "规则链加载成功, 版本: {}",
        engine.get_current_version().await
    );

    // 创建测试消息
    let msg = Message::new(
        "test",
        json!({
            "value": "hello redis"
        }),
    );

    // 处理消息
    match engine.process_msg(chain_id, msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => info!("处理失败: {:?}", e),
    }

    Ok(())
}
