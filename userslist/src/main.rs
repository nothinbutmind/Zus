mod error;
mod merkle;
mod postgres;
mod types;

use crate::merkle::{
    AppState, create_campaign, get_campaign, get_claim_payload_by_body, get_claim_payload_by_path,
    health, list_campaigns, list_creator_campaigns,
};
use crate::postgres::init_db;
use axum::{
    Router,
    routing::{get, post},
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set to connect to PostgreSQL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    init_db(&pool).await?;

    let state = Arc::new(AppState { pool });
    let app = Router::new()
        .route("/health", get(health))
        .route("/campaigns", get(list_campaigns).post(create_campaign))
        .route("/campaigns/{campaign_id}", get(get_campaign))
        .route(
            "/campaigns/{campaign_id}/claim",
            post(get_claim_payload_by_body),
        )
        .route(
            "/campaigns/{campaign_id}/claim/{leaf_address}",
            get(get_claim_payload_by_path),
        )
        .route(
            "/campaign-creators/{campaign_creator_address}/campaigns",
            get(list_creator_campaigns),
        )
        .with_state(state);

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3000);
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(address).await?;

    println!("Merkle proof API listening on http://{}", address);
    axum::serve(listener, app).await?;

    Ok(())
}
