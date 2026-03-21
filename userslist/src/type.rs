use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct CreateCampaignRequest {
    pub name: String,
    pub campaign_creator_address: String,
    pub recipients: Vec<RecipientInput>,
}

#[derive(Debug, Deserialize)]
pub struct RecipientInput {
    pub leaf_address: String,
    pub amount: String,
}

#[derive(Debug, Deserialize)]
pub struct ClaimLookupRequest {
    pub leaf_address: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CampaignSummary {
    pub campaign_id: String,
    pub name: String,
    pub campaign_creator_address: String,
    pub merkle_root: String,
    pub leaf_count: usize,
    pub depth: usize,
    pub hash_algorithm: String,
    pub leaf_encoding: String,
}

#[derive(Debug, Serialize)]
pub struct CreatorCampaignsResponse {
    pub campaign_creator_address: String,
    pub campaigns: Vec<CampaignSummary>,
}

#[derive(Debug, Serialize)]
pub struct ClaimPayloadResponse {
    pub campaign_id: String,
    pub name: String,
    pub campaign_creator_address: String,
    pub leaf_address: String,
    pub amount: String,
    pub index: usize,
    pub leaf_hash: String,
    pub proof: Vec<String>,
    pub merkle_root: String,
    pub hash_algorithm: String,
    pub leaf_encoding: String,
}

#[derive(Debug, Clone)]
pub struct PreparedCampaign {
    pub campaign_id: Uuid,
    pub summary: CampaignSummary,
    pub claims: Vec<PreparedClaim>,
}

#[derive(Debug, Clone)]
pub struct PreparedClaim {
    pub leaf_address: String,
    pub amount: String,
    pub index: i32,
    pub leaf_hash: String,
    pub proof: Vec<String>,
}
