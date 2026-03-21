use crate::{
    error::AppError,
    r#type::{
        CreateTreeRequest, CreatorTreesResponse, HealthResponse, ProofResponse, StoredTree,
        TreeSummary,
    },
};
use axum::{
    Json,
    extract::{Path, State},
};
use rs_merkle::{Hasher, MerkleTree, algorithms::Sha256};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};
use uuid::Uuid;

#[derive(Default)]
pub struct AppState {
    pub store: RwLock<TreeStore>,
}

#[derive(Default)]
pub struct TreeStore {
    trees: HashMap<String, StoredTree>,
    trees_by_campaign_creator: HashMap<String, Vec<String>>,
}

pub type SharedState = Arc<AppState>;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn create_tree(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTreeRequest>,
) -> Result<Json<TreeSummary>, AppError> {
    let stored_tree = build_stored_tree(payload)?;

    {
        let mut store = state
            .store
            .write()
            .map_err(|_| AppError::internal("failed to acquire tree store write lock"))?;

        let creator_trees = store
            .trees_by_campaign_creator
            .entry(stored_tree.campaign_creator_address.clone())
            .or_default();
        creator_trees.push(stored_tree.tree_id.clone());
        store
            .trees
            .insert(stored_tree.tree_id.clone(), stored_tree.clone());
    }

    Ok(Json(stored_tree.summary()))
}

pub async fn get_tree(
    State(state): State<SharedState>,
    Path(tree_id): Path<String>,
) -> Result<Json<TreeSummary>, AppError> {
    let store = state
        .store
        .read()
        .map_err(|_| AppError::internal("failed to acquire tree store read lock"))?;

    let tree = store
        .trees
        .get(&tree_id)
        .cloned()
        .ok_or_else(|| AppError::not_found(format!("tree not found: {}", tree_id)))?;

    Ok(Json(tree.summary()))
}

pub async fn get_tree_proof(
    State(state): State<SharedState>,
    Path((tree_id, leaf_address)): Path<(String, String)>,
) -> Result<Json<ProofResponse>, AppError> {
    let normalized_leaf_address = normalize_address(&leaf_address)?;
    let tree = {
        let store = state
            .store
            .read()
            .map_err(|_| AppError::internal("failed to acquire tree store read lock"))?;

        store
            .trees
            .get(&tree_id)
            .cloned()
            .ok_or_else(|| AppError::not_found(format!("tree not found: {}", tree_id)))?
    };

    Ok(Json(build_proof_response(&tree, &normalized_leaf_address)?))
}

pub async fn list_creator_trees(
    State(state): State<SharedState>,
    Path(campaign_creator_address): Path<String>,
) -> Result<Json<CreatorTreesResponse>, AppError> {
    let normalized_creator = normalize_address(&campaign_creator_address)?;
    let store = state
        .store
        .read()
        .map_err(|_| AppError::internal("failed to acquire tree store read lock"))?;

    let trees = store
        .trees_by_campaign_creator
        .get(&normalized_creator)
        .into_iter()
        .flat_map(|tree_ids| tree_ids.iter())
        .filter_map(|tree_id| store.trees.get(tree_id))
        .map(StoredTree::summary)
        .collect();

    Ok(Json(CreatorTreesResponse {
        campaign_creator_address: normalized_creator,
        trees,
    }))
}

fn build_stored_tree(payload: CreateTreeRequest) -> Result<StoredTree, AppError> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("`name` must not be empty"));
    }

    if payload.leaf_addresses.is_empty() {
        return Err(AppError::bad_request(
            "`leaf_addresses` must contain at least one Ethereum address",
        ));
    }

    let campaign_creator_address = normalize_address(&payload.campaign_creator_address)?;
    let normalized_leaf_addresses = normalize_addresses(&payload.leaf_addresses)?;
    let leaves: Vec<[u8; 32]> = normalized_leaf_addresses
        .iter()
        .map(|leaf_address| Sha256::hash(leaf_address.as_bytes()))
        .collect();

    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let root = merkle_tree
        .root_hex()
        .map(hex_with_prefix)
        .ok_or_else(|| AppError::internal("failed to compute the merkle root"))?;

    Ok(StoredTree {
        tree_id: Uuid::new_v4().to_string(),
        name: name.to_owned(),
        campaign_creator_address,
        leaf_addresses: normalized_leaf_addresses,
        leaves,
        root,
        leaf_count: payload.leaf_addresses.len(),
        depth: merkle_depth(payload.leaf_addresses.len()),
    })
}

