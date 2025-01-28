use rulego_rs::{Message, RuleEngine};
use tracing::{info, Level};

// 包含循环的规则链配置
const CIRCULAR_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "循环测试链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "transform",
            "config": {
                "template": {
                    "value": "${msg.data.value}"
                }
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303", 
            "type_name": "script",
            "config": {
                "script": "return { value: msg.data.value + 1 };"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "filter",
            "config": {
                "condition": "value < 10",
                "js_script": null
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
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
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

    // 尝试加载包含循环的规则链
    match engine.load_chain(CIRCULAR_CHAIN).await {
        Ok(_) => {
            info!("规则链加载成功");

            // 创建测试消息
            let msg = Message::new(
                "test",
                serde_json::json!({
                    "value": 1
                }),
            );

            // 尝试处理消息
            match engine.process_msg(msg).await {
                Ok(result) => info!("处理结果: {:?}", result),
                Err(e) => info!("处理失败: {:?}", e),
            }
        }
        Err(e) => info!("规则链加载失败: {:?}", e),
    }

    Ok(())
}
