use std::{fmt::Debug, net::SocketAddr, sync::Arc};

use axum::{
    debug_handler,
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::NaiveDate;
use eyre::eyre;
use http::StatusCode;
use log::{error, info};
use serde::{Deserialize, Serialize};

use mongodb::bson::Document;

use crate::domain::{models::transaction::Transaction, storage::Storage};

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

#[derive(Clone)]
pub struct Api {
    storage: Arc<Storage>,
}

impl Api {
    pub async fn new(storage: Arc<Storage>) -> eyre::Result<Self> {
        Ok(Self { storage })
    }

    // #[debug_handler]
    async fn fetch_transactions(
        Query(params): Query<Paginated<TransactionQuery>>,
        State(storage): State<Arc<Storage>>,
    ) -> Result<Json<TransactionResponse>, (StatusCode, String)> {
        dbg!(&params);
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
        dbg!(&response);

        Ok(Json(response))
    }


    pub async fn start(self, api_listen: SocketAddr) -> eyre::Result<()> {
        let app = Router::new()
            .route("/transactions", get(Self::fetch_transactions))
            .with_state(self.storage);

        info!("Starting API server on {}", api_listen);

        let listener = tokio::net::TcpListener::bind(api_listen).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
