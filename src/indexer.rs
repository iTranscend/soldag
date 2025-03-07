use std::sync::Arc;

use log::info;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig, rpc_request::RpcRequest,
    rpc_response::RpcBlockhash,
};
use solana_rpc_client_api::response::Response;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_transaction_status_client_types::{TransactionDetails, UiTransactionEncoding};
use url::Url;

use crate::domain::storage::Storage;

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
    ) -> anyhow::Result<Self> {
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

    pub async fn start(self, update_interval: u64) -> anyhow::Result<()> {
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

            info!("SLOT: {}", latest_block_slot);

            let config = RpcBlockConfig {
                encoding: Some(UiTransactionEncoding::Json),
                transaction_details: Some(TransactionDetails::Full),
                rewards: Some(true),
                commitment: Some(CommitmentConfig {
                    commitment: CommitmentLevel::Finalized,
                }),
                max_supported_transaction_version: Some(0),
            };

            let block = self.client.get_block_with_config(324916165, config).await?;

            dbg!(&block);

            // store

            tokio::time::sleep(tokio::time::Duration::from_millis(update_interval as u64)).await;
        }
    }
}
