use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 加载包含日志节点的规则链
    let chain_json = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
        "name": "日志测试链",
        "root": true,
        "nodes": [
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "script",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "script": "return { value: msg.data.value + 1, name: 'test' };"
                },
                "layout": { "x": 100, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
                "type_name": "log",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "template": "处理结果 - ID: ${msg.id}, 类型: ${msg.type}, 值: ${msg.value}, 名称: ${msg.name}"
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

    engine.load_chain(chain_json).await?;

    // 处理消息
    let msg = Message::new("test", json!({ "value": 1 }));
    engine.process_msg(msg).await?;

    Ok(())
}
