use rulego_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "HTTP请求示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "rest_client",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "url": "https://api.weatherapi.com/v1/current.json?key=5b90fd27b0eb4fcf86132317252801&q=${city}",
                "method": "GET",
                "headers": {
                    "Accept": "application/json"
                },
                "timeout_ms": 5000,
                "success_branch": "request_success",
                "error_branch": "request_failed"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "天气查询成功 - ${msg.data.body.current.temp_c}°C"
            },
            "layout": { "x": 300, "y": 50 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "天气查询失败 - ${msg.metadata.error}"
            },
            "layout": { "x": 300, "y": 150 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "request_success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "request_failed"
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
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例
    let engine = RuleEngine::new().await;

    // 加载规则链
    engine.load_chain(RULE_CHAIN).await?;

    // 成功场景 - 有效的城市名
    let success_msg = Message::new(
        "weather_query",
        json!({
            "city": "Shanghai"
        }),
    );

    info!("测试成功场景 - 查询上海天气");
    match engine.process_msg(success_msg).await {
        Ok(_) => info!("成功场景处理完成"),
        Err(e) => info!("成功场景处理失败: {:?}", e),
    }

    // 失败场景 - 无效的城市名
    let error_msg = Message::new(
        "weather_query",
        json!({
            "city": "NonExistentCity123"
        }),
    );

    info!("测试失败场景 - 查询不存在的城市");
    match engine.process_msg(error_msg).await {
        Ok(_) => info!("失败场景处理完成"),
        Err(e) => info!("失败场景处理失败: {:?}", e),
    }

    Ok(())
}
