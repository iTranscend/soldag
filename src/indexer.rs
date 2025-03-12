//! Indexer module for processing Solana blockchain data.
//!
//! This module is responsible for fetching, processing, and storing Solana blockchain
//! transactions. It runs multiple concurrent tasks to efficiently handle block processing
//! and catch up with missed blocks. The indexer maintains consistency by tracking the
//! last processed block and ensuring no blocks are missed.

use std::{str::FromStr, sync::Arc};

use chrono::{DateTime, Utc};
use log::{error, info};
use solana_account_decoder_client_types::{UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcBlockConfig},
    rpc_request::RpcRequest,
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

/// Core indexer struct managing blockc data processing.
///
/// The indexer maintains a connection to a Solana RPC node and tracks block
/// processing state. It uses channels to coordinate between different processing tasks.
#[derive(Clone)]
pub struct Indexer {
    /// RPC client for Solana blockchain interaction
    client: Arc<RpcClient>,
    /// Storage interface for persisting processed data
    storage: Arc<Storage>,
    /// Last processed block slot for tracking progress
    previous_block_slot: Option<u64>,
}

impl Indexer {
    /// Creates a new Indexer instance.
    ///
    /// # Arguments
    ///
    /// * `rpc_url` - URL of the Solana RPC endpoint
    /// * `rpc_api_key` - Optional API key for RPC access
    /// * `storage` - Storage instance for persisting data
    ///
    /// # Returns
    ///
    /// * `eyre::Result<Self>` - Configured indexer instance
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * RPC endpoint is unreachable
    /// * Health check fails
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

    /// Starts the indexer service.
    ///
    /// This function initiates three concurrent tasks:
    /// 1. Main block processing loop
    /// 2. Block data processing and storage
    /// 3. Missing block detection and catch-up
    ///
    /// # Arguments
    ///
    /// * `update_interval` - Milliseconds between block checks
    ///
    /// # Returns
    ///
    /// * `eyre::Result<()>` - Runs indefinitely unless an error occurs
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

    /// Retrieves account information from the Solana blockchain.
    ///
    /// # Arguments
    ///
    /// * `pubkey` - Public key of the account to fetch
    ///
    /// # Returns
    ///
    /// * `eyre::Result<Account>` - Account data if found
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Public key is invalid
    /// * Account does not exist
    /// * RPC request fails
    pub async fn get_account(&self, pubkey: String) -> eyre::Result<Account> {
        let pubkey = Pubkey::from_str(&pubkey)?;
        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64Zstd),
            data_slice: Some(UiDataSliceConfig {
                offset: 0,
                length: 20,
            }),
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Finalized,
                ..Default::default()
            }),
            min_context_slot: None,
        };

        if let Some(account) = self
            .client
            .get_account_with_config(&pubkey, config)
            .await?
            .value
        {
            Ok(account)
        } else {
            Err(eyre::eyre!("Account not found"))
        }
    }
}

/// Creates a configuration for block fetching.
///
/// Sets up the RPC configuration for retrieving block data with full
/// transaction details and finalized commitment.
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

/// Processes blocks and stores transactions.
///
/// This function runs in a separate task and handles the storage of
/// transaction data from processed blocks.
///
/// # Arguments
///
/// * `storage` - Storage instance for persisting data
/// * `rx` - Channel receiver for block data
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

/// Handles missed block detection and processing.
///
/// This function runs in a separate task and ensures no blocks are missed
/// during normal operation. If gaps are detected, it processes the missing blocks.
///
/// # Arguments
///
/// * `client` - RPC client for fetching missed blocks
/// * `store_tx` - Channel sender for block processing
/// * `rx` - Channel receiver for missed block ranges
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

/// Fetches a block from the Solana blockchain with retry logic.
///
/// # Arguments
///
/// * `client` - RPC client for block fetching
/// * `config` - Block fetch configuration
/// * `slot` - Block slot to fetch
/// * `interval` - Time between retries
/// * `retries` - Number of retry attempts
///
/// # Returns
///
/// * `eyre::Result<UiConfirmedBlock>` - Block data if successful
///
/// # Errors
///
/// Returns an error if all retry attempts fail
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
