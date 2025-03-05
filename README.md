English| [简体中文](README_ZH.md)

If you like it or find it useful, please give it a star.

# rule-rs

A lightweight rule engine implemented in Rust that supports asynchronous execution, component extension, and rule chain orchestration.

## Key Features

- Asynchronous execution engine
- Rich built-in components
- Custom component extension support
- Nested sub-rule chain support
- AOP interceptor support
- Rule chain hot-reloading
- REST API service support

## Node Types

Nodes in rule chains are divided into three types:

| Type   | Description                           | Restriction                           |
| ------ | ------------------------------------- | ------------------------------------- |
| Head   | Head nodes (start/delay/schedule)     | Cannot be pointed to by other nodes  |
| Middle | Intermediate processing nodes         | No special restrictions              |
| Tail   | Tail nodes (like log)                | Cannot point to other nodes          |

## Rule Chain Specifications

1. Rule chains must start with a Head type node (usually a start node)
2. Head nodes cannot be pointed to by other nodes
3. Tail nodes cannot point to other nodes
4. Circular dependencies are not allowed

## Built-in Components

| Component Type | Description      | Node Type | Example Configuration                   |
| ------------- | ---------------- | --------- | -------------------------------------- |
| start         | Start node      | Head      | `{}`                                   |
| delay         | Delay process   | Head      | `{"delay_ms": 1000}`                   |
| schedule      | Scheduled task  | Head      | `{"cron": "*/5 * * * * *"}`            |
| fork          | Branch node     | Head      | `{}`                                   |
| join          | Merge node      | Tail      | `{}`                                   |
| log           | Log output      | Tail      | `{"template": "${msg.data}"}`          |
| script        | JS script       | Middle    | `{"script": "return msg.data;"}`       |
| filter        | Message filter  | Middle    | `{"condition": "value > 10"}`          |
| transform     | Data transform  | Middle    | `{"template": {"key": "${msg.value}"}}` |
| transform_js  | JS transform    | Middle    | `{"script": "return {...msg};"}`       |
| rest_client   | HTTP request    | Middle    | `{"url": "http://api.example.com"}`    |
| subchain      | Sub rule chain  | Middle    | `{"chain_id": "..."}`                  |

## Quick Start

### 1. Create Rule Chain

```rust
use rule_rs::{Message, RuleEngine};

#[tokio::main]
async fn main() {
    // Create engine instance
    let engine = RuleEngine::new().await;
    
    // Load rule chain
    let chain_id1 = engine.load_chain(r#"{
        "id": "00000000-0000-0000-0000-000000000000",
        "name": "Example Rule Chain",
        "root": true,
        "nodes": [
            {
                "id": "11111111-1111-1111-1111-111111111111",
                "type_name": "start",
                "chain_id": "00000000-0000-0000-0000-000000000000",
                "config": {},
                "layout": { "x": 50, "y": 100 }
            },
            {
                "id": "22222222-2222-2222-2222-222222222222", 
                "type_name": "log",
                "chain_id": "00000000-0000-0000-0000-000000000000",
                "config": {
                    "template": "Received message: ${msg.data}"
                },
                "layout": { "x": 200, "y": 100 }
            }
        ],
        "connections": [
            {
                "from_id": "11111111-1111-1111-1111-111111111111",
                "to_id": "22222222-2222-2222-2222-222222222222",
                "type_name": "success"
            }
        ],
        "metadata": {
            "version": 1,
            "created_at": 1679800000,
            "updated_at": 1679800000
        }
    }"#).await?;

    let msg = Message::new("test", json!({"value": 1}));
    engine.process_msg(chain_id1, msg).await?; // Ignore error result
}
```

## Component Development Guide

### 1. Define Component Configuration

```rust
#[derive(Debug, Deserialize)]
pub struct CustomConfig {
    pub param1: String,
    pub param2: i32,
}
impl Default for CustomConfig {
    fn default() -> Self {
        Self {}
    }
}
```

### 2. Implement Component Logic

```rust
#[derive(Debug)]
pub struct CustomNode {
    #[allow(dead_code)]
    config: CustomConfig,
}

impl CustomNode {
    pub fn new(config: UpperConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl NodeHandler for CustomNode {
    async fn handle(&'a self, ctx: NodeContext<'_>, msg: Message) -> Result<Message, RuleError> {
        // 1. Get configuration parameters
        let param1 = self.config.param1;
        let param2 = self.config.param2;

        // 2. Process message
        let new_msg = process_message(msg, param1, param2)?;

        // 3. Send to next node
        ctx.send_next(new_msg.clone()).await?;
        
        Ok(new_msg)
    }

    fn get_descriptor(&self) -> NodeDescriptor {
        NodeDescriptor {
            type_name: "custom/type".to_string(),
            name: "Custom Node".to_string(),
            description: "This is a custom processing node".to_string(),
            node_type: NodeType::Middle,
        }
    }
}
```

### 3. Register Component

```rust
engine.register_node_type("custom/type", Arc::new(|config| {
    Ok(Arc::new(CustomNode::new(config)) as Arc<dyn NodeHandler>)
})).await;
```

## Examples

The project includes multiple complete examples:

- examples/simple_rule - Basic rule chain example
- examples/custom_component - Custom case conversion component example
- examples/filter_example - Filter example
- examples/transform_example - Data transformation example
- examples/delay_example - Delay processing example
- examples/schedule_example - Scheduled task example
- examples/rest_client - HTTP request example
- examples/weather_service - Custom weather service component example
- examples/redis_example - Redis custom component example
- examples/aop_example - AOP interceptor example
- examples/subchain_example - Sub rule chain example
- examples/circular_chain - Circular dependency example
- examples/circular_subchain - Circular dependency sub rule chain example
- examples/circular_three_chains - Circular dependency three chains example

## Best Practices

1. Rule Chain Design
   - Each rule chain must start with a header node
   - Use branch and merge nodes appropriately to control flow
   - Avoid deep node nesting

2. Component Development
   - Follow single responsibility principle
   - Handle error cases properly
   - Provide clear configuration parameter documentation

3. Performance Optimization
   - Use async operations for I/O
   - Avoid repeated calculations
   - Use caching appropriately

## Documentation

For more detailed documentation, please refer to:

- [Architecture Design](docs/architecture.md)
- [Component Development Guide](docs/component.md)
- [API Documentation](docs/api.md)

## Acknowledgments

Thanks to the following projects and libraries for inspiring and helping rule-rs:

- [rulego](https://github.com/rulego/rulego)

## License

MIT License 