use std::sync::Arc;

use chrono::{DateTime, Utc};
use log::info;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig, rpc_request::RpcRequest,
    rpc_response::RpcBlockhash,
};
use solana_rpc_client_api::response::Response;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_transaction_status_client_types::{
    TransactionDetails, UiConfirmedBlock, UiTransactionEncoding,
};
use url::Url;

use crate::domain::{models::transaction::Transaction, storage::Storage};

#[derive(Clone)]
pub struct Indexer {
    client: Arc<RpcClient>,
    storage: Arc<Storage>,
}

impl Indexer {
    pub async fn new(
        rpc_url: Url,
        rpc_api_key: Option<&str>,
        storage: Arc<Storage>,
    ) -> eyre::Result<Self> {
        let mut rpc_url = rpc_url;

        // Construct rpc url if api key present
        if let Some(rpc_api_key) = rpc_api_key {
            rpc_url
                .query_pairs_mut()
                .append_pair("api-key", rpc_api_key);
        }

        let client = Arc::new(RpcClient::new(rpc_url.to_string()));

        client.get_health().await?;

        Ok(Self { client, storage })
    }

    pub async fn start(self, update_interval: u64) -> eyre::Result<()> {
        info!("Starting indexer service...");

        loop {
            // Data fetching and processing
            let latest_blockhash_resp = self
                .client
                .send::<Response<RpcBlockhash>>(
                    RpcRequest::GetLatestBlockhash,
                    serde_json::json!([]),
                )
                .await?;

            let latest_block_slot = latest_blockhash_resp.context.slot;

            let config = RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::Json),
                transaction_details: Some(TransactionDetails::Full),
                rewards: Some(true),
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Finalized,
                }),
                max_supported_transaction_version: Some(0),
            };

            let block = self
                .client
                .get_block_with_config(latest_block_slot, config)
                .await?;


            // process and store block
            self.process_block(&block).await?;

            info!("Block: {:?} stored", latest_block_slot);

            tokio::time::sleep(tokio::time::Duration::from_millis(update_interval as u64)).await;
        }
    }

    pub async fn process_block(&self, block: &UiConfirmedBlock) -> eyre::Result<()> {
        let block_time = block.block_time;
        match &block.transactions {
            Some(transactions) => {
                for transaction in transactions.iter() {
                    let mut transaction = Transaction::try_from(transaction.clone())?;

                    transaction.block_time = block_time
                        .and_then(|t| DateTime::<Utc>::from_timestamp(t, 0))
                        .map(bson::DateTime::from_chrono);

                    self.storage.insert_transaction(transaction).await?;
                }
            }
            None => {
                log::warn!("Block {} has no transactions", block.parent_slot);
            }
        }

        Ok(())
    }
}
