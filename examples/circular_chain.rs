use rule_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{error, info, Level};

// 包含循环的规则链配置
const CIRCULAR_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "循环测试链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "transform",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
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
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "script": "return { value: msg.data.value + 1 };"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "filter",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
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
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 尝试加载循环依赖的规则链
    info!("尝试加载循环依赖的规则链...");
    match engine.load_chain(CIRCULAR_CHAIN).await {
        Ok(_) => {
            error!("错误: 成功加载了循环依赖的规则链!");

            // 如果加载成功，尝试执行看是否会死循环
            info!("尝试执行循环依赖的规则链...");
            let msg = Message::new(
                "test",
                json!({
                    "value": 1
                }),
            );

            match engine.process_msg(msg).await {
                Ok(result) => info!("执行结果: {:?}", result),
                Err(e) => info!("执行失败(预期行为): {}", e),
            }
        }
        Err(e) => info!("加载失败(预期行为): {}", e),
    }

    Ok(())
}
