use rulego_rs::{Message, RuleEngine};
use tracing::{info, Level};

// 子规则链：数据转换
const SUB_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
    "name": "数据转换子链",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3402",
            "type_name": "transform_js",
            "config": {
                "script": "const value = msg.value; return { celsius: value, fahrenheit: (value * 9/5) + 32 };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3403",
            "type_name": "script",
            "config": {
                "script": "const data = msg.data; return { temperature: { value: data.celsius, unit: 'C' }, converted: { value: data.fahrenheit, unit: 'F' } };",
                "output_type": "temperature_data"
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3402",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3403",
            "type_name": "success"
        }
    ],
    "metadata": {
        "version": 1,
        "created_at": 1679800000,
        "updated_at": 1679800000
    }
}"#;

// 主规则链
const MAIN_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "温度监控系统",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "filter",
            "config": {
                "condition": "value > 0",
                "js_script": null
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "subchain",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
                "output_type": "converted_data"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "switch",
            "config": {
                "cases": [
                    {
                        "condition": "msg.data.temperature.value > 30",
                        "output": "high_temp"
                    },
                    {
                        "condition": "msg.data.temperature.value < 10",
                        "output": "low_temp"
                    }
                ],
                "default": "normal_temp"
            },
            "layout": { "x": 500, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "type_name": "rest_client",
            "config": {
                "url": "https://httpbin.org/get",
                "method": "GET",
                "headers": null,
                "timeout_ms": 5000,
                "output_type": "http_response"
            },
            "layout": { "x": 700, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3306",
            "type_name": "transform",
            "config": {
                "template": {
                    "摄氏度": "${msg.data.temperature.value}°C",
                    "华氏度": "${msg.data.converted.value}°F",
                    "状态": "${msg.type}",
                    "时间": "${msg.timestamp}"
                }
            },
            "layout": { "x": 900, "y": 100 }
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
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "type_name": "success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3306",
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

    // 先加载子规则链
    engine.load_chain(SUB_CHAIN).await?;

    // 再加载主规则链
    engine.load_chain(MAIN_CHAIN).await?;

    info!(
        "规则链加载完成, 版本: {}",
        engine.get_current_version().await
    );

    // 创建测试消息
    let msg = Message::new(
        "temperature",
        serde_json::json!({
            "value": 25.0,
            "unit": "C"
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
