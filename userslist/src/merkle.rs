use crate::{
    error::AppError,
    types::{
        CampaignSummary, ClaimLookupRequest, ClaimPayloadResponse, CreateCampaignRequest,
        CreatorCampaignsResponse, HealthResponse, PreparedCampaign, PreparedClaim, RecipientInput,
    },
};
use axum::{
    Json,
    extract::{Path, State},
};
use num_bigint::BigUint;
use rs_merkle::{Hasher, MerkleTree, algorithms::Sha256};
use sqlx::{FromRow, PgPool, types::Json as SqlJson};
use std::{collections::HashSet, sync::Arc};
use uuid::Uuid;

const HASH_ALGORITHM: &str = "sha256";
const LEAF_ENCODING: &str =
    "sha256(abi.encodePacked(uint64 index, address leaf_address, uint256 amount))";

pub struct AppState {
    pub pool: PgPool,
}

pub type SharedState = Arc<AppState>;

#[derive(Debug, Clone)]
struct NormalizedRecipient {
    leaf_address: String,
    amount: String,
}

#[derive(Debug, FromRow)]
struct CampaignSummaryRow {
    campaign_id: Uuid,
    name: String,
    campaign_creator_address: String,
    merkle_root: String,
    leaf_count: i32,
    depth: i32,
    hash_algorithm: String,
    leaf_encoding: String,
}

