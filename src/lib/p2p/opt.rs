use clap::Parser;
use std::path::PathBuf;

use libp2p::Multiaddr;
#[derive(Parser, Debug)]
#[command(name = "libp2p file sharing example")]
pub struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[arg(long)]
    pub secret_key_seed: Option<u8>,

    #[arg(long)]
    pub peer: Option<Multiaddr>,

    #[arg(long)]
    pub listen_address: Option<Multiaddr>,

    #[command(subcommand)]
    pub argument: CliArgument,
}

#[derive(Debug, Parser)]
pub enum CliArgument {
    Provide {
        #[arg(long)]
        path: PathBuf,
        #[arg(long)]
        name: String,
    },
    Get {
        #[arg(long)]
        name: String,
    },
    Kv {
        #[arg(long)]
        get: Option<String>,
    },
}
