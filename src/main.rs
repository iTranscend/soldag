use clap::Parser;
use log::{error, info};

mod cli;

async fn init() -> anyhow::Result<()> {
    let args = cli::Args::parse();

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
