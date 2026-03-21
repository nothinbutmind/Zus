use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct CreateTreeRequest {
    pub name: String,
    pub campaign_creator_address: String,
    pub leaf_addresses: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TreeSummary {
    pub tree_id: String,
    pub name: String,
    pub campaign_creator_address: String,
    pub root: String,
    pub leaf_count: usize,
    pub depth: usize,
    pub hash_algorithm: &'static str,
    pub leaf_encoding: &'static str,
}

#[derive(Debug, Serialize)]
pub struct CreatorTreesResponse {
    pub campaign_creator_address: String,
    pub trees: Vec<TreeSummary>,
}

#[derive(Debug, Serialize)]
pub struct ProofResponse {
    pub tree_id: String,
    pub name: String,
    pub campaign_creator_address: String,
    pub leaf_address: String,
    pub index: usize,
    pub leaf_hash: String,
    pub path: Vec<String>,
    pub root: String,
    pub verified: bool,
}

#[derive(Debug, Clone)]
pub struct StoredTree {
    pub tree_id: String,
    pub name: String,
    pub campaign_creator_address: String,
    pub leaf_addresses: Vec<String>,
    pub leaves: Vec<[u8; 32]>,
    pub root: String,
    pub leaf_count: usize,
    pub depth: usize,
}

impl StoredTree {
    pub fn summary(&self) -> TreeSummary {
        TreeSummary {
            tree_id: self.tree_id.clone(),
            name: self.name.clone(),
            campaign_creator_address: self.campaign_creator_address.clone(),
            root: self.root.clone(),
            leaf_count: self.leaf_count,
            depth: self.depth,
            hash_algorithm: "sha256",
            leaf_encoding: "normalized_address_string_utf8",
        }
    }
}
