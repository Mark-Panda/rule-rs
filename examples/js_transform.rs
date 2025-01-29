use rule_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "JS转换测试链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "transform_js",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "script": "const celsius = msg.data.value; const fahrenheit = (celsius * 9/5) + 32; return { celsius: celsius, fahrenheit: fahrenheit  };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "温度转换: ${msg.data.celsius}°C = ${msg.data.fahrenheit}°F"
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
        "temperature",
        json!({
            "data": {
                "value": 25
            }
        }),
    );

    // 处理消息
    match engine.process_msg(msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => info!("处理失败: {:?}", e),
    }

    Ok(())
}
