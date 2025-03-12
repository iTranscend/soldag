use eyre::{bail, eyre, OptionExt};
use mongodb::bson::{oid::ObjectId, RawDocument};
use serde::{Deserialize, Serialize};
use solana_transaction_status_client_types::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage, UiRawMessage,
    UiTransactionStatusMeta,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    // pub _id: ObjectId,
    pub signature: String,
    pub message: UiRawMessage,
    pub meta: UiTransactionStatusMeta,
    pub block_time: Option<bson::DateTime>,
}

impl TryFrom<EncodedTransactionWithStatusMeta> for Transaction {
    type Error = eyre::Report;

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
            // _id: ObjectId::new(),
            signature: transaction_data.signatures[0].clone(),
            message,
            meta,
            block_time: None,
        })
    }
}
