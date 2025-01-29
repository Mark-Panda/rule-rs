use rulego_rs::RuleEngine;
use tracing::{info, Level};

const MAIN_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "主规则链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "主链处理消息"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "subchain",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304"
            },
            "layout": { "x": 300, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "success"
        }
    ],
    "metadata": {
        "version": 1,
        "created_at": 1679800000,
        "updated_at": 1679800000
    }
}"#;

const SUB_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
    "name": "子规则链",
    "root": false,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3305",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "config": {
                "template": "子链处理消息"
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

    // 2. 先加载子规则链
    let subchain_id = engine.load_chain(SUB_CHAIN).await?;
    info!("子规则链加载成功: {}", subchain_id);

    // 3. 再加载主规则链
    let chain_id = engine.load_chain(MAIN_CHAIN).await?;
    info!("主规则链加载成功: {}", chain_id);

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

    // 7. 先删除主规则链
    engine.remove_chain(chain_id).await?;
    info!("主规则链删除成功");

    // 8. 再删除子规则链
    engine.remove_chain(subchain_id).await?;
    info!("子规则链删除成功");

    // 9. 确认规则链已被删除
    info!("删除后的规则链列表:");
    for chain in engine.get_loaded_chains().await {
        info!("- {}: {}", chain.id, chain.name);
    }

    Ok(())
}
