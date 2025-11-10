use crate::p2p::network_config::Network;
use clap::Parser;
use libp2p::Multiaddr;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long)]
    pub secret: Option<u8>,

    #[clap(long)]
    pub peer: Option<String>,

    #[clap(long)]
    pub multiaddr: Option<Multiaddr>,

    #[clap(long, value_enum, default_value = &"ipfs")]
    pub network: Option<Network>,

    #[clap(long)]
    pub flag_topo_order: bool,
    #[clap(long)]
    pub flag_date_order: bool,
    #[clap(long)]
    pub flag_reverse: bool,
    #[clap(long)]
    pub flag_author: Option<String>,
    #[clap(long)]
    pub flag_committer: Option<String>,
    #[clap(long = "grep")]
    pub flag_grep: Option<String>,
    #[clap(long = "git-dir")]
    pub flag_git_dir: Option<String>,
    #[clap(long)]
    pub flag_skip: Option<usize>,
    #[clap(short = 'n', long)]
    pub flag_max_count: Option<usize>,
    #[clap(long)]
    pub flag_merges: bool,
    #[clap(long)]
    pub flag_no_merges: bool,
    #[clap(long)]
    pub flag_no_min_parents: bool,
    #[clap(long)]
    pub flag_no_max_parents: bool,
    #[clap(long)]
    pub flag_max_parents: Option<usize>,
    #[clap(long)]
    pub flag_min_parents: Option<usize>,
    #[clap(long, short)]
    pub flag_patch: bool,
    pub arg_commit: Vec<String>,
    #[clap(last = true)]
    pub arg_spec: Vec<String>,
}
