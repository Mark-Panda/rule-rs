use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

pub struct RestClientNode {
    pub(crate) config: RestClientConfig,
    client: Client,
}

#[derive(Deserialize)]
pub struct RestClientConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<serde_json::Map<String, Value>>,
    pub timeout_ms: Option<u64>,
    pub output_type: Option<String>,
}

impl RestClientNode {
    pub fn new(config: RestClientConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms.unwrap_or(5000)))
            .build()
            .unwrap();

        Self { config, client }
    }

    async fn make_request(&self, msg: &Message) -> Result<Value, RuleError> {
        // 替换 URL 中的变量
        let url = if self.config.url.contains("${") {
            let mut url = self.config.url.clone();
            if let Some(obj) = msg.data.as_object() {
                for (key, value) in obj {
                    let placeholder = format!("${{{}}}", key);
                    if url.contains(&placeholder) {
                        url = url.replace(&placeholder, &value.to_string());
                    }
                }
            }
            url
        } else {
            self.config.url.clone()
        };

        let mut request = match self.config.method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => {
                return Err(RuleError::NodeExecutionError(
                    "不支持的 HTTP 方法".to_string(),
                ))
            }
        };

        // 添加请求头
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                if let Some(value_str) = value.as_str() {
                    request = request.header(key, value_str);
                }
            }
        }

        // 对于 POST/PUT 请求，添加消息数据作为请求体
        if ["POST", "PUT"].contains(&self.config.method.to_uppercase().as_str()) {
            request = request.json(&msg.data);
        }

        // 发送请求
        let response = request
            .send()
            .await
            .map_err(|e| RuleError::NodeExecutionError(format!("HTTP请求失败: {}", e)))?;

        // 解析响应
        let status = response.status();
        let body = response
            .json::<Value>()
            .await
            .map_err(|e| RuleError::NodeExecutionError(format!("响应解析失败: {}", e)))?;

        // 检查状态码
        if !status.is_success() {
            return Err(RuleError::NodeExecutionError(format!(
                "HTTP请求返回错误状态码: {}",
                status
            )));
        }

        // 构造响应数据
        Ok(serde_json::json!({
            "status": status.as_u16(),
            "body": body,
        }))
    }
}

#[async_trait]
impl NodeHandler for RestClientNode {
    async fn handle(&self, _ctx: NodeContext, msg: Message) -> Result<Message, RuleError> {
        let response_data = self.make_request(&msg).await?;

        Ok(Message {
            id: msg.id,
            msg_type: self
                .config
                .output_type
                .clone()
                .unwrap_or_else(|| "http_response".to_string()),
            metadata: msg.metadata,
            data: response_data,
            timestamp: msg.timestamp,
        })
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "rest_client".to_string(),
            name: "REST客户端".to_string(),
            description: "发送HTTP请求".to_string(),
        }
    }
}
