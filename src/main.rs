use clap::Parser;
use log::{error, info};

use domain::storage::Storage;

mod api;
mod cli;
mod domain;
mod indexer;
mod logger;

async fn init() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    dotenv::dotenv().ok();

    let storage = Storage::init().await?;

    let indexer = indexer::Indexer::new(args.rpc_url, args.rpc_api_key.as_deref(), storage).await?;

    let mut indexer_handle = tokio::spawn(indexer.clone().start(args.update_interval));
    let mut api_handle = tokio::spawn(api::start(args.api_listen));

    // retry 3 times
    for _ in 1..=3 {
        tokio::select! {
            res = &mut indexer_handle => {
                if let Ok(Err(e)) = res {
                    error!("Indexer service failed: {}", e)
                }
                indexer_handle = tokio::spawn(indexer.clone().start(args.update_interval))
            }
            res = &mut api_handle => {
                if let Ok(Err(e)) = res {
                    error!("API service failed: {}", e)
                }
                api_handle = tokio::spawn(api::start(args.api_listen))

            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::setup();

    info!("SolDag started, initializing services....");

    init().await?;

    if let Err(e) = init().await {
        error!("Initialization error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
