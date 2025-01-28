use rulego_rs::{Message, RuleEngine};
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "温度单位转换规则链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "transform_js",
            "config": {
                "script": "const celsius = msg.value; const fahrenheit = (celsius * 9/5) + 32; return { value: fahrenheit, unit: 'fahrenheit', original: { value: celsius, unit: 'celsius' } };"
            },
            "layout": {
                "x": 100,
                "y": 100
            }
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
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建规则引擎实例
    let engine = RuleEngine::new();

    // 加载规则链配置
    engine.load_chain(RULE_CHAIN).await?;

    info!(
        "规则链加载完成, 版本: {}",
        engine.get_current_version().await
    );

    // 创建测试消息
    let msg = Message::new(
        "temperature",
        serde_json::json!({
            "value": 25,
            "unit": "celsius"
        }),
    );

    info!("开始处理消息: {:?}", msg);

    // 处理消息
    match engine.process_msg(msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => eprintln!("处理失败: {:?}", e),
    }

    Ok(())
}
