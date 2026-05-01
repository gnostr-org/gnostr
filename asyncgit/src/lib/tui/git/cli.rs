use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "gnostr-asyncgit", disable_version_flag = true)]
pub struct Args {
    #[arg(long)]
    pub version: bool,

    #[arg(long)]
    pub log: bool,

    #[arg(long)]
    pub print: bool,

    #[arg(long)]
    pub keys: Option<String>,
}
