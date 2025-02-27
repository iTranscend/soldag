use std::net::SocketAddr;

use clap::Parser;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(short, long, value_name = "")]
    pub api_listen: SocketAddr,
}
