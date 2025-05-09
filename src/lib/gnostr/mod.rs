use clap::{Args, Parser};
use libp2p::gossipsub;
use std::{error::Error, time::Duration};
use tokio::{io, io::AsyncBufReadExt};
use tracing::debug;
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use git2::{ObjectType, Repository};
use nostr_sdk_0_37_0::prelude::*;

use gnostr_asyncgit::sync::RepoPath;

use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct GnostrSubCommands {
    /// theme path
    #[arg(long = "theme", default_value = "theme.rom")]
    pub theme: Option<PathBuf>,
    /// repo path
    #[arg(long = "repo_path", default_value = ".")]
    pub repo_path: Option<RepoPath>,
    /// Enable notify_watcher
    #[clap(
        long,
        value_name = "NOTIFY_WATCHER",
        help = "gnostr --notify_watcher",
        default_value = "false"
    )]
    pub notify_watcher: bool,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    #[arg(long, global = true)]
    pub name: Option<String>,
    ///// chat topic
    #[arg(long, global = true)]
    pub topic: Option<String>,
    ///// chat hash
    #[arg(long, global = true)]
    pub hash: Option<String>,
    ///// disable spinner animations
    #[arg(long, action, default_value = "false")]
    pub disable_cli_spinners: bool,
    #[arg(long, action)]
    pub info: bool,
    #[arg(long, action)]
    pub debug: bool,
    #[arg(long, action)]
    pub trace: bool,
}

pub async fn gnostr(sub_command_args: &GnostrSubCommands) -> Result<(), Box<dyn Error>> {
	let _ = crate::tui::tui().await;
    //let args: ChatCli = ChatCli::parse();

    let args = sub_command_args.clone();

    if let Some(hash) = args.hash {
        debug!("hash={}", hash);
    };

    if let Some(name) = args.name {
        use std::env;
        env::set_var("USER", &name);
    };

    let level = if args.debug {
        LevelFilter::DEBUG
    } else if args.trace {
        LevelFilter::TRACE
    } else if args.info {
        LevelFilter::INFO
    } else {
        LevelFilter::OFF
    };

    let filter = EnvFilter::default()
        .add_directive(level.into())
        .add_directive("nostr_sdk=off".parse().unwrap())
        .add_directive("nostr_sdk::relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::client::handler=off".parse().unwrap())
        .add_directive("nostr_relay_pool=off".parse().unwrap())
        .add_directive("nostr_relay_pool::relay=off".parse().unwrap())
        .add_directive("nostr_relay_pool::relay::inner=off".parse().unwrap())
        .add_directive("nostr_sdk::relay::connection=off".parse().unwrap())
        //.add_directive("nostr_sdk::relay::*,off".parse().unwrap())
        .add_directive("gnostr::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap())
        .add_directive("gnostr::nostr_proto=off".parse().unwrap());

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(filter);

    let _ = subscriber.try_init();

    //initialize git repo
    let repo = Repository::discover(".").expect("");

    //gather some repo info
    //find HEAD
    let head = repo.head().expect("");
    let obj = head
        .resolve()
        .expect("")
        .peel(ObjectType::Commit)
        .expect("");

    //read top commit
    let commit = obj.peel_to_commit().expect("");
    let commit_id = commit.id().to_string();
    //some info wrangling
    println!("commit_id:\n{}", commit_id);

    println!("{:?}", &sub_command_args.clone());
	let _ = crate::tui::tui().await;

    Ok(())
}
