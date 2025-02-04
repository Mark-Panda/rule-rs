use rule_rs::{engine::rule::RuleEngineTrait, Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 加载包含天气节点的规则链
    let chain_json = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
        "name": "天气服务测试链",
        "root": true,
        "nodes": [
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "weather",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "api_key": "5b90fd27b0eb4fcf86132317252801",
                    "city": "Shanghai",
                    "language": "zh"
                },
                "layout": { "x": 100, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
                "type_name": "log",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "config": {
                    "template": "天气信息 - 城市: ${msg.城市}, 温度: ${msg.温度}, 天气: ${msg.天气}, 湿度: ${msg.湿度}, 更新时间: ${msg.更新时间}"
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
    let msg = Message::new(
        "query",
        json!({
            "city": "Shanghai",
            "aqi": "no"
        }),
    );

    info!("开始处理消息: {:?}", msg);
    let result = engine.process_msg(msg).await?;
    info!("处理结果: {:?}", result);

    Ok(())
}
