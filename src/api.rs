use std::net::SocketAddr;

use axum::{debug_handler, extract::Query, routing::get, Json, Router};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransactionQuery {
    id: Option<String>,
    day: Option<String>,
}

#[derive(Serialize)]
pub struct TransactionResponse {}

#[debug_handler]
async fn fetch_transactions(
    Query(params): Query<TransactionQuery>,
) -> Json<Vec<TransactionResponse>> {
    Json(vec![])
}

pub async fn start(api_listen: SocketAddr) -> anyhow::Result<()> {
    let app = Router::new().route("/transactions", get(fetch_transactions));

    info!("Starting API server on {}", api_listen);

    let listener = tokio::net::TcpListener::bind(api_listen).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
