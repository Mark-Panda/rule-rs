use async_trait::async_trait;
use reqwest::Client;
use rule_rs;
use rule_rs::engine::NodeHandler;
use rule_rs::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use rule_rs::{engine::rule::RuleEngineTrait, RuleEngine};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info, Level};
// 组件配置
#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub api_key: String,
    pub city: String,
    pub language: String,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            api_key: "demo".to_string(),
            city: String::new(),
            language: "zh".to_string(),
        }
    }
}

// 组件实现
#[derive(Debug)]
pub struct WeatherNode {
    config: WeatherConfig,
    client: Client,
}

impl WeatherNode {
    pub fn new(config: WeatherConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    async fn get_weather(&self, city: &str) -> Result<WeatherInfo, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.weatherapi.com/v1/current.json?key={}&q={}&lang={}",
            self.config.api_key, city, self.config.language
        );

        debug!("Requesting weather data for city: {}", city);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await?;
            error!(
                "Weather API error - status: {}, body: {}",
                status, error_text
            );
            return Err(format!("API error: {} - {}", status, error_text).into());
        }

        let data: serde_json::Value = resp.json().await?;
        debug!("Weather API response: {:?}", data);

        // 检查错误响应
        if let Some(error) = data.get("error") {
            let error_msg = error["message"].as_str().unwrap_or("Unknown error");
            error!("Weather API returned error: {}", error_msg);
            return Err(error_msg.into());
        }

        // 解析返回数据
        let current = data
            .get("current")
            .and_then(|v| v.as_object())
            .ok_or("Missing 'current' field in response")?;

        let location = data
            .get("location")
            .and_then(|v| v.as_object())
            .ok_or("Missing 'location' field in response")?;

        let weather = WeatherInfo {
            city: location["name"]
                .as_str()
                .ok_or("Missing city name")?
                .to_string(),
            temperature: format!(
                "{}°C",
                current["temp_c"].as_f64().ok_or("Invalid temperature")?
            ),
            condition: current
                .get("condition")
                .and_then(|c| c["text"].as_str())
                .ok_or("Missing weather condition")?
                .to_string(),
            humidity: format!(
                "{}%",
                current["humidity"].as_i64().ok_or("Invalid humidity")?
            ),
            last_updated: current["last_updated"]
                .as_str()
                .ok_or("Missing update time")?
                .to_string(),
        };

        info!("Successfully fetched weather data: {:?}", weather);
        Ok(weather)
    }
}

#[derive(Debug, Serialize)]
struct WeatherInfo {
    city: String,
    temperature: String,
    condition: String,
    humidity: String,
    last_updated: String,
}

#[async_trait]
impl NodeHandler for WeatherNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        // 从消息中获取城市名称，如果没有则使用配置中的默认城市
        let city = msg
            .data
            .get("city")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.config.city);

        debug!("Processing weather request for city: {}", city);

        // 调用天气 API
        let weather = self.get_weather(city).await.map_err(|e| {
            error!("Weather API error: {}", e);
            RuleError::NodeExecutionError(e.to_string())
        })?;

        // 构造返回消息
        let mut new_msg = msg;
        new_msg.msg_type = "weather_info".to_string();
        new_msg.data = serde_json::json!({
            "城市": weather.city,
            "温度": weather.temperature,
            "天气": weather.condition,
            "湿度": weather.humidity,
            "更新时间": weather.last_updated,
        });

        // 发送到下一个节点
        ctx.send_next(new_msg.clone()).await?;

        info!("Weather node processed successfully: {:?}", new_msg);
        Ok(new_msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "custom/weather".to_string(),
            name: "天气服务".to_string(),
            description: "获取指定城市的天气信息".to_string(),
            node_type: NodeType::Middle,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例并等待组件注册完成
    let engine = RuleEngine::new().await;

    // 注册自定义组件
    engine
        .register_node_type(
            "custom/weather",
            Arc::new(|config| {
                if config.is_object() && config.as_object().unwrap().is_empty() {
                    Ok(Arc::new(WeatherNode::new(WeatherConfig::default()))
                        as Arc<dyn NodeHandler>)
                } else {
                    let config: WeatherConfig = serde_json::from_value(config)?;
                    Ok(Arc::new(WeatherNode::new(config)) as Arc<dyn NodeHandler>)
                }
            }),
        )
        .await;

    info!("已注册的组件:");
    for desc in engine.get_registered_components().await {
        info!("- {}: {}", desc.type_name, desc.description);
    }

    // 加载包含天气节点的规则链
    let chain_json = r#"{
        "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
        "name": "天气服务测试链",
        "root": true,
        "nodes": [
            {
                "id": "00000000-0000-0000-0000-000000000000",
                "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
                "type_name": "start",
                "config": {},
                "layout": { "x": 50, "y": 100 }
            },
            {
                "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "custom/weather",
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
                "from_id": "00000000-0000-0000-0000-000000000000",
                "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
                "type_name": "success"
            },
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

    let chain_id = engine.load_chain(chain_json).await?;

    // 处理消息
    let msg = Message::new(
        "query",
        json!({
            "city": "Shanghai",
            "aqi": "no"
        }),
    );

    info!("开始处理消息: {:?}", msg);
    let result = engine.process_msg(chain_id, msg).await?;
    info!("处理结果: {:?}", result);

    Ok(())
}
