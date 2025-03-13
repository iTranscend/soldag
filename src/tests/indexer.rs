use url::Url;

use crate::{domain::storage::Storage, indexer::Indexer, tests::helpers::get_global_state};

#[tokio::test]
async fn test_indexer_initialization() {
    let storage = get_global_state().await.storage.clone();
    let rpc_url = Url::parse("https://api.mainnet-beta.solana.com").unwrap();

    let indexer = Indexer::new(rpc_url, None, storage).await;
    assert!(indexer.is_ok());
}

#[tokio::test]
async fn test_account_retrieval() {
    let storage = Storage::init("soldag_test")
        .await
        .expect("Failed to initialize storage");
    let rpc_url = Url::parse("https://api.mainnet-beta.solana.com").unwrap();

    let indexer = Indexer::new(rpc_url, None, storage)
        .await
        .expect("Failed to create indexer");

    // Test with a known Solana system program account
    let result = indexer
        .get_account("11111111111111111111111111111111".to_string())
        .await;

    assert!(result.is_ok());
    let account = result.unwrap();
    assert!(account.executable); // System program should be executable
}

#[tokio::test]
async fn test_indexer_with_invalid_url() {
    let storage = get_global_state().await.storage.clone();
    let invalid_url = Url::parse("https://invalid.solana.endpoint").unwrap();

    let result = Indexer::new(invalid_url, None, storage).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_block_processing() {
    let state = get_global_state().await;

    let rpc_url = Url::parse("https://api.mainnet-beta.solana.com").unwrap();
    let indexer = Indexer::new(rpc_url, None, state.storage.clone())
        .await
        .expect("Failed to create indexer");

    // Start the indexer with a short update interval
    let handle = tokio::spawn(async move { indexer.start(100).await });

    // Let it process a few blocks
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let (transactions, _) = state
        .storage
        .get_transactions(None, None, 10, 0)
        .await
        .expect("Failed to retrieve transactions");

    state.notifier.notify_waiters();

    assert!(!transactions.is_empty());

    handle.abort();
    let err = handle
        .await
        .expect_err("Indexer should have been cancelled");
    assert!(err.is_cancelled(), "{err}");
}
