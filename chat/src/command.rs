use anyhow::{anyhow, Result};
use clap::Parser;
use gnostr_asyncgit::types::PrivateKey;
use libp2p::gossipsub;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{
    event::ChatEvent,
    msg::{Msg, MsgKind},
    p2p::evt_loop,
};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ChatSubCommands {
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    #[arg(long, global = true)]
    pub name: Option<String>,
    #[arg(long, global = true, default_value = "gnostr")]
    pub topic: Option<String>,
    #[arg(long, global = true)]
    pub hash: Option<String>,
    #[arg(long, default_value_t = false)]
    pub disable_cli_spinners: bool,
    #[arg(long, default_value_t = false)]
    pub info: bool,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub trace: bool,
    #[arg(long, default_value_t = false, help = "Run in headless mode (no TUI)")]
    pub headless: bool,
    pub workdir: Option<String>,
    #[arg(long, value_name = "GITDIR", default_value = ".", help = "gnostr --gitdir '<string>'")]
    pub gitdir: Option<gnostr_asyncgit::sync::RepoPath>,
    #[arg(short = '1', long, global = true, requires = "topic")]
    pub oneshot: Option<String>,
}

pub async fn chat(sub_command_args: &ChatSubCommands) -> Result<()> {
    run(sub_command_args).await
}

pub async fn run(sub_command_args: &ChatSubCommands) -> Result<()> {
    let username_to_set: Option<String> = if let Some(name) = sub_command_args.name.clone() {
        Some(name)
    } else if let Some(nsec_hex) = sub_command_args.nsec.clone() {
        match PrivateKey::try_from_hex_string(&nsec_hex) {
            Ok(private_key) => {
                let public_key = private_key.public_key();
                Some(public_key.as_hex_string().chars().take(8).collect())
            }
            Err(e) => {
                tracing::warn!(
                    "Could not derive public key from --nsec due to error: {}. USER env var will not be set from nsec.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    if let Some(user_name) = username_to_set {
        if !user_name.is_empty() {
            use std::env;
            unsafe { env::set_var("USER", &user_name) };
            tracing::debug!("USER environment variable set to: {}", user_name);
        }
    }

    let level = if sub_command_args.debug {
        Level::DEBUG
    } else if sub_command_args.trace {
        Level::TRACE
    } else if sub_command_args.info {
        Level::INFO
    } else {
        Level::WARN
    };

    let filter = EnvFilter::default()
        .add_directive(level.into())
        .add_directive("nostr_sdk=off".parse().unwrap())
        .add_directive("nostr_sdk::relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::client=off".parse().unwrap())
        .add_directive("nostr_sdk::client::handler=off".parse().unwrap())
        .add_directive("nostr_relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::relay::connection=off".parse().unwrap())
        .add_directive("gnostr::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap())
        .add_directive("gnostr::nostr_proto=off".parse().unwrap())
        .add_directive("libp2p_mdns::behaviour::iface=off".parse().unwrap())
        .add_directive("libp2p_gossipsub::behaviour=off".parse().unwrap());

    let subscriber = Registry::default()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_thread_ids(true),
        )
        .with(filter);

    let _ = subscriber.try_init();
    tracing::trace!("\n{:?}\n", &sub_command_args);
    tracing::debug!("\n{:?}\n", &sub_command_args);
    tracing::info!("\n{:?}\n", &sub_command_args);

    run_chat_session(sub_command_args).await
}

async fn run_chat_session(sub_command_args: &ChatSubCommands) -> Result<()> {
    let topic = gossipsub::IdentTopic::new(
        sub_command_args
            .topic
            .clone()
            .unwrap_or_else(|| "gnostr".to_string()),
    );
    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);
    let (output_tx, mut output_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);

    tokio::spawn(async move {
        let _ = evt_loop(input_rx, output_tx, topic).await;
    });

    use tokio::io::AsyncBufReadExt;
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let Some(line) = line? else { break; };
                if line.is_empty() {
                    continue;
                }

                let msg = Msg::default().set_content(line, 0).set_kind(MsgKind::Chat);
                input_tx.send(ChatEvent::ChatMessage(msg)).await?;
            }
            Some(event) = output_rx.recv() => {
                match event {
                    ChatEvent::ChatMessage(msg) => println!("{msg}"),
                    ChatEvent::ShowErrorMsg(text) => eprintln!("{text}"),
                    ChatEvent::ShowInfoMsg(text) => println!("{text}"),
                }
            }
        }
    }

    Ok(())
}
