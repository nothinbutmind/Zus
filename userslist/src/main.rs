mod error;
mod merkle;
mod r#type;

use crate::merkle::{AppState, create_tree, get_tree, get_tree_proof, health, list_creator_trees};
use axum::{
    Router,
    routing::{get, post},
};
use std::{env, net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let state = Arc::new(AppState::default());
    let app = Router::new()
        .route("/health", get(health))
        .route("/trees", post(create_tree))
        .route("/trees/:tree_id", get(get_tree))
        .route("/trees/:tree_id/proof/:leaf_address", get(get_tree_proof))
        .route(
            "/campaign-creators/:campaign_creator_address/trees",
            get(list_creator_trees),
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
