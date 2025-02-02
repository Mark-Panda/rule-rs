#[derive(Clone, Debug, serde::Serialize)]
pub struct NodeDescriptor {
    pub type_name: String,
    pub name: String,
    pub description: String,
}
