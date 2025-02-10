use rule_rs::{engine::rule::RuleEngineTrait, Message, RuleEngine};
use serde_json::json;
use tracing::info;

#[tokio::main]
async fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 创建规则引擎并等待初始化完成
    let engine = RuleEngine::new().await;

    // 加载规则链
    let rule_chain = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
        "name": "transform示例",
        "root": true,
        "nodes": [
            {
                "id": "00000000-0000-0000-0000-000000000000",
                "type_name": "start",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {},
                "layout": { "x": 50, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "transform",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "template": {
                        "name": "${msg.data.name}",
                        "age": "${msg.data.age}",
                        "greeting": "Hello ${msg.data.name}!"
                    },
                    "node_type": "middle"
                },
                "layout": {
                    "x": 100,
                    "y": 100
                }
            }
        ],
        "connections": [
            {
                "from_id": "00000000-0000-0000-0000-000000000000",
                "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "success"
            }
        ],
        "metadata": {
            "version": 1,
            "created_at": 1625097600000,
            "updated_at": 1625097600000
        }
    }"#;

    match engine.load_chain(rule_chain).await {
        Ok(chain_id) => {
            info!(
                "规则链加载成功, 版本: {}",
                engine.get_current_version().await
            );

            // 创建测试消息
            let msg = Message::new(
                "test",
                json!({
                    "name": "Alice",
                    "age": 20
                }),
            );

            // 处理消息
            match engine.process_msg(chain_id, msg).await {
                Ok(result) => {
                    info!("处理成功: {:?}", result);
                }
                Err(e) => {
                    info!("处理失败: {:?}", e);
                }
            }
        }
        Err(e) => {
            info!("规则链加载失败: {:?}", e);
        }
    }
}
