use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{error, info, Level};

// 规则链 A
const CHAIN_A: &str = r#"{
    "id": "chain-a",
    "name": "Chain A",
    "root": true,
    "nodes": [
        {
            "id": "node-a1",
            "type_name": "log",
            "chain_id": "chain-a",
            "config": {
                "template": "Chain A -> Chain B"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "node-a2",
            "type_name": "subchain",
            "chain_id": "chain-a",
            "config": {
                "chain_id": "chain-b"
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "node-a1",
            "to_id": "node-a2",
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
    "id": "chain-b",
    "name": "Chain B",
    "root": false,
    "nodes": [
        {
            "id": "node-b1",
            "type_name": "log",
            "chain_id": "chain-b",
            "config": {
                "template": "Chain B -> Chain C"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "node-b2",
            "type_name": "subchain",
            "chain_id": "chain-b",
            "config": {
                "chain_id": "chain-c"
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "node-b1",
            "to_id": "node-b2",
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
    "id": "chain-c",
    "name": "Chain C",
    "root": false,
    "nodes": [
        {
            "id": "node-c1",
            "type_name": "log",
            "chain_id": "chain-c",
            "config": {
                "template": "Chain C -> Chain A"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "node-c2",
            "type_name": "subchain",
            "chain_id": "chain-c",
            "config": {
                "chain_id": "chain-a"
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "node-c1",
            "to_id": "node-c2",
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
