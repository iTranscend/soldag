use std::net::SocketAddr;

use clap::Parser;
use url::Url;

#[derive(Parser)]
#[clap(author, version, about = "Solana data aggregator")]
pub struct Args {
    /// Helios RPC API key
    #[clap(short = 'k', long, env = "RPC_API_KEY")]
    pub rpc_api_key: Option<String>,

    /// Solana RPC endpoint
    #[clap(short, long, default_value = "https://mainnet.helius-rpc.com")]
    pub rpc_url: Url,

    /// Aggregator update interval in milliseconds
    #[clap(short, long, default_value = "400")]
    pub update_interval: u64,

    /// API server listen address
    #[clap(short, long, default_value = "127.0.0.1:8081")]
    pub api_listen: SocketAddr,
}
