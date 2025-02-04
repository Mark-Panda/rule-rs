use rule_rs::{engine::rule::RuleEngineTrait, Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "温度监控规则链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "switch",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "cases": [
                    {
                        "name": "high_temp",
                        "condition": "msg.data.temperature > 30",
                        "description": "温度过高告警"
                    },
                    {
                        "name": "low_temp", 
                        "condition": "msg.data.temperature < 10",
                        "description": "温度过低告警"
                    }
                ],
                "default_next": "normal_temp"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "温度过高告警: ${msg.data.temperature}°C"
            },
            "layout": { "x": 300, "y": 50 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "温度过低告警: ${msg.data.temperature}°C"
            },
            "layout": { "x": 300, "y": 150 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "温度正常: ${msg.data.temperature}°C"
            },
            "layout": { "x": 300, "y": 250 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "high_temp"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "low_temp"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "type_name": "normal_temp"
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
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = RuleEngine::new().await;
    engine.load_chain(RULE_CHAIN).await?;

    // 测试高温场景
    let high_temp_msg = Message::new(
        "temperature",
        json!({
            "temperature": 35
        }),
    );

    info!("测试高温场景 - 35°C");
    match engine.process_msg(high_temp_msg).await {
        Ok(_) => info!("高温场景处理完成"),
        Err(e) => info!("高温场景处理失败: {:?}", e),
    }

    // 测试低温场景
    let low_temp_msg = Message::new(
        "temperature",
        json!({
            "temperature": 5
        }),
    );

    info!("测试低温场景 - 5°C");
    match engine.process_msg(low_temp_msg).await {
        Ok(_) => info!("低温场景处理完成"),
        Err(e) => info!("低温场景处理失败: {:?}", e),
    }

    // 测试正常温度场景
    let normal_temp_msg = Message::new(
        "temperature",
        json!({
            "temperature": 25
        }),
    );

    info!("测试正常温度场景 - 25°C");
    match engine.process_msg(normal_temp_msg).await {
        Ok(_) => info!("正常温度场景处理完成"),
        Err(e) => info!("正常温度场景处理失败: {:?}", e),
    }

    Ok(())
}
