use regex::Regex;
use serde_json::Value;
use uuid::Uuid;

use crate::types::{Message, NodeContext};

/// 解析字符串中的占位符，例如：
/// - ${msg} 或 ${msg.data.user.name}
/// - ${<nodeId>.msg.id}，可选 scope：${<nodeId>.input.msg.id} 或 ${<nodeId>.output.msg.id}
/// 支持转义：\${...} 不进行替换，转为字面量 ${...}
pub fn resolve_placeholders_in_str(s: &str, ctx: &NodeContext, current_msg: &Message) -> String {
    // 先将被转义的占位符替换为不匹配主模式的掩码，避免被解析
    // 如: \${msg.id} -> §§MASK§§{msg.id}
    let mask_re = Regex::new(r"\\\$\{([^}]+)\}").unwrap();

    let mut result = String::from(s);
    result = mask_re.replace_all(&result, "§§MASK§§{$1}").into_owned();

    // Replace real placeholders
    // We iterate until no more matches to support multiple replacements
    let main_re = Regex::new(r"\$\{([^}]+)\}").unwrap();
    loop {
        let replaced = main_re.replace_all(&result, |caps: &regex::Captures| {
            let expr = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            match resolve_expr(expr, ctx, current_msg) {
                Some(v) => v,
                None => caps.get(0).unwrap().as_str().to_string(), // keep original
            }
        });
        let new_result = replaced.into_owned();
        if new_result == result {
            break;
        }
        result = new_result;
    }

    // 最后将掩码还原为字面量占位符: §§MASK§§${...} -> ${...}
    let unmask_re = Regex::new(r"§§MASK§§\{([^}]+)\}").unwrap();
    result = unmask_re
        .replace_all(&result, |caps: &regex::Captures| {
            let inner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            format!("${{{}}}", inner)
        })
        .into_owned();

    result
}

fn resolve_expr(expr: &str, _ctx: &NodeContext, current_msg: &Message) -> Option<String> {
    // expr forms:
    // - msg
    // - msg.id / msg.msg_type / msg.timestamp / msg.metadata.<k> / msg.data.<path>
    // - <uuid>.msg.<path> or <uuid>.input.msg.<path> or <uuid>.output.msg.<path>
    let parts: Vec<&str> = expr.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    if parts[0] == "msg" {
        return Some(get_from_message(current_msg, &parts[1..]));
    }

    // Try parse as UUID
    if let Ok(node_id) = Uuid::parse_str(parts[0]) {
        let mut idx = 1;
        let mut scope = "output"; // default
        if idx < parts.len() && (parts[idx] == "input" || parts[idx] == "output") {
            scope = parts[idx];
            idx += 1;
        }

        if idx >= parts.len() || parts[idx] != "msg" {
            return None;
        }
        idx += 1;
        let key = format!("node.{}.{}", node_id, scope);
        if let Some(raw) = current_msg.metadata.get(&key) {
            if let Ok(message) = serde_json::from_str::<Message>(raw) {
                return Some(get_from_message(&message, &parts[idx..]));
            }
        }
        return None;
    }

    None
}

fn get_from_message(message: &Message, path: &[&str]) -> String {
    if path.is_empty() {
        return serde_json::to_string(message).unwrap_or_else(|_| String::new());
    }
    match path[0] {
        "id" => message.id.to_string(),
        "msg_type" | "type" => message.msg_type.clone(),
        "timestamp" => message.timestamp.to_string(),
        "metadata" => {
            if path.len() >= 2 {
                let key = path[1];
                message
                    .metadata
                    .get(key)
                    .cloned()
                    .unwrap_or_default()
            } else {
                serde_json::to_string(&message.metadata).unwrap_or_default()
            }
        }
        "data" => get_from_json(&message.data, &path[1..]),
        _ => String::new(),
    }
}

fn get_from_json(value: &Value, path: &[&str]) -> String {
    fn value_to_text(v: &Value) -> String {
        match v {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            _ => v.to_string(),
        }
    }

    if path.is_empty() {
        return value_to_text(value);
    }
    let mut cur = value;
    for key in path.iter() {
        if let Some(obj) = cur.as_object() {
            if let Some(next) = obj.get(*key) {
                cur = next;
            } else {
                return String::new();
            }
        } else if let Some(arr) = cur.as_array() {
            if let Ok(idx) = key.parse::<usize>() {
                if let Some(next) = arr.get(idx) {
                    cur = next;
                } else {
                    return String::new();
                }
            } else {
                return String::new();
            }
        } else {
            return String::new();
        }
    }
    value_to_text(cur)
}