fn build_proof_response(tree: &StoredTree, leaf_address: &str) -> Result<ProofResponse, AppError> {
    let index = tree
        .leaf_addresses
        .iter()
        .position(|candidate| candidate == leaf_address)
        .ok_or_else(|| {
            AppError::not_found(format!("leaf address not found in tree: {}", leaf_address))
        })?;

    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&tree.leaves);
    let proof = merkle_tree.proof(&[index]);
    let root = merkle_tree
        .root()
        .ok_or_else(|| AppError::internal("failed to load the merkle root from the tree"))?;
    let leaf = [tree.leaves[index]];
    let verified = proof.verify(root, &[index], &leaf, tree.leaves.len());

    Ok(ProofResponse {
        tree_id: tree.tree_id.clone(),
        name: tree.name.clone(),
        campaign_creator_address: tree.campaign_creator_address.clone(),
        leaf_address: leaf_address.to_owned(),
        index,
        leaf_hash: hex_with_prefix(hex::encode(tree.leaves[index])),
        path: proof
            .proof_hashes_hex()
            .into_iter()
            .map(hex_with_prefix)
            .collect(),
        root: tree.root.clone(),
        verified,
    })
}

fn normalize_addresses(addresses: &[String]) -> Result<Vec<String>, AppError> {
    let mut seen = HashSet::with_capacity(addresses.len());
    let mut normalized = Vec::with_capacity(addresses.len());

    for address in addresses {
        let normalized_address = normalize_address(address)?;
        if !seen.insert(normalized_address.clone()) {
            return Err(AppError::bad_request(format!(
                "duplicate address found: {}",
                normalized_address
            )));
        }
        normalized.push(normalized_address);
    }

    Ok(normalized)
}

fn normalize_address(address: &str) -> Result<String, AppError> {
    let trimmed = address.trim();
    if trimmed.len() != 42 || !trimmed.starts_with("0x") {
        return Err(AppError::bad_request(format!(
            "invalid Ethereum address format: {}",
            trimmed
        )));
    }

    if !trimmed[2..]
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(AppError::bad_request(format!(
            "invalid Ethereum address hex: {}",
            trimmed
        )));
    }

    Ok(format!("0x{}", trimmed[2..].to_ascii_lowercase()))
}

fn hex_with_prefix(value: impl AsRef<str>) -> String {
    format!("0x{}", value.as_ref())
}

fn merkle_depth(leaf_count: usize) -> usize {
    match leaf_count {
        0 | 1 => 0,
        _ => usize::BITS as usize - (leaf_count - 1).leading_zeros() as usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    fn sample_request() -> CreateTreeRequest {
        CreateTreeRequest {
            name: "allowlist".to_string(),
            campaign_creator_address: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa".to_string(),
            leaf_addresses: vec![
                "0x0000000000000000000000000000000000000001".to_string(),
                "0x0000000000000000000000000000000000000002".to_string(),
                "0x0000000000000000000000000000000000000003".to_string(),
            ],
        }
    }

    #[test]
    fn creates_a_tree_with_creator_metadata() {
        let tree = build_stored_tree(sample_request()).expect("tree should build");

        assert_eq!(tree.name, "allowlist");
        assert_eq!(
            tree.campaign_creator_address,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(tree.leaf_count, 3);
        assert_eq!(tree.depth, 2);
    }

    #[test]
    fn builds_index_path_and_root_for_a_stored_tree() {
        let tree = build_stored_tree(sample_request()).expect("tree should build");
        let proof = build_proof_response(&tree, "0x0000000000000000000000000000000000000002")
            .expect("proof should build");

        assert_eq!(proof.index, 1);
        assert_eq!(proof.root, tree.root);
        assert!(proof.verified);
        assert!(!proof.path.is_empty());
    }

    #[test]
    fn rejects_duplicate_addresses_after_normalization() {
        let request = CreateTreeRequest {
            name: "allowlist".to_string(),
            campaign_creator_address: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa".to_string(),
            leaf_addresses: vec![
                "0xabcdef0000000000000000000000000000001234".to_string(),
                "0xABCDEF0000000000000000000000000000001234".to_string(),
            ],
        };

        let error = build_stored_tree(request).expect_err("duplicates should be rejected");
        assert_eq!(error.into_response().status(), 400);
    }
}
