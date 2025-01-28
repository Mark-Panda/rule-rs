# RuleGo 规则引擎 Rust 实现

一个基于 Rust 实现的轻量级规则引擎，支持灵活的规则链配置和丰富的内置组件。

## 功能特点

- 支持异步执行
- 内置多种组件
- 支持子规则链
- 支持循环依赖检测
- 支持版本管理
- 支持 AOP 拦截器

## 内置组件

| 组件类型 | 说明 | 配置示例 |
|---------|------|---------|
| log | 日志记录 | `{"template": "日志内容"}` |
| script | JavaScript脚本 | `{"script": "return msg;"}` |
| transform | 数据转换 | `{"template": {"key": "${msg.value}"}}` |
| transform_js | JS数据转换 | `{"script": "return msg;"}` |
| filter | 消息过滤 | `{"condition": "value < 10"}` |
| delay | 延时处理 | `{"delay_ms": 1000}` |
| switch | 条件分支 | `{"cases": [], "default": null}` |
| rest_client | REST调用 | `{"url": "http://api.example.com"}` |
| weather | 天气服务 | `{"api_key": "xxx", "city": "Shanghai"}` |
| subchain | 子规则链 | `{"chain_id": "xxx"}` |

## 使用示例

### 1. 基础规则链

```rust
// 创建规则引擎实例
let engine = RuleEngine::new().await;

// 加载规则链
let chain_json = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "示例规则链",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "script",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "script": "return { value: msg.data.value + 1 };"
            }
        }
    ]
}"#;

engine.load_chain(chain_json).await?;

// 处理消息
let msg = Message::new("test", json!({ "value": 1 }));
let result = engine.process_msg(msg).await?;
```

### 2. 天气服务示例

```rust
let chain_json = r#"{
    "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
    "name": "天气查询系统",
    "root": true,
    "nodes": [
        {
            "id": "3f2504e0-4f89-11d3-9a0c-0305e82c3302",
            "type_name": "weather",
            "chain_id": "3f2504e0-4f89-11d3-9a0c-0305e82c3301",
            "config": {
                "api_key": "your_api_key",
                "city": "Shanghai"
            }
        }
    ]
}"#;
```

## 高级特性

### 循环依赖检测

系统会自动检测规则链之间的循环依赖，包括:
- 节点间循环依赖
- 子规则链循环依赖

### 版本管理

- 支持规则链版本控制
- 记录更新时间戳
- 支持版本回滚(计划中)

### 拦截器系统

支持两种拦截器:
- 消息拦截器(MessageInterceptor)
- 节点拦截器(NodeInterceptor)

## 开发计划

- [ ] 规则链热更新
- [ ] 配置持久化
- [ ] 监控指标
- [ ] 可视化编辑器

## 贡献指南

欢迎提交 Issue 和 Pull Request。

## 许可证

MIT License
