//! Command-line interface configuration for SolDag.
//!
//! This module defines the command-line arguments and environment variables
//! that can be used to configure the application. It uses the clap framework
//! for argument parsing and supports both command-line flags and environment
//! variables for configuration.

use std::net::SocketAddr;

use clap::Parser;
use url::Url;

/// Command-line arguments for configuring the application.
///
/// These arguments can be provided via command-line flags or environment
/// variables. Use --help to see all available options.
#[derive(Parser)]
#[clap(author, version, about = "Solana data aggregator")]
pub struct Args {
    /// Helios RPC API key for authenticated access to Solana RPC endpoints.
    /// Can be set via RPC_API_KEY environment variable.
    #[clap(short = 'k', long, env = "RPC_API_KEY")]
    pub rpc_api_key: Option<String>,

    /// Solana RPC endpoint URL for blockchain data access.
    /// Defaults to Helius mainnet RPC.
    #[clap(short, long, default_value = "https://mainnet.helius-rpc.com")]
    pub rpc_url: Url,

    /// Time interval in milliseconds between block fetches.
    /// Controls how frequently the indexer checks for new blocks.
    #[clap(short, long, default_value = "400")]
    pub update_interval: u64,

    /// Network address and port for the API server to listen on.
    /// Specify in the format "host:port".
    #[clap(short, long, default_value = "127.0.0.1:8081")]
    pub api_listen: SocketAddr,
}
