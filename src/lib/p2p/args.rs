use clap::Parser;
use libp2p::Multiaddr;
use crate::p2p::network_config::Network;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub secret: Option<u8>,

    #[arg(long)]
    pub peer: Option<String>,

    #[arg(long)]
    pub multiaddr: Option<Multiaddr>,

    #[arg(long, value_enum, default_value = &"ipfs")]
    pub network: Option<Network>,

    #[arg(long)]
    pub flag_topo_order: bool,
    #[arg(long)]
    pub flag_date_order: bool,
    #[arg(long)]
    pub flag_reverse: bool,
    #[arg(long)]
    pub flag_author: Option<String>,
    #[arg(long)]
    pub flag_committer: Option<String>,
    #[arg(long = "grep")]
    pub flag_grep: Option<String>,
    #[arg(long = "git-dir")]
    pub flag_git_dir: Option<String>,
    #[arg(long)]
    pub flag_skip: Option<usize>,
    #[arg(short = 'n', long)]
    pub flag_max_count: Option<usize>,
    #[arg(long)]
    pub flag_merges: bool,
    #[arg(long)]
    pub flag_no_merges: bool,
    #[arg(long)]
    pub flag_no_min_parents: bool,
    #[arg(long)]
    pub flag_no_max_parents: bool,
    #[arg(long)]
    pub flag_max_parents: Option<usize>,
    #[arg(long)]
    pub flag_min_parents: Option<usize>,
    #[arg(long, short)]
    pub flag_patch: bool,
    pub arg_commit: Vec<String>,
    #[arg(last = true)]
    pub arg_spec: Vec<String>,
}
