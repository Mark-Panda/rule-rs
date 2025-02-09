use rule_rs::engine::rule::RuleEngineTrait;
use rule_rs::engine::RuleEngine;
use rule_rs::types::Message;
use serde_json::json;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建规则引擎
    let engine = RuleEngine::new().await;

    let chain_id = Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap();

    // 加载规则链配置
    let chain_json = r#"{
        "id": "01234567-89ab-cdef-0123-456789abcdef",
        "name": "Fork-Join测试",
        "type_name": "fork_join_test",
        "root": true,
        "metadata": {
            "version": 1,
            "description": "Fork-Join测试规则链",
            "author": "test",
            "created_at": 1707321600,
            "updated_at": 1707321600
        },
        "nodes": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "chain_id": "01234567-89ab-cdef-0123-456789abcdef",
                "type_name": "fork",
                "type": "fork",
                "config": {},
                "layout": {"x": 100, "y": 100}
            },
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "chain_id": "01234567-89ab-cdef-0123-456789abcdef",
                "type_name": "transform",
                "type": "transform",
                "config": {
                    "template": {"value": "branch1"}
                },
                "layout": {"x": 300, "y": 50}
            },
            {
                "id": "33333333-3333-3333-3333-333333333333",
                "chain_id": "01234567-89ab-cdef-0123-456789abcdef",
                "type_name": "transform",
                "type": "transform",
                "config": {
                    "template": {"value": "branch2"}
                },
                "layout": {"x": 300, "y": 150}
            },
            {
                "id": "44444444-4444-4444-4444-444444444444",
                "chain_id": "01234567-89ab-cdef-0123-456789abcdef",
                "type_name": "join",
                "type": "join",
                "config": {
                    "timeout": 5
                },
                "layout": {"x": 500, "y": 100}
            }
        ],
        "connections": [
            {
                "from_id": "11111111-1111-1111-1111-111111111111",
                "to_id": "22222222-2222-2222-2222-222222222222",
                "type_name": "Success"
            },
            {
                "from_id": "11111111-1111-1111-1111-111111111111",
                "to_id": "33333333-3333-3333-3333-333333333333",
                "type_name": "Success"
            },
            {
                "from_id": "22222222-2222-2222-2222-222222222222",
                "to_id": "44444444-4444-4444-4444-444444444444",
                "type_name": "Success"
            },
            {
                "from_id": "33333333-3333-3333-3333-333333333333",
                "to_id": "44444444-4444-4444-4444-444444444444",
                "type_name": "Success"
            }
        ]
    }"#;

    // 加载规则链
    match engine.load_chain(chain_json).await {
        Ok(_) => {
            // 创建测试消息
            let msg = Message::new("test", json!({"value": 0}));

            // 执行规则链
            match engine.process_msg(chain_id, msg).await {
                Ok(result) => println!("执行结果: {:?}", result),
                Err(e) => println!("执行失败: {:?}", e),
            }
        }
        Err(e) => println!("加载规则链失败: {:?}", e),
    }
}