/// 递归解析 JSON 值中所有字符串的占位符
pub fn resolve_json_placeholders(value: &Value, ctx: &NodeContext, current_msg: &Message) -> Value {
    match value {
        Value::String(s) => Value::String(resolve_placeholders_in_str(s, ctx, current_msg)),
        Value::Array(arr) => Value::Array(arr.iter().map(|v| resolve_json_placeholders(v, ctx, current_msg)).collect()),
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map.iter() {
                out.insert(k.clone(), resolve_json_placeholders(v, ctx, current_msg));
            }
            Value::Object(out)
        }
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::rule::RuleEngine;
    use crate::types::{ExecutionContext, Message, Node, Position};
    use serde_json::json;
    use std::sync::Arc;

    async fn make_ctx_with_msg(msg: Message) -> NodeContext<'static> {
        let engine = Arc::new(RuleEngine::new().await) as crate::engine::rule::DynRuleEngine;
        // 构造一个虚拟节点
        let node = Box::leak(Box::new(Node {
            id: Uuid::new_v4(),
            type_name: "log".to_string(),
            config: json!({}),
            layout: Position { x: 0.0, y: 0.0 },
            chain_id: Uuid::new_v4(),
        }));
        let exec = ExecutionContext::new(msg);
        NodeContext::new(node, &exec, engine)
    }

    #[tokio::test]
    async fn test_msg_placeholders_basic() {
        let mut msg = Message::new(
            "testType",
            json!({
                "user": {"name": "alice"},
                "items": [1, 2]
            }),
        );
        msg.metadata.insert("foo".into(), "bar".into());
        let ctx = make_ctx_with_msg(msg.clone()).await;

        let s = "ID=${msg.id},TYPE=${msg.type},META=${msg.metadata.foo},NAME=${msg.data.user.name},ITEM=${msg.data.items.1}";
        let out = resolve_placeholders_in_str(s, &ctx, &msg);
        assert!(out.contains("ID="));
        assert!(out.contains("TYPE=testType"));
        assert!(out.contains("META=bar"));
        assert!(out.contains("NAME=alice"));
        assert!(out.contains("ITEM=2"));
    }

    #[tokio::test]
    async fn test_escape_placeholder_literal() {
        let msg = Message::new("t", json!({"user": {"name": "alice"}}));
        let ctx = make_ctx_with_msg(msg.clone()).await;
        let s = r"A \${msg.data.user.name} B ${msg.data.user.name}";
        let out = resolve_placeholders_in_str(s, &ctx, &msg);
        // 未转义部分应被解析为 alice，转义占位符应保持为字面量 ${...}
        assert!(out.contains("${msg.data.user.name}"));
        assert!(out.contains("B alice"));
    }

    #[tokio::test]
    async fn test_cross_node_input_output_placeholders() {
        let mut current = Message::new("cur", json!({"ok": true}));
        // 构造前置节点input/output消息并记录到metadata
        let prev_id = Uuid::new_v4();
        let prev_input = Message::new("inputType", json!({"x": 1}));
        let prev_output = Message::new("outputType", json!({"user": {"name": "bob"}, "value": 42}));
        current
            .metadata
            .insert(format!("node.{}.input", prev_id), serde_json::to_string(&prev_input).unwrap());
        current
            .metadata
            .insert(format!("node.{}.output", prev_id), serde_json::to_string(&prev_output).unwrap());

        let ctx = make_ctx_with_msg(current.clone()).await;

        // 默认不写scope时取output
        let out1 = resolve_placeholders_in_str(&format!("${{{}}}", format!("{}.msg.data.user.name", prev_id)), &ctx, &current);
        assert_eq!(out1, "bob");

        // 指定input
        let out2 = resolve_placeholders_in_str(&format!("${{{}}}", format!("{}.input.msg.type", prev_id)), &ctx, &current);
        assert_eq!(out2, "inputType");

        // 指定output数值
        let out3 = resolve_placeholders_in_str(&format!("${{{}}}", format!("{}.output.msg.data.value", prev_id)), &ctx, &current);
        assert_eq!(out3, "42");
    }

    #[tokio::test]
    async fn test_nonexistent_path_returns_empty() {
        let msg = Message::new("t", json!({"user": {"name": "alice"}}));
        let ctx = make_ctx_with_msg(msg.clone()).await;
        let out = resolve_placeholders_in_str("${msg.data.nope}", &ctx, &msg);
        assert_eq!(out, "");
    }

    #[tokio::test]
    async fn test_json_recursive_placeholders() {
        let msg = Message::new(
            "t",
            json!({"user": {"name": "alice"}, "items": [1, 2] }),
        );
        let ctx = make_ctx_with_msg(msg.clone()).await;
        let prev_id = Uuid::new_v4();
        let prev_output = Message::new("out", json!({"v": 7}));
        let mut cur = msg.clone();
        cur.metadata.insert(
            format!("node.{}.output", prev_id),
            serde_json::to_string(&prev_output).unwrap(),
        );

        let templ = json!({
            "greet": "Hello ${msg.data.user.name}",
            "first": "${msg.data.items.0}",
            "cross": format!("${{{}}}", format!("{}.msg.data.v", prev_id)),
            "arr": ["X", "${msg.data.items.1}"]
        });
        let resolved = resolve_json_placeholders(&templ, &ctx, &cur);
        assert_eq!(resolved["greet"], json!("Hello alice"));
        assert_eq!(resolved["first"], json!("1"));
        assert_eq!(resolved["cross"], json!("7"));
        assert_eq!(resolved["arr"][1], json!("2"));
    }
}