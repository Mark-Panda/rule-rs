use rulego_rs::{Message, RuleEngine};
use tracing::{info, Level};

// 规则链 A
const CHAIN_A: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "规则链A",
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

// 规则链 B
const CHAIN_B: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3401",
    "name": "规则链B",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3402",
            "type_name": "transform",
            "config": {
                "template": {
                    "value": "${msg.data.value}"
                }
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3403",
            "type_name": "subchain",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3501",
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

// 规则链 C
const CHAIN_C: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3501",
    "name": "规则链C",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3502",
            "type_name": "filter",
            "config": {
                "condition": "value < 10",
                "js_script": null
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3503",
            "type_name": "subchain",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "output_type": "test"
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
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = RuleEngine::new();

    // 按顺序加载三个规则链
    match engine.load_chain(CHAIN_A).await {
        Ok(_) => info!("规则链A加载成功"),
        Err(e) => info!("规则链A加载失败: {:?}", e),
    }

    match engine.load_chain(CHAIN_B).await {
        Ok(_) => info!("规则链B加载成功"),
        Err(e) => info!("规则链B加载失败: {:?}", e),
    }

    match engine.load_chain(CHAIN_C).await {
        Ok(_) => {
            info!("规则链C加载成功");

            let msg = Message::new(
                "test",
                serde_json::json!({
                    "value": 1
                }),
            );

            match engine.process_msg(msg).await {
                Ok(result) => info!("处理结果: {:?}", result),
                Err(e) => info!("处理失败: {:?}", e),
            }
        }
        Err(e) => info!("规则链C加载失败: {:?}", e),
    }

    Ok(())
}
