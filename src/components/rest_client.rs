use crate::engine::NodeHandler;
use crate::types::{CommonConfig, Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct RestClientConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub timeout_ms: Option<u64>,
    pub success_branch: Option<String>, // 成功分支名称
    pub error_branch: Option<String>,   // 失败分支名称
    #[serde(flatten)]
    pub common: CommonConfig,
}

impl Default for RestClientConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost".to_string(),
            method: "GET".to_string(),
            headers: None,
            timeout_ms: None,
            success_branch: None,
            error_branch: None,
            common: CommonConfig {
                node_type: NodeType::Middle,
            },
        }
    }
}

pub struct RestClientNode {
    config: RestClientConfig,
    client: Client,
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

        let mut request = self
            .client
            .request(self.config.method.parse().unwrap(), &url);

        // 添加请求头
        if let Some(headers) = &self.config.headers {
            for (key, value) in headers {
                request = request.header(key, value);
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

        // 检查状态码和错误响应
        if !status.is_success() || body.get("error").is_some() {
            let error_msg = if let Some(error) = body.get("error") {
                error["message"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string()
            } else {
                format!("HTTP请求返回错误状态码: {}", status)
            };
            return Err(RuleError::NodeExecutionError(error_msg));
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
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        let mut msg = msg;

        // 发送请求并处理结果
        match self.make_request(&msg).await {
            Ok(response_data) => {
                println!("请求成功: {:?}", response_data);
                // 请求成功
                msg.data = response_data;
                msg.msg_type = "http_response".to_string();

                // 设置成功分支
                if let Some(branch) = &self.config.success_branch {
                    msg.metadata.insert("branch_name".into(), branch.clone());
                }

                // 发送到成功分支的下一个节点
                ctx.send_next(msg.clone()).await?;
                Ok(msg)
            }
            Err(e) => {
                println!("请求失败: {:?}", e);
                // 请求失败
                msg.metadata.insert("error".into(), e.to_string());

                // 设置失败分支
                if let Some(branch) = &self.config.error_branch {
                    msg.metadata.insert("branch_name".into(), branch.clone());
                }

                // 发送到失败分支的下一个节点
                ctx.send_next(msg.clone()).await?;
                Ok(msg)
            }
        }
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "rest_client".to_string(),
            name: "HTTP客户端".to_string(),
            description: "发送HTTP请求,支持成功/失败分支路由".to_string(),
        }
    }
}
