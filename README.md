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

## 节点类型

规则链中的节点分为三种类型:

| 类型   | 说明                            | 限制               |
| ------ | ------------------------------- | ------------------ |
| Head   | 头节点(如 start/delay/schedule) | 不能被其他节点指向 |
| Middle | 中间处理节点                    | 无特殊限制         |
| Tail   | 尾节点(如 log)                  | 不能指向其他节点   |

## 规则链规范

1. 规则链必须以 Head 类型节点开始(通常是 start 节点)
2. Head 节点不能被其他节点指向
3. Tail 节点不能指向其他节点
4. 不允许出现循环依赖

## 内置组件

| 组件类型     | 说明     | 节点类型 | 示例配置                                |
| ------------ | -------- | -------- | --------------------------------------- |
| start        | 起始节点 | Head     | `{}`                                    |
| fork         | 分支节点 | Head     | `{}`                                    |
| join         | 汇聚节点 | Tail     | `{}`                                    |
| log          | 日志输出 | Tail     | `{"template": "${msg.data}"}`           |
| script       | JS脚本   | Middle   | `{"script": "return msg.data;"}`        |
| filter       | 消息过滤 | Middle   | `{"condition": "value > 10"}`           |
| transform    | 数据转换 | Middle   | `{"template": {"key": "${msg.value}"}}` |
| transform_js | JS转换   | Middle   | `{"script": "return {...msg};"}`        |
| delay        | 延时处理 | Head     | `{"delay_ms": 1000}`                    |
| schedule     | 定时任务 | Head     | `{"cron": "*/5 * * * * *"}`             |
| rest_client  | HTTP请求 | Middle   | `{"url": "http://api.example.com"}`     |
| weather      | 天气服务 | Middle   | `{"city": "Shanghai"}`                  |
| subchain     | 子规则链 | Middle   | `{"chain_id": "..."}`                   |

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
                "id": "start-node",
                "type_name": "start",
                "config": {},
                "layout": { "x": 50, "y": 100 }
            },
            {
                "id": "log-node", 
                "type_name": "log",
                "config": {
                    "template": "收到消息: ${msg.data}"
                },
                "layout": { "x": 200, "y": 100 }
            }
        ],
        "connections": [
            {
                "from_id": "start-node",
                "to_id": "log-node",
                "type_name": "success"
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

- examples/simple_rule - 基础规则链示例
- examples/custom_component - 自定义大小写转换组件示例  
- examples/filter_example - 过滤器示例
- examples/transform_example - 数据转换示例
- examples/delay_example - 延时处理示例
- examples/schedule_example - 定时任务示例
- examples/rest_client - HTTP请求示例
- examples/weather_service - 自定义天气服务组件示例
- examples/redis_example - Redis自定义组件示例

## 规则链示例

### 1. 基础规则链 - 数据转换和日志

```json
{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "基础示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3300",
            "type_name": "start",
            "config": {},
            "layout": { "x": 50, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "transform",
            "config": {
                "template": {
                    "value": "${msg.data.value * 2}"
                }
            },
            "layout": { "x": 200, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "config": {
                "template": "转换结果: ${msg.data.value}"
            },
            "layout": { "x": 350, "y": 100 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3300",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "success"
        }
    ]
}
```

### 2. 分支处理 - Filter 节点

```json
{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "分支处理示例",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3300",
            "type_name": "start",
            "config": {},
            "layout": { "x": 50, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "filter",
            "config": {
                "condition": "msg.data.value > 10"
            },
            "layout": { "x": 200, "y": 100 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "config": {
                "template": "大于10: ${msg.data.value}"
            },
            "layout": { "x": 350, "y": 50 }
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "log",
            "config": {
                "template": "小于等于10: ${msg.data.value}"
            },
            "layout": { "x": 350, "y": 150 }
        }
    ],
    "connections": [
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3300",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "success"
        },
        {
            "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3304",
            "type_name": "failure"
        }
    ]
}
```

## 组件开发指南

### 1. 定义组件配置

```rust
#[derive(Debug, Deserialize)]
pub struct CustomConfig {
    pub param1: String,
    pub param2: i32,
}
```

### 2. 实现组件处理逻辑

```rust
#[derive(Debug)]
pub struct CustomNode {
    config: CustomConfig,
}

#[async_trait]
impl NodeHandler for CustomNode {
    async fn handle(&self, ctx: NodeContext<'_>, msg: Message) -> Result<Message, RuleError> {
        // 1. 获取配置参数
        let param1 = &self.config.param1;
        let param2 = self.config.param2;

        // 2. 处理消息
        let new_msg = process_message(msg, param1, param2)?;

        // 3. 发送到下一个节点
        ctx.send_next(new_msg.clone()).await?;
        
        Ok(new_msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "custom".to_string(),
            name: "自定义节点".to_string(),
            description: "这是一个自定义处理节点".to_string(),
            node_type: NodeType::Middle,
        }
    }
}
```

## 最佳实践

1. 规则链设计
   - 每个规则链都必须以 start 节点开始
   - 合理使用分支和汇聚节点控制流程
   - 避免过深的节点嵌套

2. 组件开发
   - 遵循单一职责原则
   - 合理处理错误情况
   - 提供清晰的配置参数说明

3. 性能优化
   - 使用异步操作处理 I/O
   - 避免重复计算
   - 合理使用缓存

## 文档

更多详细文档请参考:

- [架构设计](docs/architecture.md)
- [组件开发指南](docs/component.md) 
- [API文档](docs/api.md)

## License

MIT License