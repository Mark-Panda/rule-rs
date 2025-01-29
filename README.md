# rule-rs

一个基于 Rust 实现的轻量级规则引擎,支持异步执行、组件扩展和规则链编排。

## 主要特性

- 异步执行引擎
- 丰富的内置组件
- 支持自定义组件扩展
- 支持子规则链嵌套
- 支持 AOP 拦截器
- 支持规则链热加载
- 支持 REST API 服务

## 内置组件

| 组件类型 | 说明 | 示例配置 |
|---------|------|---------|
| log | 日志输出 | `{"template": "${msg.data}"}` |
| script | JS脚本 | `{"script": "return msg.data;"}` |
| filter | 消息过滤 | `{"condition": "value > 10"}` |
| transform | 数据转换 | `{"template": {"key": "${msg.value}"}}` |
| transform_js | JS转换 | `{"script": "return {...msg};"}` |
| delay | 延时处理 | `{"delay_ms": 1000}` |
| schedule | 定时任务 | `{"cron": "*/5 * * * * *"}` |
| rest_client | HTTP请求 | `{"url": "http://api.example.com"}` |
| weather | 天气服务 | `{"city": "Shanghai"}` |
| subchain | 子规则链 | `{"chain_id": "..."}` |

## 快速开始

### 1. 添加依赖

```toml
[dependencies]
rule-rs = "0.1.0"
```

### 2. 创建规则链

```rust
use rule_rs::{Message, RuleEngine};

#[tokio::main]
async fn main() {
    // 创建引擎实例
    let engine = RuleEngine::new().await;
    
    // 加载规则链
    engine.load_chain(r#"{
        "id": "...",
        "name": "示例规则链",
        "root": true,
        "nodes": [
            {
                "id": "...", 
                "type_name": "log",
                "config": {
                    "template": "收到消息: ${msg.data}"
                }
            }
        ]
    }"#).await?;

    // 处理消息
    let msg = Message::new("test", json!({"value": 100}));
    engine.process_msg(msg).await?;
}
```

### 3. 自定义组件

```rust
use async_trait::async_trait;
use rule_rs::engine::NodeHandler;

pub struct CustomNode {
    config: CustomConfig,
}

#[async_trait]
impl NodeHandler for CustomNode {
    async fn handle(&self, ctx: NodeContext<'_>, msg: Message) -> Result<Message, RuleError> {
        // 处理消息
        Ok(msg)
    }
}

// 注册组件
engine.register_node_type("custom/type", Arc::new(|config| {
    Ok(Arc::new(CustomNode::new(config)) as Arc<dyn NodeHandler>)
})).await;
```

## 示例代码

项目包含多个完整的示例:

- examples/simple_rule.rs - 基础规则链示例
- examples/custom_component.rs - 自定义组件示例  
- examples/filter_example.rs - 过滤器示例
- examples/transform_example.rs - 数据转换示例
- examples/delay_example.rs - 延时处理示例
- examples/schedule_example.rs - 定时任务示例
- examples/rest_client.rs - HTTP请求示例
- examples/weather_service.rs - 天气服务示例

## 文档

更多详细文档请参考:

- [架构设计](docs/architecture.md)
- [组件开发指南](docs/component.md) 
- [API文档](docs/api.md)

## License

MIT License
