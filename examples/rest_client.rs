use rulego_rs::{Message, RuleEngine};
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "天气查询系统",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "rest_client",
            "config": {
                "url": "https://api.weatherapi.com/v1/current.json?key=5b90fd27b0eb4fcf86132317252801&q=${q}&aqi=${aqi}",
                "method": "GET",
                "headers": {},
                "timeout_ms": 5000,
                "output_type": "weather_data"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "script",
            "config": {
                "script": "try { const resp = msg.data; if (!resp || !resp.body) { throw new Error('Invalid response data'); } const location = resp.body.location || {}; const current = resp.body.current || {}; const condition = current.condition || {}; return { location: location.name || 'Unknown', temperature: current.temp_c || 0, condition: condition.text || 'Unknown', humidity: current.humidity || 0, updated_at: current.last_updated || new Date().toISOString() }; } catch (err) { console.error('Error processing weather data:', err); return { location: 'Error', temperature: 0, condition: 'Error: ' + err.message, humidity: 0, updated_at: new Date().toISOString() }; }",
                "output_type": "weather_info"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "transform",
            "config": {
                "template": {
                    "城市": "${msg.location}",
                    "温度": "${msg.temperature}°C",
                    "天气": "${msg.condition}",
                    "湿度": "${msg.humidity}%",
                    "更新时间": "${msg.updated_at}"
                }
            },
            "layout": { "x": 500, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
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

    // 创建查询消息
    let msg = Message::new(
        "query",
        serde_json::json!({
            "q": "Shanghai",
            "aqi": "no"
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
