use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use solana_transaction_status_client_types::{
    EncodedTransaction, EncodedTransactionWithStatusMeta, UiMessage, UiRawMessage,
    UiTransactionStatusMeta,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub _id: ObjectId,
    pub signatures: Vec<String>,
    pub message: UiRawMessage,
    pub meta: UiTransactionStatusMeta,
}

impl TryFrom<EncodedTransactionWithStatusMeta> for Transaction {
    type Error = anyhow::Error;

    fn try_from(encoded: EncodedTransactionWithStatusMeta) -> Result<Self, Self::Error> {
        let meta = encoded
            .meta
            .ok_or_else(|| anyhow::anyhow!("Transaction meta is missing"))?;

        let transaction_data = match encoded.transaction {
            EncodedTransaction::Json(tx) => tx,
            _ => return Err(anyhow::anyhow!("Unsupported transaction encoding")),
        };

        let message = match transaction_data.message {
            UiMessage::Raw(raw) => raw,
            _ => return Err(anyhow::anyhow!("Unsupported message encoding")),
        };

        Ok(Self {
            _id: ObjectId::new(),
            signatures: transaction_data.signatures,
            message,
            meta,
        })
    }
}
