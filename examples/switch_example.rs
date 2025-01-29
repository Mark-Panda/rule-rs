use rulego_rs::{Message, RuleEngine};
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
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例
    let engine = RuleEngine::new().await;

    // 加载规则链
    engine.load_chain(RULE_CHAIN).await?;

    // 测试不同温度场景
    let test_temps = vec![35.0, 5.0, 25.0];

    for temp in test_temps {
        let msg = Message::new(
            "temperature",
            json!({
                "temperature": temp,
                "device_id": "sensor_001",
                "timestamp": chrono::Utc::now().timestamp_millis()
            }),
        );

        info!("处理温度数据: {}°C", temp);
        match engine.process_msg(msg).await {
            Ok(_) => info!("温度处理完成"),
            Err(e) => info!("处理失败: {:?}", e),
        }
    }

    Ok(())
}
