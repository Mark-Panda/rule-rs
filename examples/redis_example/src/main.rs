use async_trait::async_trait;
use redis::{cmd, AsyncCommands, Client, RedisResult, Value as RedisValue};
use rule_rs;
use rule_rs::engine::NodeHandler;
use rule_rs::types::{Message, NodeContext, NodeDescriptor, NodeType, RuleError};
use rule_rs::{engine::rule::RuleEngineTrait, RuleEngine};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber;

const RULE_CHAIN: &str = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "Redis Hash JSON示例",
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
            "type_name": "custom/redis",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "url": "redis://localhost:6379",
                "operation": {
                    "type": "Command",
                    "config": "HSET"
                },
                "key": "user:1",
                "field": "profile",
                "value": "{\"name\":\"张三\",\"age\":25,\"email\":\"zhangsan@example.com\"}",
                "success_branch": "get",
                "error_branch": "error"
            },
            "layout": { "x": 100, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "custom/redis",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "url": "redis://localhost:6379",
                "operation": {
                    "type": "Command",
                    "config": "HGET"
                },
                "key": "user:1",
                "field": "profile",
                "success_branch": "log",
                "error_branch": "error"
            },
            "layout": { "x": 300, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "template": "用户信息: ${msg.data.value}",
                "common": {
                    "node_type": "Tail"
                }
            },
            "layout": { "x": 500, "y": 100 }
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
            "type_name": "get"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log"
        }
    ],
    "metadata": {
        "version": 1,
        "created_at": 1679800000,
        "updated_at": 1679800000
    }
}"#;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum RedisOperation {
    Command(RedisCommand),
    Raw { command: String, args: Vec<String> },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum RedisCommand {
    GET,
    SET,
    DEL,
    EXISTS,
    INCR,
    DECR,
    EXPIRE,
    TTL,
    HGET,
    HSET,
    HDEL,
    HGETALL,
    LPUSH,
    RPUSH,
    LPOP,
    RPOP,
    LLEN,
    SADD,
    SREM,
    SMEMBERS,
    SCARD,
    ZADD,
    ZREM,
    ZRANGE,
    ZCARD,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    // Redis连接配置
    pub url: String,

    // 操作类型
    pub operation: RedisOperation,

    // 原有字段保持不变
    pub key: String,
    pub field: Option<String>,
    pub value: Option<String>,
    pub values: Option<Vec<String>>,
    pub score: Option<f64>,
    pub ttl: Option<u64>,
    pub start: Option<i64>,
    pub stop: Option<i64>,

    pub success_branch: Option<String>,
    pub error_branch: Option<String>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            operation: RedisOperation::Raw {
                command: "PING".to_string(),
                args: vec![],
            },
            key: String::new(),
            field: None,
            value: None,
            values: None,
            score: None,
            ttl: None,
            start: None,
            stop: None,
            success_branch: None,
            error_branch: None,
        }
    }
}

#[derive(Debug)]
pub struct RedisNode {
    config: RedisConfig,
    client: Client,
}

impl RedisNode {
    pub fn new(config: RedisConfig) -> Self {
        let client = Client::open(config.url.clone()).unwrap();

        Self { config, client }
    }

    // 添加辅助方法转换Redis值到JSON
    fn redis_value_to_json(value: RedisValue) -> serde_json::Value {
        match value {
            RedisValue::Nil => json!(null),
            RedisValue::Int(i) => json!(i),
            RedisValue::SimpleString(s) => json!(s),
            RedisValue::BulkString(bytes) => {
                if let Ok(s) = String::from_utf8(bytes) {
                    json!(s)
                } else {
                    json!("<binary data>")
                }
            }
            RedisValue::Array(values) => {
                json!(values
                    .into_iter()
                    .map(|v| Self::redis_value_to_json(v))
                    .collect::<Vec<_>>())
            }
            RedisValue::Map(map) => {
                let mut result = HashMap::new();
                for (k, v) in map {
                    let key = match k {
                        RedisValue::SimpleString(s) => s,
                        RedisValue::BulkString(bytes) => {
                            String::from_utf8(bytes).unwrap_or_else(|_| "<binary key>".to_string())
                        }
                        _ => format!("{:?}", k),
                    };
                    result.insert(key, Self::redis_value_to_json(v));
                }
                json!(result)
            }
            _ => json!(format!("{:?}", value)),
        }
    }
}

