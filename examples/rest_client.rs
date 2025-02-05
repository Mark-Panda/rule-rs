use rule_rs::{engine::rule::RuleEngineTrait, Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

// const RULE_GET_CHAIN: &str = r#"{
//     "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
//     "name": "REST客户端测试链",
//     "root": true,
//     "nodes": [
//         {
//             "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
//             "type_name": "rest_client",
//             "config": {
//                 "url": "https://api.example.com/weather",
//                 "method": "GET",
//                 "headers": {
//                     "Authorization": "Bearer ${msg.data.token}",
//                     "Content-Type": "application/json"
//                 },
//                 "query_params": {
//                     "city": "${msg.data.city}",
//                     "aqi": "no"
//                 },
//                 "timeout": 5000
//             },
//             "layout": { "x": 100, "y": 100 }
//         },
//         {
//             "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
//             "type_name": "transform_js",
//             "config": {
//                 "script": "const resp = msg.data; return { data: { temperature: resp.temp, humidity: resp.humidity, weather: resp.weather.description } };"
//             },
//             "layout": { "x": 300, "y": 100 }
//         },
//         {
//             "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
//             "type_name": "log",
//             "config": {
//                 "template": "天气信息 - 温度: ${msg.data.temperature}°C, 湿度: ${msg.data.humidity}%, 天气: ${msg.data.weather}"
//             },
//             "layout": { "x": 500, "y": 100 }
//         }
//     ],
//     "connections": [
//         {
//             "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
//             "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
//             "type_name": "success"
//         },
//         {
//             "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
//             "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
//             "type_name": "success"
//         }
//     ],
//     "metadata": {
//         "version": 1,
//         "created_at": 1679800000,
//         "updated_at": 1679800000
//     }
// }"#;

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "天气查询系统",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "rest_client",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
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
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "script": "try { const resp = msg.data; if (!resp || !resp.body) { throw new Error('Invalid response data'); } const location = resp.body.location || {}; const current = resp.body.current || {}; const condition = current.condition || {}; return { location: location.name || 'Unknown', temperature: current.temp_c || 0, condition: condition.text || 'Unknown', humidity: current.humidity || 0, updated_at: current.last_updated || new Date().toISOString() }; } catch (err) { console.error('Error processing weather data:', err); return { location: 'Error', temperature: 0, condition: 'Error: ' + err.message, humidity: 0, updated_at: new Date().toISOString() }; }",
                "output_type": "weather_info"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "transform",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
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
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 加载规则链
    let chain_id = engine.load_chain(RULE_CHAIN).await?;
    info!(
        "规则链加载成功, 版本: {}",
        engine.get_current_version().await
    );

    // 创建测试消息
    let msg = Message::new(
        "weather_query",
        json!({
            "city": "Shanghai",
            "token": "test_token"
        }),
    );

    // 处理消息
    match engine.process_msg(chain_id, msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => info!("处理失败: {:?}", e),
    }

    Ok(())
}
