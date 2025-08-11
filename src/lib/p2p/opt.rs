use clap::Parser;
use std::path::PathBuf;

use libp2p::Multiaddr;
#[derive(Parser, Debug)]
#[clap(name = "libp2p file sharing example")]
pub struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[clap(long)]
    pub secret_key_seed: Option<u8>,

    #[clap(long)]
    pub peer: Option<Multiaddr>,

    #[clap(long)]
    pub listen_address: Option<Multiaddr>,

    #[clap(subcommand)]
    pub argument: CliArgument,
}

#[derive(Debug, Parser)]
pub enum CliArgument {
    Provide {
        #[clap(long)]
        path: PathBuf,
        #[clap(long)]
        name: String,
    },
    Get {
        #[clap(long)]
        name: String,
    },
    Kv {
        #[clap(long)]
        get: Option<String>,
    },
}
