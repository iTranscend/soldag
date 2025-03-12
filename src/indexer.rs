use std::{str::FromStr, sync::Arc};

use chrono::{DateTime, Utc};
use log::{error, info};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::RpcBlockConfig, rpc_request::RpcRequest,
    rpc_response::RpcBlockhash,
};
use solana_rpc_client_api::response::Response;
use solana_sdk::{
    account::Account,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
};
use solana_transaction_status_client_types::{
    TransactionDetails, UiConfirmedBlock, UiTransactionEncoding,
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use url::Url;

use crate::domain::{models::transaction::Transaction, storage::Storage};

#[derive(Clone)]
pub struct Indexer {
    client: Arc<RpcClient>,
    storage: Arc<Storage>,
    previous_block_slot: Option<u64>,
}

impl Indexer {
    pub async fn new(
        rpc_url: Url,
        rpc_api_key: Option<&str>,
        storage: Arc<Storage>,
    ) -> eyre::Result<Self> {
        let mut rpc_url = rpc_url;

        // Construct rpc url if api key is supplied
        if let Some(rpc_api_key) = rpc_api_key {
            rpc_url
                .query_pairs_mut()
                .append_pair("api-key", rpc_api_key);
        }

        let client = Arc::new(RpcClient::new(rpc_url.to_string()));

        client.get_health().await?;

        Ok(Self {
            client,
            storage,
            previous_block_slot: None,
        })
    }

    pub async fn start(mut self, update_interval: u64) -> eyre::Result<()> {
        info!("Starting indexer service...");

        let (store_tx, store_rx) = mpsc::unbounded_channel();
        tokio::spawn(process_block(self.storage.clone(), store_rx));

        let (catch_up_tx, catch_up_rx) = mpsc::unbounded_channel();
        tokio::spawn(catch_up(self.client.clone(), store_tx.clone(), catch_up_rx));

        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_millis(update_interval as u64));

        let config = get_block_config();

        loop {
            interval.tick().await;

            // Data fetching and processing
            let latest_blockhash_resp = self
                .client
                .send::<Response<RpcBlockhash>>(
                    RpcRequest::GetLatestBlockhash,
                    serde_json::json!([]),
                )
                .await?;

            let latest_block_slot = latest_blockhash_resp.context.slot;
            info!("Latest block slot: {}", latest_block_slot);

            let previous_slot = self.previous_block_slot.get_or_insert_default();

            if !(latest_block_slot == *previous_slot + 1 || *previous_slot == 0) {
                catch_up_tx.send((*previous_slot, latest_block_slot))?;
            }

            *previous_slot = latest_block_slot;

            let block =
                get_block(&self.client, config, latest_block_slot, &mut interval, 1).await?;

            store_tx.send((block, latest_block_slot))?;
        }
    }

    pub async fn get_account(&self, pubkey: String) -> eyre::Result<Account> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        if let Some(account) = self
            .client
            .get_account_with_commitment(&pubkey, CommitmentConfig::finalized())
            .await?
            .value
        {
            Ok(account)
        } else {
            Err(eyre::eyre!("Account not found"))
        }
    }
}

fn get_block_config() -> RpcBlockConfig {
    RpcBlockConfig {
        encoding: Some(UiTransactionEncoding::Json),
        transaction_details: Some(TransactionDetails::Full),
        rewards: Some(true),
        commitment: Some(CommitmentConfig {
            commitment: CommitmentLevel::Finalized,
        }),
        max_supported_transaction_version: Some(0),
    }
}

async fn process_block(storage: Arc<Storage>, mut rx: UnboundedReceiver<(UiConfirmedBlock, u64)>) {
    let task = |storage: Arc<Storage>, block: UiConfirmedBlock, slot: u64| async move {
        match &block.transactions {
            Some(transactions) => {
                for transaction in transactions.iter() {
                    let mut transaction = Transaction::try_from(transaction.clone())?;

                    transaction.block_time = block
                        .block_time
                        .and_then(|t| DateTime::<Utc>::from_timestamp(t, 0))
                        .map(bson::DateTime::from_chrono);

                    storage.insert_transaction(transaction).await?;
                }
                info!("Block Slot: {:?} stored", slot);
            }
            None => {
                log::warn!("Block {} has no transactions", block.parent_slot);
            }
        }
        eyre::Ok(())
    };

    while let Some((block, slot)) = rx.recv().await {
        if let Err(err) = task(storage.clone(), block, slot).await {
            error!("Error processing block: {:?}", err);
        }
    }
}

async fn catch_up(
    client: Arc<RpcClient>,
    store_tx: UnboundedSender<(UiConfirmedBlock, u64)>,
    mut rx: UnboundedReceiver<(u64, u64)>,
) {
    let task = |client: Arc<RpcClient>,
                store_tx: UnboundedSender<(UiConfirmedBlock, u64)>,
                (previous_block_slot, current_block_slot)| async move {
        info!(
            "Missing {} blocks {} -> {}",
            current_block_slot - previous_block_slot,
            previous_block_slot,
            current_block_slot
        );

        let config = get_block_config();

        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(200));

        for slot in previous_block_slot..current_block_slot {
            interval.tick().await;
            let block = get_block(&client, config, slot, &mut interval, 5).await?;

            store_tx.send((block, slot))?;
        }
        interval.tick().await;

        eyre::Ok(())
    };

    while let Some(value) = rx.recv().await {
        if let Err(err) = task(client.clone(), store_tx.clone(), value).await {
            error!("Error processing block: {:?}", err);
        }
    }
}

async fn get_block(
    client: &RpcClient,
    config: RpcBlockConfig,
    slot: u64,
    interval: &mut tokio::time::Interval,
    retries: u8,
) -> eyre::Result<UiConfirmedBlock> {
    let mut error = None;

    for _ in 0..=retries {
        let block = client.get_block_with_config(slot, config).await;
        match block {
            Ok(block) => return Ok(block),
            Err(e) => error = Some(e),
        }
        interval.tick().await;
    }

    Err(error.unwrap().into())
}