#[derive(Debug, FromRow)]
struct ClaimPayloadRow {
    campaign_id: Uuid,
    name: String,
    campaign_creator_address: String,
    leaf_address: String,
    amount: String,
    leaf_index: i32,
    leaf_hash: String,
    proof: SqlJson<Vec<String>>,
    merkle_root: String,
    hash_algorithm: String,
    leaf_encoding: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub async fn create_campaign(
    State(state): State<SharedState>,
    Json(payload): Json<CreateCampaignRequest>,
) -> Result<Json<CampaignSummary>, AppError> {
    let prepared = prepare_campaign(payload)?;
    insert_campaign(&state.pool, &prepared).await?;

    Ok(Json(prepared.summary))
}

pub async fn get_campaign(
    State(state): State<SharedState>,
    Path(campaign_id): Path<String>,
) -> Result<Json<CampaignSummary>, AppError> {
    let campaign_id = parse_campaign_id(&campaign_id)?;
    let row = sqlx::query_as::<_, CampaignSummaryRow>(
        r#"
        SELECT
            id AS campaign_id,
            name,
            campaign_creator_address,
            merkle_root,
            leaf_count,
            depth,
            hash_algorithm,
            leaf_encoding
        FROM campaigns
        WHERE id = $1
        "#,
    )
    .bind(campaign_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::not_found(format!("campaign not found: {}", campaign_id)))?;

    Ok(Json(campaign_summary_from_row(row)?))
}

pub async fn get_claim_payload_by_path(
    State(state): State<SharedState>,
    Path((campaign_id, leaf_address)): Path<(String, String)>,
) -> Result<Json<ClaimPayloadResponse>, AppError> {
    let campaign_id = parse_campaign_id(&campaign_id)?;
    let leaf_address = normalize_address(&leaf_address)?;
    let response = fetch_claim_payload(&state.pool, campaign_id, &leaf_address).await?;

    Ok(Json(response))
}

pub async fn get_claim_payload_by_body(
    State(state): State<SharedState>,
    Path(campaign_id): Path<String>,
    Json(payload): Json<ClaimLookupRequest>,
) -> Result<Json<ClaimPayloadResponse>, AppError> {
    let campaign_id = parse_campaign_id(&campaign_id)?;
    let leaf_address = normalize_address(&payload.leaf_address)?;
    let response = fetch_claim_payload(&state.pool, campaign_id, &leaf_address).await?;

    Ok(Json(response))
}

pub async fn list_creator_campaigns(
    State(state): State<SharedState>,
    Path(campaign_creator_address): Path<String>,
) -> Result<Json<CreatorCampaignsResponse>, AppError> {
    let campaign_creator_address = normalize_address(&campaign_creator_address)?;
    let rows = sqlx::query_as::<_, CampaignSummaryRow>(
        r#"
        SELECT
            id AS campaign_id,
            name,
            campaign_creator_address,
            merkle_root,
            leaf_count,
            depth,
            hash_algorithm,
            leaf_encoding
        FROM campaigns
        WHERE campaign_creator_address = $1
        ORDER BY created_at DESC, name ASC
        "#,
    )
    .bind(&campaign_creator_address)
    .fetch_all(&state.pool)
    .await?;

    let campaigns = rows
        .into_iter()
        .map(campaign_summary_from_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(CreatorCampaignsResponse {
        campaign_creator_address,
        campaigns,
    }))
}

async fn insert_campaign(pool: &PgPool, prepared: &PreparedCampaign) -> Result<(), AppError> {
    let mut transaction = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO campaigns (
            id,
            name,
            campaign_creator_address,
            merkle_root,
            leaf_count,
            depth,
            hash_algorithm,
            leaf_encoding
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(prepared.campaign_id)
    .bind(&prepared.summary.name)
    .bind(&prepared.summary.campaign_creator_address)
    .bind(&prepared.summary.merkle_root)
    .bind(i32::try_from(prepared.summary.leaf_count).map_err(|_| {
        AppError::internal("leaf count overflow while storing the prepared campaign")
    })?)
    .bind(i32::try_from(prepared.summary.depth).map_err(|_| {
        AppError::internal("tree depth overflow while storing the prepared campaign")
    })?)
    .bind(&prepared.summary.hash_algorithm)
    .bind(&prepared.summary.leaf_encoding)
    .execute(&mut *transaction)
    .await?;

    for claim in &prepared.claims {
        sqlx::query(
            r#"
            INSERT INTO campaign_claims (
                campaign_id,
                leaf_address,
                amount,
                leaf_index,
                leaf_hash,
                proof
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(prepared.campaign_id)
        .bind(&claim.leaf_address)
        .bind(&claim.amount)
        .bind(claim.index)
        .bind(&claim.leaf_hash)
        .bind(SqlJson(&claim.proof))
        .execute(&mut *transaction)
        .await?;
    }

    transaction.commit().await?;
    Ok(())
}

async fn fetch_claim_payload(
    pool: &PgPool,
    campaign_id: Uuid,
    leaf_address: &str,
) -> Result<ClaimPayloadResponse, AppError> {
    let row = sqlx::query_as::<_, ClaimPayloadRow>(
        r#"
        SELECT
            c.id AS campaign_id,
            c.name,
            c.campaign_creator_address,
            cc.leaf_address,
            cc.amount,
            cc.leaf_index,
            cc.leaf_hash,
            cc.proof,
            c.merkle_root,
            c.hash_algorithm,
            c.leaf_encoding
        FROM campaign_claims cc
        INNER JOIN campaigns c ON c.id = cc.campaign_id
        WHERE c.id = $1 AND cc.leaf_address = $2
        "#,
    )
    .bind(campaign_id)
    .bind(leaf_address)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        AppError::not_found(format!(
            "claim payload not found for campaign {} and address {}",
            campaign_id, leaf_address
        ))
    })?;

    claim_payload_from_row(row)
}

fn prepare_campaign(payload: CreateCampaignRequest) -> Result<PreparedCampaign, AppError> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err(AppError::bad_request("`name` must not be empty"));
    }

    if payload.recipients.is_empty() {
        return Err(AppError::bad_request(
            "`recipients` must contain at least one recipient",
        ));
    }

