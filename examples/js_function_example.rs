use rule_rs::{Message, RuleEngine};
use serde_json::json;
use tracing::{info, Level};

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "JS函数示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "js_function",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "functions": {
                    "getBMICategory": "if (msg < 18.5) return '偏瘦'; if (msg < 24) return '正常'; if (msg < 28) return '偏胖'; return '肥胖'; ",
                    "calculateBMI": "const height = msg.data.height / 100; const weight = msg.data.weight; const bmi = weight / (height * height); const category = getBMICategory(bmi); return { bmi: bmi.toFixed(2), category: category };"
                },
                "main": "calculateBMI"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "BMI计算结果: ${msg.data.bmi}, 身体状态: ${msg.data.category}"
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let engine = RuleEngine::new().await;
    engine.load_chain(RULE_CHAIN).await?;

    // 测试 BMI 计算
    let test_cases = vec![
        (170, 55), // 偏瘦
        (170, 65), // 正常
        (170, 75), // 偏胖
        (170, 85), // 肥胖
    ];

    for (height, weight) in test_cases {
        let msg = Message::new(
            "bmi_calc",
            json!({
                "height": height,
                "weight": weight
            }),
        );

        info!("测试身高{}cm, 体重{}kg的BMI", height, weight);
        match engine.process_msg(msg).await {
            Ok(_) => info!("BMI计算完成"),
            Err(e) => info!("计算失败: {:?}", e),
        }
    }

    Ok(())
}
