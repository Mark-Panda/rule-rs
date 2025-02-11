use crate::types::NodeType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeDescriptor {
    pub type_name: String,
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub node_type: NodeType,
}