#[async_trait]
impl NodeHandler for RedisNode {
    async fn handle<'a>(
        &'a self,
        ctx: NodeContext<'a>,
        msg: Message,
    ) -> Result<Message, RuleError> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| RuleError::ComponentError(format!("获取Redis连接失败: {}", e)))?;

        let result = match &self.config.operation {
            RedisOperation::Command(cmd) => {
                match cmd {
                    // String操作
                    RedisCommand::GET => {
                        let value: Option<String> =
                            conn.get(&self.config.key).await.map_err(|e| {
                                RuleError::ComponentError(format!("Redis GET失败: {}", e))
                            })?;
                        info!("Redis GET {} = {:?}", self.config.key, value);
                        Ok(value.map(|v| json!({"value": v})))
                    }
                    RedisCommand::SET => {
                        if let Some(value) = &self.config.value {
                            if let Some(ttl) = self.config.ttl {
                                let _: () = conn
                                    .set_ex(&self.config.key, value, ttl)
                                    .await
                                    .map_err(|e| {
                                        RuleError::ComponentError(format!("Redis SETEX失败: {}", e))
                                    })?;
                            } else {
                                let _: () =
                                    conn.set(&self.config.key, value).await.map_err(|e| {
                                        RuleError::ComponentError(format!("Redis SET失败: {}", e))
                                    })?;
                            }
                            info!("Redis SET {} = {}", self.config.key, value);
                            Ok(Some(json!({"key": self.config.key, "value": value})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::DEL => {
                        let deleted: i64 = conn.del(&self.config.key).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis DEL失败: {}", e))
                        })?;
                        info!("Redis DEL {} = {}", self.config.key, deleted);
                        Ok(Some(json!({"deleted": deleted})))
                    }
                    RedisCommand::EXISTS => {
                        let exists: bool = conn.exists(&self.config.key).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis EXISTS失败: {}", e))
                        })?;
                        Ok(Some(json!({"exists": exists})))
                    }
                    RedisCommand::INCR => {
                        let value: i64 = conn.incr(&self.config.key, 1).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis INCR失败: {}", e))
                        })?;
                        Ok(Some(json!({"value": value})))
                    }
                    RedisCommand::DECR => {
                        let value: i64 = conn.decr(&self.config.key, 1).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis DECR失败: {}", e))
                        })?;
                        Ok(Some(json!({"value": value})))
                    }
                    RedisCommand::EXPIRE => {
                        if let Some(ttl) = self.config.ttl {
                            let _: bool =
                                conn.expire(&self.config.key, ttl as i64)
                                    .await
                                    .map_err(|e| {
                                        RuleError::ComponentError(format!(
                                            "Redis EXPIRE失败: {}",
                                            e
                                        ))
                                    })?;
                            Ok(Some(json!({"result": true})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::TTL => {
                        let ttl: i64 = conn.ttl(&self.config.key).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis TTL失败: {}", e))
                        })?;
                        Ok(Some(json!({"ttl": ttl})))
                    }

                    // Hash操作
                    RedisCommand::HGET => {
                        if let Some(field) = &self.config.field {
                            let value: RedisValue =
                                conn.hget(&self.config.key, field).await.map_err(|e| {
                                    RuleError::ComponentError(format!("Redis HGET失败: {}", e))
                                })?;
                            info!("Redis HGET {}:{} = {:?}", self.config.key, field, value);
                            Ok(Some(json!({
                                "field": field,
                                "value": Self::redis_value_to_json(value)
                            })))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::HSET => {
                        if let Some(field) = &self.config.field {
                            if let Some(value) = &self.config.value {
                                let _: bool = conn
                                    .hset(&self.config.key, field, value)
                                    .await
                                    .map_err(|e| {
                                        RuleError::ComponentError(format!("Redis HSET失败: {}", e))
                                    })?;
                                Ok(Some(json!({"field": field, "value": value})))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::HDEL => {
                        if let Some(field) = &self.config.field {
                            let deleted: i64 =
                                conn.hdel(&self.config.key, field).await.map_err(|e| {
                                    RuleError::ComponentError(format!("Redis HDEL失败: {}", e))
                                })?;
                            Ok(Some(json!({"deleted": deleted})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::HGETALL => {
                        let values: HashMap<String, String> =
                            conn.hgetall(&self.config.key).await.map_err(|e| {
                                RuleError::ComponentError(format!("Redis HGETALL失败: {}", e))
                            })?;
                        Ok(Some(json!({"values": values})))
                    }

                    // List操作
                    RedisCommand::LPUSH => {
                        if let Some(value) = &self.config.value {
                            let _: i64 =
                                conn.lpush(&self.config.key, value).await.map_err(|e| {
                                    RuleError::ComponentError(format!("Redis LPUSH失败: {}", e))
                                })?;
                            Ok(Some(json!({"length": 1})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::RPUSH => {
                        if let Some(value) = &self.config.value {
                            let _: i64 =
                                conn.rpush(&self.config.key, value).await.map_err(|e| {
                                    RuleError::ComponentError(format!("Redis RPUSH失败: {}", e))
                                })?;
                            Ok(Some(json!({"length": 1})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::LPOP | RedisCommand::RPOP => {
                        let value: Option<String> = if matches!(cmd, RedisCommand::LPOP) {
                            conn.lpop(&self.config.key, None)
                        } else {
                            conn.rpop(&self.config.key, None)
                        }
                        .await
                        .map_err(|e| RuleError::ComponentError(format!("Redis POP失败: {}", e)))?;
                        Ok(value.map(|v| json!({"value": v})))
                    }
                    RedisCommand::LLEN => {
                        let len: i64 = conn.llen(&self.config.key).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis LLEN失败: {}", e))
                        })?;
                        Ok(Some(json!({"length": len})))
                    }

                    // Set操作
                    RedisCommand::SADD => {
                        if let Some(values) = &self.config.values {
                            let added: i64 = conn
                                .sadd(&self.config.key, values.as_slice())
                                .await
                                .map_err(|e| {
                                RuleError::ComponentError(format!("Redis SADD失败: {}", e))
                            })?;
                            Ok(Some(json!({"added": added})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::SREM => {
                        if let Some(values) = &self.config.values {
                            let removed: i64 = conn
                                .srem(&self.config.key, values.as_slice())
                                .await
                                .map_err(|e| {
                                    RuleError::ComponentError(format!("Redis SREM失败: {}", e))
                                })?;
                            Ok(Some(json!({"removed": removed})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::SMEMBERS => {
                        let members: Vec<String> =
                            conn.smembers(&self.config.key).await.map_err(|e| {
                                RuleError::ComponentError(format!("Redis SMEMBERS失败: {}", e))
                            })?;
                        Ok(Some(json!({"members": members})))
                    }
                    RedisCommand::SCARD => {
                        let count: i64 = conn.scard(&self.config.key).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis SCARD失败: {}", e))
                        })?;
                        Ok(Some(json!({"count": count})))
                    }

                    // Sorted Set操作
                    RedisCommand::ZADD => {
                        if let Some(score) = self.config.score {
                            if let Some(value) = &self.config.value {
                                let added: i64 = conn
                                    .zadd(&self.config.key, value, score)
                                    .await
                                    .map_err(|e| {
                                        RuleError::ComponentError(format!("Redis ZADD失败: {}", e))
                                    })?;
                                Ok(Some(json!({"added": added})))
                            } else {
                                Ok(None)
                            }
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::ZREM => {
                        if let Some(value) = &self.config.value {
                            let removed: i64 =
                                conn.zrem(&self.config.key, value).await.map_err(|e| {
                                    RuleError::ComponentError(format!("Redis ZREM失败: {}", e))
                                })?;
                            Ok(Some(json!({"removed": removed})))
                        } else {
                            Ok(None)
                        }
                    }
                    RedisCommand::ZRANGE => {
                        let start = self.config.start.unwrap_or(0);
                        let stop = self.config.stop.unwrap_or(-1);
                        let members: Vec<(String, f64)> = conn
                            .zrange_withscores(
                                &self.config.key,
                                start.try_into().unwrap_or(0),
                                stop.try_into().unwrap_or(-1),
                            )
                            .await
                            .map_err(|e| {
                                RuleError::ComponentError(format!("Redis ZRANGE失败: {}", e))
                            })?;
                        Ok(Some(json!({
                            "members": members.into_iter().map(|(m, s)| json!({
                                "member": m,
                                "score": s
                            })).collect::<Vec<_>>()
                        })))
                    }
                    RedisCommand::ZCARD => {
                        let count: i64 = conn.zcard(&self.config.key).await.map_err(|e| {
                            RuleError::ComponentError(format!("Redis ZCARD失败: {}", e))
                        })?;
                        Ok(Some(json!({"count": count})))
                    }
                }
            }
            RedisOperation::Raw { command, args } => {
                // 直接执行Redis命令
                let mut redis_cmd = cmd(command);
                for arg in args {
                    redis_cmd.arg(arg);
                }

                let value: RedisResult<RedisValue> = redis_cmd.query_async(&mut conn).await;
                match value {
                    Ok(v) => {
                        info!("Redis {} {:?} = {:?}", command, args, v);
                        Ok(Some(Self::redis_value_to_json(v)))
                    }
                    Err(e) => Err(RuleError::ComponentError(format!(
                        "Redis命令执行失败: {} {:?} - {}",
                        command, args, e
                    ))),
                }
            }
        }?;

        // 构造返回消息
        let mut new_msg = msg;
        if let Some(value) = result {
            new_msg.data = value;
            // 设置成功分支
            if let Some(branch) = &self.config.success_branch {
                new_msg
                    .metadata
                    .insert("branch_name".into(), branch.clone());
            }
        } else {
            // 设置失败分支
            if let Some(branch) = &self.config.error_branch {
                new_msg
                    .metadata
                    .insert("branch_name".into(), branch.clone());
            }
        }

        // 发送到下一个节点
        ctx.send_next(new_msg.clone()).await?;

        Ok(new_msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "custom/redis".to_string(),
            name: "Redis客户端".to_string(),
            description: "执行Redis命令".to_string(),
            node_type: NodeType::Middle,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    // 创建引擎实例
    let engine = RuleEngine::new().await;

    // 注册自定义组件
    engine
        .register_node_type(
            "custom/redis",
            Arc::new(|config| {
                if config.is_object() && config.as_object().unwrap().is_empty() {
                    Ok(Arc::new(RedisNode::new(RedisConfig::default())) as Arc<dyn NodeHandler>)
                } else {
                    let config: RedisConfig = serde_json::from_value(config)?;
                    Ok(Arc::new(RedisNode::new(config)) as Arc<dyn NodeHandler>)
                }
            }),
        )
        .await;

    info!("已注册的组件:");
    for desc in engine.get_registered_components().await {
        info!("- {}: {}", desc.type_name, desc.description);
    }

    // 加载规则链
    let chain_id = engine.load_chain(RULE_CHAIN).await?;

    info!(
        "规则链加载成功, 版本: {}",
        engine.get_current_version().await
    );

    // 创建测试消息
    let msg = Message::new(
        "test",
        json!({
            "value": "hello redis"
        }),
    );

    // 处理消息
    match engine.process_msg(chain_id, msg).await {
        Ok(result) => info!("处理结果: {:?}", result),
        Err(e) => info!("处理失败: {:?}", e),
    }

    Ok(())
}
