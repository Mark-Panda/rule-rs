use super::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct NodeContext<'a> {
    pub node: &'a Node,
    pub metadata: HashMap<String, String>,
}

impl<'a> NodeContext<'a> {
    pub fn new(node: &'a Node, ctx: &ExecutionContext) -> Self {
        Self {
            node,
            metadata: ctx.metadata.clone(),
        }
    }
}

pub struct ExecutionContext {
    pub msg: Message,
    pub metadata: HashMap<String, String>,
}

impl ExecutionContext {
    pub fn new(msg: Message) -> Self {
        Self {
            msg,
            metadata: HashMap::new(),
        }
    }
}
