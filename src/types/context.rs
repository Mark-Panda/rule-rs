use super::*;
use crate::engine::RuleEngine;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NodeContext<'a> {
    pub node: &'a Node,
    pub metadata: HashMap<String, String>,
    pub engine: Arc<RuleEngine>,
    pub msg: Message,
}

impl<'a> NodeContext<'a> {
    pub fn new(node: &'a Node, ctx: &ExecutionContext, engine: Arc<RuleEngine>) -> Self {
        Self {
            node,
            metadata: ctx.metadata.clone(),
            engine,
            msg: ctx.msg.clone(),
        }
    }

    pub fn create_subchain_context(&self) -> ExecutionContext {
        ExecutionContext {
            msg: self.msg.clone(),
            metadata: self.metadata.clone(),
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
