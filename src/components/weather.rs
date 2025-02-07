use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

// 组件配置
#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub api_key: String,
    pub city: String,
    pub language: String,
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            api_key: "demo".to_string(),
            city: String::new(),
            language: "zh".to_string(),
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

// 组件实现
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
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
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
            type_name: "weather".to_string(),
            name: "天气服务".to_string(),
            description: "获取指定城市的天气信息".to_string(),
        }
    }
}
