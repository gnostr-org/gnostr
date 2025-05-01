#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]
use crate::cli::ChatCommands;
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::list;
use crate::sub_commands::login;
use crate::sub_commands::pull;
use crate::sub_commands::push;
use crate::sub_commands::send;
use clap::Args;
use nostr_sdk::prelude::*;
use serde::ser::StdError;

use anyhow::Result;

//use anyhow::Result;
use asyncgit::sync::commit::{deserialize_commit, serialize_commit};
use clap::{Parser /*, Subcommand*/};
use git2::{ObjectType, Repository};
//use gnostr::chat::create_event;
//use gnostr::chat::msg::*;
//use gnostr::chat::p2p::evt_loop;
//use gnostr::chat::parse_json;
//use gnostr::chat::split_json_string;
//use gnostr::chat::ui;
//use gnostr::chat::ChatCli;
//use gnostr::global_rt::global_rt;

use libp2p::gossipsub;
use nostr_sdk_0_37_0::prelude::*;
use nostr_sdk_0_37_0::EventBuilder;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::{error::Error, time::Duration};
use tracing::{debug, info, Level};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};




#[derive(Args)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ChatSubCommand {
    #[command(subcommand)]
    command: ChatCommands,
    ///// nsec or hex private key
    #[arg(short, long, global = true)]
    nsec: Option<String>,
    ///// password to decrypt nsec
    #[arg(short, long, global = true)]
    password: Option<String>,
    ///// chat topic
    #[arg(short, long, global = true)]
    topic: Option<String>,
    ///// disable spinner animations
    #[arg(long, action)]
    disable_cli_spinners: bool,
}

pub async fn chat(sub_command_args: &ChatSubCommand) -> Result<(), Box<dyn StdError>> {
    match &sub_command_args.command {
        ChatCommands::Login(args) => login::launch(&args).await?,
        ChatCommands::Init(args) => init::launch(&args).await?,
        ChatCommands::Send(args) => send::launch(&args, true).await?,
        ChatCommands::List => list::launch().await?,
        ChatCommands::Pull => pull::launch().await?,
        ChatCommands::Push(args) => push::launch(&args).await?,
        ChatCommands::Fetch(args) => fetch::launch(&args).await?,
    }
    //continue





	Ok(())
}
