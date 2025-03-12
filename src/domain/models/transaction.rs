//! Transaction model module for representing Solana blockchain transactions.
//!
//! This module defines the structure and conversion traits for Solana transactions
//! as they are stored in the MongoDB database. It handles the transformation from
//! Solana's encoded transaction format to our internal representation.

use eyre::{bail, OptionExt};
use mongodb::bson;
use serde::{Deserialize, Serialize};
use solana_transaction_status_client_types::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage, UiRawMessage,
    UiTransactionStatusMeta,
};

/// Represents a Solana transaction in our database.
///
/// This struct contains the essential information about a Solana transaction,
/// including its signature, message content, metadata, and block time.
#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction signature
    pub signature: String,
    /// Raw transaction message containing instructions and account keys
    pub message: UiRawMessage,
    /// Transaction metadata including status and fee information
    pub meta: UiTransactionStatusMeta,
    /// Timestamp when the transaction was included in a block
    pub block_time: Option<bson::DateTime>,
}

impl TryFrom<EncodedTransactionWithStatusMeta> for Transaction {
    type Error = eyre::Report;

    /// Converts a Solana encoded transaction into our internal Transaction type.
    ///
    /// # Arguments
    ///
    /// * `encoded` - The encoded transaction from Solana's RPC
    ///
    /// # Returns
    ///
    /// * `eyre::Result<Transaction>` - Our internal transaction representation
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Transaction metadata is missing
    /// * Transaction encoding is not JSON
    /// * Message encoding is not Raw format
    fn try_from(encoded: EncodedTransactionWithStatusMeta) -> eyre::Result<Self> {
        let meta = encoded.meta.ok_or_eyre("Transaction meta is missing")?;

        let transaction_data = match encoded.transaction {
            EncodedTransaction::Json(tx) => tx,
            _ => bail!("Unsupported transaction encoding"),
        };

        let message = match transaction_data.message {
            UiMessage::Raw(raw) => raw,
            _ => bail!("Unsupported message encoding"),
        };

        Ok(Self {
            signature: transaction_data.signatures[0].clone(),
            message,
            meta,
            block_time: None,
        })
    }
}
