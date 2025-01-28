use rulego_rs::RuleEngine;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 1. 获取所有已注册的组件
    info!("已注册的组件:");
    for desc in engine.get_registered_components().await {
        info!("- {}: {}", desc.type_name, desc.description);
    }

    // 2. 加载规则链
    let chain_json = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
        "name": "示例规则链",
        "root": true,
        "nodes": [
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "script",
                "config": {
                    "script": "return { value: msg.data.value + 1 };"
                },
                "layout": { "x": 100, "y": 100 }
            }
        ],
        "connections": [],
        "metadata": {
            "version": 1,
            "created_at": 1679800000,
            "updated_at": 1679800000
        }
    }"#;

    let chain_id = engine.load_chain(chain_json).await?;
    info!("规则链加载成功: {}", chain_id);

    // 3. 加载子规则链
    let subchain_json = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
        "name": "子规则链",
        "root": false,
        "nodes": [
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
                "type_name": "log",
                "config": {
                    "template": "子链处理: ${msg.value}"
                },
                "layout": { "x": 100, "y": 100 }
            }
        ],
        "connections": [],
        "metadata": {
            "version": 1,
            "created_at": 1679800000,
            "updated_at": 1679800000
        }
    }"#;

    let subchain_id = engine.load_chain(subchain_json).await?;
    info!("子规则链加载成功: {}", subchain_id);

    // 4. 获取所有已加载的规则链
    info!("已加载的规则链:");
    for chain in engine.get_loaded_chains().await {
        info!("- {}: {}", chain.id, chain.name);
    }

    // 5. 获取指定规则链
    if let Some(chain) = engine.get_chain(chain_id).await {
        info!("获取规则链成功: {}", chain.name);
        // 使用 serde_json 格式化输出完整的规则链
        match serde_json::to_string_pretty(&*chain) {
            Ok(json) => info!("规则链完整配置:\n{}", json),
            Err(e) => info!("序列化规则链失败: {}", e),
        }
    }

    // 6. 尝试删除根规则链(应该失败)
    match engine.remove_chain(chain_id).await {
        Ok(_) => info!("规则链删除成功"),
        Err(e) => info!("删除根规则链失败(预期行为): {}", e),
    }

    // 7. 删除子规则链
    engine.remove_chain(subchain_id).await?;
    info!("子规则链删除成功");

    // 8. 确认子规则链已被删除
    info!("删除后的规则链列表:");
    for chain in engine.get_loaded_chains().await {
        info!("- {}: {}", chain.id, chain.name);
    }

    Ok(())
}
