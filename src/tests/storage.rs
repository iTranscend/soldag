use chrono::Utc;
use mongodb::bson::DateTime;

use crate::{
    domain::{models::transaction::Transaction, storage::Storage},
    tests::helpers::{create_mock_message, create_mock_meta, create_mock_transaction},
};

#[tokio::test]
async fn test_storage_initialization() {
    let storage = Storage::init("soldag_test")
        .await
        .expect("Failed to initialize storage");
    assert!(!storage.transactions.name().is_empty());
}

#[tokio::test]
async fn test_transaction_insertion_and_retrieval() {
    let storage = Storage::init("soldag_test")
        .await
        .expect("Failed to initialize storage");

    // Create a mock transaction
    let transaction = Transaction {
        signature: uuid::Uuid::new_v4().to_string(),
        message: create_mock_message(),
        meta: create_mock_meta(),
        block_time: Some(DateTime::from_chrono(Utc::now())),
    };

    // Test insertion
    let tx_signature = transaction.signature.clone();
    let result = storage.insert_transaction(transaction).await;
    assert!(result.is_ok());

    // Test retrieval by signature
    let (transactions, next) = storage
        .get_transactions(Some(tx_signature.clone()), None, 10, 0)
        .await
        .expect("Failed to retrieve transaction");

    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0].signature, tx_signature);
    assert!(next.is_none());
}

#[tokio::test]
async fn test_transaction_pagination() {
    let storage = Storage::init("soldag_test")
        .await
        .expect("Failed to initialize storage");

    // Insert multiple transactions
    for i in 0..20 {
        let transaction = create_mock_transaction(i);
        storage
            .insert_transaction(transaction)
            .await
            .expect("Failed to insert");
    }

    // Test pagination
    let (transactions, next) = storage
        .get_transactions(None, None, 10, 0)
        .await
        .expect("Failed to retrieve transactions");

    assert_eq!(transactions.len(), 10);
    assert_eq!(next, Some(10));
}
