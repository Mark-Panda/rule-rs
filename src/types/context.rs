use super::*;
use std::sync::Arc;

pub struct NodeContext {
    pub node: Arc<Node>,
    pub metadata: HashMap<String, String>,
}

impl NodeContext {
    pub fn new(node: &Node, ctx: &ExecutionContext) -> Self {
        Self {
            node: Arc::new(node.clone()),
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
