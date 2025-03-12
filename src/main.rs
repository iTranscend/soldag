//! SolDag - A Solana blockchain data aggregator.
//!
//! SolDag is a high-performance application that indexes and serves Solana blockchain
//! data. It consists of two main services:
//! 1. An indexer that processes blockchain data and stores it in MongoDB
//! 2. A REST API that provides access to the indexed data
//!
//! The application is built with reliability in mind, featuring automatic service
//! recovery and concurrent processing of blockchain data.

use clap::Parser;
use log::{error, info};

use domain::storage::Storage;

mod api;
mod cli;
mod domain;
pub mod indexer;
mod logger;

/// Initializes application services and starts processing.
///
/// This function sets up the environment, establishes database connections,
/// and starts both the indexer and API services. It includes automatic
/// retry logic for service recovery.
///
/// # Returns
///
/// * `eyre::Result<()>` - Success or error status
///
/// # Errors
///
/// Returns an error if:
/// * Environment setup fails
/// * Database connection fails
/// * Service initialization fails
async fn init() -> eyre::Result<()> {
    color_eyre::install()?;

    dotenv::dotenv().ok();

    let args = cli::Args::parse();

    let storage = Storage::init().await?;

    let indexer =
        indexer::Indexer::new(args.rpc_url, args.rpc_api_key.as_deref(), storage.clone()).await?;
    let mut indexer_handle = tokio::spawn(indexer.clone().start(args.update_interval));

    let mut api_handle = tokio::spawn(api::start(
        args.api_listen,
        storage.clone(),
        indexer.clone(),
    ));

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
                api_handle = tokio::spawn(api::start(args.api_listen, storage.clone(), indexer.clone()))

            }
        }
    }
    Ok(())
}

/// Entry point.
///
/// Sets up logging and starts the application services. If initialization
/// fails, SolDag will exit with a non-zero status code.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    logger::setup();

    info!("SolDag started, initializing services....");

    init().await?;

    if let Err(e) = init().await {
        error!("Initialization error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
