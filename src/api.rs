use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use axum::{
    debug_handler,
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use http::StatusCode;
use log::{error, info};
use serde::{Deserialize, Serialize};

use solana_sdk::account::Account;

use crate::{
    domain::{models::transaction::Transaction, storage::Storage},
    indexer::Indexer,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Paginated<T> {
    pub count: Option<u64>,
    pub offset: Option<u64>,
    #[serde(flatten)]
    pub data: T,
}

#[derive(Serialize, Debug, Deserialize)]
pub struct TransactionQuery {
    id: Option<String>,
    day: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct TransactionResponse {
    pub data: Vec<Transaction>,
    pub next: Option<u64>,
}

async fn fetch_transactions(
    Query(params): Query<Paginated<TransactionQuery>>,
    State((storage, _)): State<(Arc<Storage>, Indexer)>,
) -> Result<Json<TransactionResponse>, (StatusCode, String)> {
    let date = if let Some(day) = params.data.day {
        let date = NaiveDate::parse_from_str(&day, "%d/%m/%Y")
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid date: {}", e)))?;
        Some(date.and_hms_opt(0, 0, 0).expect("Infallible").and_utc())
    } else {
        None
    };

    let (data, next) = match storage
        .get_transactions(
            params.data.id,
            date,
            params.count.unwrap_or(10),
            params.offset.unwrap_or(0),
        )
        .await
    {
        Ok(res) => res,
        Err(e) => {
            error!("Error fetching transactions: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching transactions".to_string(),
            ));
        }
    };

    let response = TransactionResponse { data, next };

    Ok(Json(response))
}

#[derive(Serialize, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountQuery {
    pubkey: String,
}

#[derive(Serialize, Debug)]
pub struct AccountResponse {
    pub data: Account,
}

#[debug_handler]
async fn fetch_account(
    Query(params): Query<AccountQuery>,
    State((_, indexer)): State<(Arc<Storage>, Indexer)>,
) -> Result<Json<AccountResponse>, (StatusCode, String)> {
    let data = match indexer.get_account(params.pubkey).await {
        Ok(res) => res,
        Err(e) => {
            error!("Error fetching transactions: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching transactions".to_string(),
            ));
        }
    };

    let response = AccountResponse { data };

    Ok(Json(response))
}

pub async fn start(
    api_listen: SocketAddr,
    storage: Arc<Storage>,
    indexer: Indexer,
) -> eyre::Result<()> {
    let app = Router::new()
        .route("/transactions", get(fetch_transactions))
        .route("/accounts", get(fetch_account))
        .with_state((storage, indexer));

    info!("Starting API server on {}", api_listen);

    let listener = tokio::net::TcpListener::bind(api_listen).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
