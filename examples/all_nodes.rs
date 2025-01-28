use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{error, info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "温度监控系统",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "filter",
            "config": {
                "condition": "temperature > 0",
                "js_script": null
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "transform_js",
            "config": {
                "script": "const celsius = msg.value; const fahrenheit = (celsius * 9/5) + 32; return { celsius, fahrenheit };"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "switch",
            "config": {
                "cases": [
                    {
                        "condition": "value > 30",
                        "output": "high_temp"
                    },
                    {
                        "condition": "value < 10",
                        "output": "low_temp"
                    }
                ],
                "default": "normal_temp"
            },
            "layout": { "x": 500, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "type_name": "script",
            "config": {
                "script": "const temp = msg.celsius; const status = msg.type === 'high_temp' ? '过热' : msg.type === 'low_temp' ? '过冷' : '正常'; return { temperature: temp, status: status, alert: msg.type !== 'normal_temp', timestamp: new Date().toISOString() };",
                "output_type": "alert"
            },
            "layout": { "x": 700, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3306",
            "type_name": "transform",
            "config": {
                "template": {
                    "device_id": "temp_sensor_01",
                    "alert_type": "${msg.type}",
                    "alert_message": "温度${msg.status}: ${msg.temperature}°C",
                    "need_action": "${msg.alert}",
                    "created_at": "${msg.timestamp}"
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
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 加载规则链
    engine.load_chain(RULE_CHAIN).await?;

    // 获取当前版本
    info!("当前规则链版本: {}", engine.get_current_version().await);

    // 处理消息
    let msg = Message::new(
        "test",
        json!({
            "value": 1,
            "type": "test"
        }),
    );

    info!("开始处理消息: {:?}", msg);
    match engine.process_msg(msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => error!("处理失败: {}", e),
    }

    Ok(())
}
