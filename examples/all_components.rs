use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 完整的规则链配置
    let chain_json = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
        "name": "完整示例规则链",
        "root": true,
        "nodes": [
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "script",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "script": "return { value: msg.data.value + 1, type: msg.type };"
                },
                "layout": { "x": 100, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
                "type_name": "filter",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "condition": "msg.value > 10",
                    "checkFields": ["value", "type"],
                    "strict": false
                },
                "layout": { "x": 300, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
                "type_name": "transform",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "fields": {
                        "transformed_value": "${msg.value * 2}",
                        "timestamp": "${Date.now()}",
                        "source": "transform_node"
                    },
                    "dropFields": ["temp_field"]
                },
                "layout": { "x": 500, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
                "type_name": "transform_js",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "script": "function transform(msg) { return { ...msg, processed: true, processed_time: new Date().toISOString() }; }"
                },
                "layout": { "x": 700, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3306",
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
                "layout": { "x": 900, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3307",
                "type_name": "rest_client",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "url": "https://api.example.com/data",
                    "method": "POST",
                    "headers": {
                        "Content-Type": "application/json",
                        "Authorization": "Bearer ${msg.token}"
                    },
                    "params": {
                        "id": "${msg.id}",
                        "type": "${msg.type}"
                    },
                    "body": {
                        "data": "${msg}",
                        "timestamp": "${Date.now()}"
                    },
                    "timeout": 5000,
                    "retry": {
                        "maxAttempts": 3,
                        "delay": 1000
                    }
                },
                "layout": { "x": 1100, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3308",
                "type_name": "weather",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "api_key": "your_api_key_here",
                    "city": "${msg.city}",
                    "language": "zh"
                },
                "layout": { "x": 1300, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3309",
                "type_name": "log",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "template": "处理结果 - ID: ${msg.id}, 类型: ${msg.type}, 值: ${msg.value}, 天气: ${msg.weather}",
                    "level": "info"
                },
                "layout": { "x": 1500, "y": 100 }
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
            },
            {
                "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3306",
                "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3307",
                "type_name": "high_value"
            },
            {
                "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3307",
                "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3308",
                "type_name": "success"
            },
            {
                "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3308",
                "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3309",
                "type_name": "success"
            }
        ],
        "metadata": {
            "version": 1,
            "created_at": 1679800000,
            "updated_at": 1679800000
        }
    }"#;

    // 加载规则链
    let chain_id = engine.load_chain(chain_json).await?;
    info!("规则链加载成功: {}", chain_id);

    // 处理消息
    let msg = Message::new(
        "test",
        json!({
            "value": 25,
            "city": "Shanghai",
            "token": "test_token",
            "type": "example"
        }),
    );

    info!("开始处理消息: {:?}", msg);
    let result = engine.process_msg(msg).await?;
    info!("处理结果: {:?}", result);

    Ok(())
}
