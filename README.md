# RuleGo-RS

RuleGo-RS 是一个用 Rust 实现的规则引擎，支持自定义组件、AOP 拦截、消息流转等功能。

## 目录

- [快速开始](#快速开始)
- [规则链配置](#规则链配置)
- [自定义组件](#自定义组件)
- [AOP 拦截器](#aop-拦截器)
- [内置组件](#内置组件)

## 快速开始

1. 创建规则链配置:

```json
{
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
        },
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",
            "type_name": "log",
            "config": {
                "template": "处理结果: ${msg.value}"
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
}
```

2. 使用规则引擎:

```rust
use rulego_rs::{Message, RuleEngine};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = RuleEngine::new();
    
    // 加载规则链
    engine.load_chain(chain_json).await?;
    
    // 处理消息
    let msg = Message::new("test", json!({ "value": 1 }));
    let result = engine.process_msg(msg).await?;
    
    Ok(())
}
```

## 规则链配置

规则链由以下部分组成：
- `id`: 规则链唯一标识
- `name`: 规则链名称
- `root`: 是否为根规则链
- `nodes`: 节点列表
- `connections`: 节点间连接关系
- `metadata`: 元数据信息

### 节点配置
```json
{
    "id": "节点唯一ID",
    "type_name": "节点类型",
    "config": {
        // 节点特定配置
    },
    "layout": {
        "x": 100,
        "y": 100
    }
}
```

### 连接配置
```json
{
    "from_id": "源节点ID",
    "to_id": "目标节点ID", 
    "type_name": "连接类型"
}
```

## 自定义组件

1. 实现节点处理器:

```rust
use crate::engine::NodeHandler;
use crate::types::{Message, NodeContext, NodeDescriptor, RuleError};
use async_trait::async_trait;
use serde::Deserialize;

// 组件配置
#[derive(Debug, Deserialize)]
pub struct CustomConfig {
    pub param1: String,
    pub param2: i32,
}

// 组件实现
pub struct CustomNode {
    config: CustomConfig,
}

impl CustomNode {
    pub fn new(config: CustomConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for CustomNode {
    async fn handle<'a>(&self, ctx: NodeContext<'a>, msg: Message) -> Result<Message, RuleError> {
        // 实现节点处理逻辑
        Ok(msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "custom".to_string(),
            name: "自定义节点".to_string(),
            description: "这是一个自定义节点".to_string(),
        }
    }
}
```

2. 注册组件:

```rust
// 在 RuleEngine::new() 中注册
let factories: Vec<(&str, NodeFactory)> = vec![
    (
        "custom",
        Arc::new(|config| {
            let config: CustomConfig = serde_json::from_value(config)?;
            Ok(Arc::new(CustomNode::new(config)) as Arc<dyn NodeHandler>)
        }),
    ),
];
```

## AOP 拦截器

1. 节点拦截器:

```rust
use async_trait::async_trait;

#[async_trait]
impl NodeInterceptor for LoggingInterceptor {
    async fn before<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        // 节点执行前处理
        Ok(())
    }

    async fn after<'a>(&self, ctx: &NodeContext<'a>, msg: &Message) -> Result<(), RuleError> {
        // 节点执行后处理
        Ok(())
    }

    async fn error<'a>(&self, ctx: &NodeContext<'a>, error: &RuleError) -> Result<(), RuleError> {
        // 节点执行错误处理
        Ok(())
    }
}
```

2. 消息拦截器:

```rust
#[async_trait]
impl MessageInterceptor for MetricsInterceptor {
    async fn before_process(&self, msg: &Message) -> Result<(), RuleError> {
        // 消息处理前
        Ok(())
    }

    async fn after_process(&self, msg: &Message) -> Result<(), RuleError> {
        // 消息处理后
        Ok(())
    }
}
```

3. 注册拦截器:

```rust
// 注册节点拦截器
engine.add_node_interceptor(Arc::new(LoggingInterceptor)).await;

// 注册消息拦截器
engine.add_msg_interceptor(Arc::new(MetricsInterceptor)).await;
```

## 内置组件

### Script 节点
执行 JavaScript 脚本:
```json
{
    "type_name": "script",
    "config": {
        "script": "return { value: msg.data.value + 1 };"
    }
}
```

### Log 节点
输出日志信息:
```json
{
    "type_name": "log",
    "config": {
        "template": "消息内容: ${msg.value}"
    }
}
```

### Filter 节点
过滤消息:
```json
{
    "type_name": "filter",
    "config": {
        "condition": "msg.data.value > 10"
    }
}
```

### Transform 节点
转换消息:
```json
{
    "type_name": "transform",
    "config": {
        "fields": {
            "new_value": "${msg.value * 2}"
        }
    }
}
```

### REST Client 节点
调用 HTTP 接口:
```json
{
    "type_name": "rest_client",
    "config": {
        "url": "https://api.example.com/data",
        "method": "POST",
        "headers": {
            "Content-Type": "application/json"
        }
    }
}
```

## 许可证

MIT License

## 规则链配置详解

规则链配置采用 JSON 格式，包含以下主要部分：

### 规则链定义
```json
{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",  // UUID格式的唯一标识
    "name": "示例规则链",                            // 规则链名称
    "root": true,                                   // 是否为根规则链，每个系统只能有一个根规则链
    "nodes": [],                                    // 节点列表
    "connections": [],                              // 节点间连接关系
    "metadata": {}                                  // 元数据信息
}
```

### 节点配置
```json
{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",  // 节点唯一ID，UUID格式
    "type_name": "script",                          // 节点类型，对应已注册的组件类型
    "config": {                                     // 节点配置，根据不同类型有不同的配置项
        "param1": "value1",
        "param2": 123
    },
    "layout": {                                     // 节点布局信息，用于可视化展示
        "x": 100,                                   // X坐标
        "y": 100                                    // Y坐标
    }
}
```

### 连接配置
```json
{
    "from_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",  // 源节点ID
    "to_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3303",    // 目标节点ID
    "type_name": "success"                               // 连接类型，如：success/failure/custom
}
```

### 元数据配置
```json
{
    "version": 1,                // 规则链版本号
    "created_at": 1679800000,   // 创建时间戳
    "updated_at": 1679800000    // 更新时间戳
}
```

### 内置组件配置示例

#### Script 节点
```json
{
    "type_name": "script",
    "config": {
        "script": "return { value: msg.data.value + 1 };"  // JavaScript脚本
    }
}
```

#### Log 节点
```json
{
    "type_name": "log",
    "config": {
        "template": "消息内容: ${msg.value}",  // 日志模板，支持变量替换
        "level": "info"                       // 可选，日志级别：debug/info/warn/error
    }
}
```

#### Filter 节点
```json
{
    "type_name": "filter",
    "config": {
        "condition": "msg.data.value > 10",   // 过滤条件，JavaScript表达式
        "checkFields": ["value", "type"],     // 可选，需要检查的字段列表
        "strict": false                       // 可选，是否严格模式
    }
}
```

#### Transform 节点
```json
{
    "type_name": "transform",
    "config": {
        "fields": {                           // 字段转换映射
            "new_value": "${msg.value * 2}",  // 支持JavaScript表达式
            "timestamp": "${Date.now()}",
            "fixed_value": "constant"
        },
        "dropFields": ["temp1", "temp2"]      // 可选，需要删除的字段
    }
}
```

#### REST Client 节点
```json
{
    "type_name": "rest_client",
    "config": {
        "url": "https://api.example.com/data",         // 请求URL
        "method": "POST",                              // HTTP方法：GET/POST/PUT/DELETE
        "headers": {                                   // 请求头
            "Content-Type": "application/json",
            "Authorization": "Bearer ${msg.token}"     // 支持变量替换
        },
        "params": {                                    // URL参数
            "id": "${msg.id}",
            "type": "test"
        },
        "body": {                                      // 请求体
            "data": "${msg.data}",
            "timestamp": "${Date.now()}"
        },
        "timeout": 5000,                              // 超时时间(ms)
        "retry": {                                    // 重试配置
            "maxAttempts": 3,                         // 最大重试次数
            "delay": 1000                             // 重试间隔(ms)
        }
    }
}
```

#### Weather 节点 (自定义组件示例)
```json
{
    "type_name": "weather",
    "config": {
        "api_key": "your_api_key_here",    // API密钥
        "city": "Shanghai",                // 默认城市
        "language": "zh"                   // 返回语言：zh/en
    }
}
```

### 变量替换规则

配置中支持使用 `${expression}` 语法进行变量替换：

- `${msg.field}`: 访问消息字段
- `${msg.data.field}`: 访问消息数据中的字段
- `${ctx.field}`: 访问上下文变量
- `${Date.now()}`: 执行JavaScript表达式

### 连接类型说明

- `success`: 节点执行成功时的连接
- `failure`: 节点执行失败时的连接
- `custom`: 自定义连接类型，用于特定场景
