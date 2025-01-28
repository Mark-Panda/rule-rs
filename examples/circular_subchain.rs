use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{error, info, Level};

// 子规则链配置
const SUB_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
    "name": "子规则链",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3402",
            "type_name": "script",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
            "config": {
                "script": "return { value: msg.data.value + 1 };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3403",
            "type_name": "subchain",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "output_type": "test"
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

// 主规则链配置，引用子规则链并形成循环
const MAIN_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "主规则链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "filter",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "condition": "value < 10",
                "js_script": null
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "subchain",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
                "output_type": "test"
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

    // 先加载子规则链
    info!("尝试加载子规则链...");
    match engine.load_chain(SUB_CHAIN).await {
        Ok(_) => info!("子规则链加载成功"),
        Err(e) => error!("子规则链加载失败: {}", e),
    }

    // 再加载主规则链
    info!("尝试加载主规则链...");
    match engine.load_chain(MAIN_CHAIN).await {
        Ok(_) => {
            info!("主规则链加载成功");

            // 尝试执行规则链
            info!("尝试执行规则链...");
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
        Err(e) => info!("主规则链加载失败(预期行为): {}", e),
    }

    Ok(())
}
