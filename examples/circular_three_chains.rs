use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{error, info, Level};

// 规则链 A
const CHAIN_A: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "规则链 A",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "script",
            "config": {
                "script": "return { value: msg.data.value + 1 };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "subchain",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401"
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

// 规则链 B
const CHAIN_B: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
    "name": "规则链 B",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3402",
            "type_name": "script",
            "config": {
                "script": "return { value: msg.data.value * 2 };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3403",
            "type_name": "subchain",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3501"
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

// 规则链 C
const CHAIN_C: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3501",
    "name": "规则链 C",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3502",
            "type_name": "script",
            "config": {
                "script": "return { value: msg.data.value + 3 };"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3503",
            "type_name": "subchain",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301"
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3502",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3503",
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

    // 尝试加载规则链 A
    info!("尝试加载规则链 A...");
    match engine.load_chain(CHAIN_A).await {
        Ok(_) => info!("规则链 A 加载成功"),
        Err(e) => error!("规则链 A 加载失败: {}", e),
    }

    // 尝试加载规则链 B
    info!("尝试加载规则链 B...");
    match engine.load_chain(CHAIN_B).await {
        Ok(_) => info!("规则链 B 加载成功"),
        Err(e) => error!("规则链 B 加载失败: {}", e),
    }

    // 尝试加载规则链 C
    info!("尝试加载规则链 C...");
    match engine.load_chain(CHAIN_C).await {
        Ok(_) => {
            info!("规则链 C 加载成功");

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
        Err(e) => info!("规则链 C 加载失败(预期行为): {}", e),
    }

    Ok(())
}