    let campaign_creator_address = normalize_address(&payload.campaign_creator_address)?;
    let recipients = normalize_recipients(&payload.recipients)?;
    let leaves = recipients
        .iter()
        .enumerate()
        .map(|(index, recipient)| {
            build_leaf_hash(index, &recipient.leaf_address, &recipient.amount)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let merkle_tree = MerkleTree::<Sha256>::from_leaves(&leaves);
    let merkle_root = merkle_tree
        .root_hex()
        .map(hex_with_prefix)
        .ok_or_else(|| AppError::internal("failed to compute the merkle root"))?;
    let root_hash = merkle_tree
        .root()
        .ok_or_else(|| AppError::internal("failed to load the merkle root for proof generation"))?;

    let claims = recipients
        .into_iter()
        .enumerate()
        .map(|(index, recipient)| {
            let proof = merkle_tree.proof(&[index]);
            let leaf = [leaves[index]];
            let verified = proof.verify(root_hash, &[index], &leaf, leaves.len());
            if !verified {
                return Err(AppError::internal(format!(
                    "generated proof failed verification for {}",
                    recipient.leaf_address
                )));
            }

            Ok(PreparedClaim {
                leaf_address: recipient.leaf_address,
                amount: recipient.amount,
                index: i32::try_from(index)
                    .map_err(|_| AppError::bad_request("too many recipients for i32 leaf index"))?,
                leaf_hash: hex_with_prefix(hex::encode(leaves[index])),
                proof: proof
                    .proof_hashes_hex()
                    .into_iter()
                    .map(hex_with_prefix)
                    .collect(),
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    let campaign_id = Uuid::new_v4();
    let leaf_count = leaves.len();
    let depth = merkle_depth(leaf_count);

    Ok(PreparedCampaign {
        campaign_id,
        summary: CampaignSummary {
            campaign_id: campaign_id.to_string(),
            name: name.to_owned(),
            campaign_creator_address,
            merkle_root,
            leaf_count,
            depth,
            hash_algorithm: HASH_ALGORITHM.to_string(),
            leaf_encoding: LEAF_ENCODING.to_string(),
        },
        claims,
    })
}

fn campaign_summary_from_row(row: CampaignSummaryRow) -> Result<CampaignSummary, AppError> {
    Ok(CampaignSummary {
        campaign_id: row.campaign_id.to_string(),
        name: row.name,
        campaign_creator_address: row.campaign_creator_address,
        merkle_root: row.merkle_root,
        leaf_count: usize::try_from(row.leaf_count)
            .map_err(|_| AppError::internal("negative leaf count loaded from database"))?,
        depth: usize::try_from(row.depth)
            .map_err(|_| AppError::internal("negative tree depth loaded from database"))?,
        hash_algorithm: row.hash_algorithm,
        leaf_encoding: row.leaf_encoding,
    })
}

fn claim_payload_from_row(row: ClaimPayloadRow) -> Result<ClaimPayloadResponse, AppError> {
    Ok(ClaimPayloadResponse {
        campaign_id: row.campaign_id.to_string(),
        name: row.name,
        campaign_creator_address: row.campaign_creator_address,
        leaf_address: row.leaf_address,
        amount: row.amount,
        index: usize::try_from(row.leaf_index)
            .map_err(|_| AppError::internal("negative leaf index loaded from database"))?,
        leaf_hash: row.leaf_hash,
        proof: row.proof.0,
        merkle_root: row.merkle_root,
        hash_algorithm: row.hash_algorithm,
        leaf_encoding: row.leaf_encoding,
    })
}

fn normalize_recipients(
    recipients: &[RecipientInput],
) -> Result<Vec<NormalizedRecipient>, AppError> {
    let mut seen = HashSet::with_capacity(recipients.len());
    let mut normalized = Vec::with_capacity(recipients.len());

    for recipient in recipients {
        let leaf_address = normalize_address(&recipient.leaf_address)?;
        let amount = normalize_amount(&recipient.amount)?;

        if !seen.insert(leaf_address.clone()) {
            return Err(AppError::bad_request(format!(
                "duplicate leaf address found: {}",
                leaf_address
            )));
        }

        normalized.push(NormalizedRecipient {
            leaf_address,
            amount,
        });
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

fn normalize_amount(amount: &str) -> Result<String, AppError> {
    let trimmed = amount.trim();
    if trimmed.is_empty() {
        return Err(AppError::bad_request("amount must not be empty"));
    }

    if !trimmed.chars().all(|character| character.is_ascii_digit()) {
        return Err(AppError::bad_request(format!(
            "amount must be a base-10 integer string: {}",
            trimmed
        )));
    }

    let parsed = parse_big_uint(trimmed)?;
    Ok(parsed.to_str_radix(10))
}

fn build_leaf_hash(index: usize, leaf_address: &str, amount: &str) -> Result<[u8; 32], AppError> {
    let leaf_bytes = build_leaf_bytes(index, leaf_address, amount)?;
    Ok(Sha256::hash(&leaf_bytes))
}

fn build_leaf_bytes(index: usize, leaf_address: &str, amount: &str) -> Result<Vec<u8>, AppError> {
    let index = u64::try_from(index)
        .map_err(|_| AppError::bad_request("too many recipients for u64 index encoding"))?;
    let address_bytes = decode_address_bytes(leaf_address)?;
    let amount = parse_big_uint(amount)?;
    let amount_bytes = amount.to_bytes_be();

    if amount_bytes.len() > 32 {
        return Err(AppError::bad_request(
            "amount exceeds uint256 size for leaf encoding",
        ));
    }

    let mut leaf_bytes = Vec::with_capacity(8 + 20 + 32);
    leaf_bytes.extend_from_slice(&index.to_be_bytes());
    leaf_bytes.extend_from_slice(&address_bytes);
    leaf_bytes.extend(vec![0u8; 32 - amount_bytes.len()]);
    leaf_bytes.extend_from_slice(&amount_bytes);

    Ok(leaf_bytes)
}

fn decode_address_bytes(address: &str) -> Result<[u8; 20], AppError> {
    let decoded = hex::decode(&address[2..])
        .map_err(|_| AppError::bad_request(format!("invalid Ethereum address hex: {}", address)))?;

    <[u8; 20]>::try_from(decoded.as_slice()).map_err(|_| {
        AppError::bad_request(format!(
            "Ethereum address must decode to 20 bytes: {}",
            address
        ))
    })
}

fn parse_big_uint(value: &str) -> Result<BigUint, AppError> {
    BigUint::parse_bytes(value.as_bytes(), 10)
        .ok_or_else(|| AppError::bad_request(format!("failed to parse integer amount: {}", value)))
}

fn parse_campaign_id(campaign_id: &str) -> Result<Uuid, AppError> {
    Uuid::parse_str(campaign_id)
        .map_err(|_| AppError::bad_request(format!("invalid campaign id: {}", campaign_id)))
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

    fn sample_request() -> CreateCampaignRequest {
        CreateCampaignRequest {
            name: "summer airdrop".to_string(),
            campaign_creator_address: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa".to_string(),
            recipients: vec![
                RecipientInput {
                    leaf_address: "0x0000000000000000000000000000000000000001".to_string(),
                    amount: "100".to_string(),
                },
                RecipientInput {
                    leaf_address: "0x0000000000000000000000000000000000000002".to_string(),
                    amount: "250".to_string(),
                },
                RecipientInput {
                    leaf_address: "0x0000000000000000000000000000000000000003".to_string(),
                    amount: "500".to_string(),
                },
            ],
        }
    }

    #[test]
    fn prepares_a_campaign_with_precomputed_claims() {
        let prepared = prepare_campaign(sample_request()).expect("campaign should prepare");

        assert_eq!(prepared.summary.name, "summer airdrop");
        assert_eq!(prepared.summary.leaf_count, 3);
        assert_eq!(prepared.summary.depth, 2);
        assert_eq!(prepared.claims.len(), 3);
        assert_eq!(prepared.claims[1].amount, "250");
        assert!(!prepared.claims[1].proof.is_empty());
    }

    #[test]
    fn normalizes_amounts_before_hashing() {
        let request = CreateCampaignRequest {
            name: "summer airdrop".to_string(),
            campaign_creator_address: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa".to_string(),
            recipients: vec![RecipientInput {
                leaf_address: "0x0000000000000000000000000000000000000001".to_string(),
                amount: "00042".to_string(),
            }],
        };

        let prepared = prepare_campaign(request).expect("campaign should prepare");
        assert_eq!(prepared.claims[0].amount, "42");
    }

    #[test]
    fn rejects_duplicate_leaf_addresses() {
        let request = CreateCampaignRequest {
            name: "summer airdrop".to_string(),
            campaign_creator_address: "0xaAaAaAaaAaAaAaaAaAAAAAAAAaaaAaAaAaaAaaAa".to_string(),
            recipients: vec![
                RecipientInput {
                    leaf_address: "0xabcdef0000000000000000000000000000001234".to_string(),
                    amount: "1".to_string(),
                },
                RecipientInput {
                    leaf_address: "0xABCDEF0000000000000000000000000000001234".to_string(),
                    amount: "2".to_string(),
                },
            ],
        };

        let error = prepare_campaign(request).expect_err("duplicate addresses should fail");
        assert!(matches!(error, AppError::BadRequest(_)));
    }
}
