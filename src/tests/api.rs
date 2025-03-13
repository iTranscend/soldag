use url::Url;

use crate::{api, indexer::Indexer, tests::helpers::get_global_state};

#[tokio::test]
async fn test_fetch_transactions() {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let api_listen = listener.local_addr().unwrap();
    let storage = get_global_state().await.storage.clone();

    let indexer = Indexer::new(
        Url::parse("https://api.mainnet-beta.solana.com").unwrap(),
        None,
        storage.clone(),
    )
    .await
    .unwrap();

    tokio::spawn(api::start(listener, storage, indexer));

    let response = reqwest::Client::new()
        .get(format!("http://{}/transactions", api_listen))
        .send()
        .await
        .expect("Failed to send request");

    response.error_for_status().unwrap();
}

#[tokio::test]
async fn test_fetch_account() {
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .unwrap();
    let api_listen = listener.local_addr().unwrap();

    let storage = get_global_state().await.storage.clone();
    let indexer = Indexer::new(
        Url::parse("https://api.mainnet-beta.solana.com").unwrap(),
        None,
        storage.clone(),
    )
    .await
    .unwrap();

    tokio::spawn(api::start(listener, storage.clone(), indexer.clone()));

    let test_pubkey = "2y51bo8nuGLGzGCV4zr2zJuD2Ddu7myaRV3bjjw6GP9y";
    let response = reqwest::Client::new()
        .get(format!(
            "http://{}/accounts?pubkey={}",
            api_listen, test_pubkey
        ))
        .send()
        .await
        .expect("Failed to send request");

    response.error_for_status().unwrap();
}
