use chrono;
use rule_rs::{
    engine::rule::{RuleEngine, RuleEngineTrait},
    types::{Message, RuleError},
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use tokio;
use tracing::{info, Level};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), RuleError> {
    // 初始化规则引擎
    let engine = RuleEngine::new().await;

    // 加载一个测试规则链
    let chain_json = r#"{
        "id": "11111111-1111-1111-1111-111111111111",
        "name": "test chain",
        "root": true,
        "nodes": [
            {
                "id": "22222222-2222-2222-2222-222222222222",
                "type_name": "delay",
                "chain_id": "11111111-1111-1111-1111-111111111111",
                "layout": {
                    "x": 100,
                    "y": 100
                },
                "config": {
                    "delay_ms": 1000,
                    "periodic": true,
                    "period_count": 3,
                    "common": {
                        "node_type": "middle"
                    }
                }
            },
            {
                "id": "33333333-3333-3333-3333-333333333333", 
                "type_name": "log",
                "chain_id": "11111111-1111-1111-1111-111111111111",
                "layout": {
                    "x": 300,
                    "y": 100
                },
                "config": {
                    "template": "执行完成",
                    "common": {
                        "node_type": "tail"
                    }
                }
            }
        ],
        "connections": [
            {
                "from_id": "22222222-2222-2222-2222-222222222222",
                "to_id": "33333333-3333-3333-3333-333333333333",
                "type_name": "success"
            }
        ],
        "metadata": {
            "version": 1,
            "created_at": 1679800000,
            "updated_at": 1679800000
        }
    }"#;

    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let chain_id = engine.load_chain(chain_json).await?;
    println!("规则链加载完成: {}", chain_id);

    info!("已注册的组件:");
    for desc in engine.get_registered_components().await {
        info!("- {}: {}", desc.type_name, desc.description);
    }

    // 创建多个并发任务执行规则链
    let mut handles = vec![];
    for i in 0..5 {
        let engine = engine.clone();
        let handle = tokio::spawn(async move {
            let msg = Message {
                data: json!({
                    "index": i
                }),
                metadata: HashMap::new(),
                id: Uuid::new_v4(),
                msg_type: "test".to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            };
            println!("开始执行任务 {}", i);
            let result = engine.process_msg(chain_id, msg).await;
            println!("任务 {} 执行结果: {:?}", i, result);
        });
        handles.push(handle);
    }

    // 在删除规则链之前等待一小段时间
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("尝试删除规则链");
    match engine.remove_chain(chain_id).await {
        Ok(_) => println!("规则链删除成功"),
        Err(e) => println!("规则链删除失败: {:?}", e),
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
