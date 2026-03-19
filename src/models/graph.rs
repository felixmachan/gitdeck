use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub short_id: String,
    pub parents: Vec<String>,
    pub children: Vec<String>,
    pub branch: String,
    pub subject: String,
    pub lane: usize,
    pub color_idx: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub branches: Vec<(String, usize)>, // Name and color index
}
