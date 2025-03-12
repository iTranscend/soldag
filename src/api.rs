//! API module for exposing block data through REST endpoints.
//!
//! This module provides HTTP endpoints for querying indexed Solana blockchain data.
//! It uses the Axum framework to handle HTTP requests and supports features like
//! pagination and filtering. The API provides access to transaction history and
//! account information.

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

/// Request parameters for paginated endpoints.
///
/// Generic struct that combines pagination parameters with endpoint-specific
/// query parameters.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Paginated<T> {
    /// Number of items to return
    pub count: Option<u64>,
    /// Number of items to skip
    pub offset: Option<u64>,
    /// Additional query parameters
    #[serde(flatten)]
    pub data: T,
}

/// Query parameters for transaction endpoints.
#[derive(Serialize, Debug, Deserialize)]
pub struct TransactionQuery {
    /// Optional transaction signature to filter by
    id: Option<String>,
    /// Optional date in DD/MM/YYYY format to filter transactions
    day: Option<String>,
}

/// Response format for transaction endpoints.
#[derive(Serialize, Debug)]
pub struct TransactionResponse {
    /// List of transactions matching the query
    pub data: Vec<Transaction>,
    /// Offset for the next page of results, if any
    pub next: Option<u64>,
}

/// Handles requests for transaction data.
///
/// Supports filtering by transaction ID or date, with pagination.
///
/// # Arguments
///
/// * `params` - Query parameters including pagination and filters
/// * `State((storage, _))` - Application state containing storage access
///
/// # Returns
///
/// * `Result<Json<TransactionResponse>, (StatusCode, String)>` - Transaction data or error
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

/// Query parameters for account information endpoints.
#[derive(Serialize, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountQuery {
    /// Public key of the account to fetch
    pubkey: String,
}

/// Response format for account information endpoints.
#[derive(Serialize, Debug)]
pub struct AccountResponse {
    /// Account data and metadata
    pub data: Account,
}

/// Handles requests for account information.
///
/// Fetches current account state from the Solana blockchain.
///
/// # Arguments
///
/// * `params` - Query parameters containing the account public key
/// * `State((_, indexer))` - Application state containing indexer access
///
/// # Returns
///
/// * `Result<Json<AccountResponse>, (StatusCode, String)>` - Account data or error
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

/// Starts the API server.
///
/// Sets up routes and begins listening for HTTP requests.
///
/// # Arguments
///
/// * `api_listen` - Socket address to listen on
/// * `storage` - Storage instance for data access
/// * `indexer` - Indexer instance for blockchain queries
///
/// # Returns
///
/// * `eyre::Result<()>` - Runs indefinitely unless an error occurs
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
