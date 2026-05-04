use anyhow::Result;
use clap::Parser;
use gnostr_asyncgit::types::PrivateKey;
use libp2p::gossipsub;
use proctitle::set_title;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{
    event::ChatEvent,
    msg::{Msg, MsgKind},
    p2p::evt_loop,
    tui::run_chat_tui,
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

    let username_for_session = username_to_set
        .clone()
        .unwrap_or_else(|| "gnostr".to_string());

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
        .add_directive("libp2p_gossipsub::behaviour=off".parse().unwrap())
        .add_directive("tokio_tungstenite=off".parse().unwrap());

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

    if let Some(message_input) = sub_command_args.oneshot.clone() {
        if sub_command_args.headless {
            println!("headless conflicts with oneshot!");
            return Ok(());
        }

        tracing::info!("Oneshot mode: sending message '{}'", message_input);
        let topic = chat_topic(sub_command_args);
        let (input_tx, input_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);
        let (output_tx, output_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);

        tokio::spawn(async move {
            let _ = evt_loop(input_rx, output_tx, topic).await;
        });

        tokio::spawn(async move {
            while let Some(event) = output_rx.recv().await {
                match event {
                    ChatEvent::ChatMessage(msg) => println!("{msg}"),
                    ChatEvent::ShowErrorMsg(text) => eprintln!("{text}"),
                    ChatEvent::ShowInfoMsg(text) => println!("{text}"),
                    ChatEvent::CrawlerSearch { .. } => {}
                }
            }
        });

        println!("Initializing network and discovering peers...");
        tokio::time::sleep(Duration::from_secs(3)).await;

        let mut msg_kind = MsgKind::OneShot;
        if message_input.contains("diff --git")
            || (message_input.contains("--- a/") && message_input.contains("+++ b/"))
        {
            msg_kind = MsgKind::GitDiff;
        }

        let msg = Msg::default()
            .set_kind(msg_kind)
            .set_content(message_input.clone(), 0);

        if input_tx.send(ChatEvent::ChatMessage(msg)).await.is_err() {
            eprintln!("Failed to send message to event loop.");
        } else {
            println!("Oneshot message sent. Waiting for propagation...");
        }

        tokio::time::sleep(Duration::from_secs(2)).await;
        tracing::info!("Oneshot operation complete.");
        return Ok(());
    }

    if sub_command_args.headless {
        let topic = chat_topic(sub_command_args);
        let topic_name = sub_command_args
            .topic
            .clone()
            .unwrap_or_else(|| "gnostr".to_string());
        let process_title = format!("gnostr-chat-{topic_name}");
        set_title(&process_title);
        println!("Headless mode enabled:");
        tracing::info!("running event loop in background.");
        tracing::info!("Process name set to: {}", process_title);

        let (input_tx, input_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);
        let (output_tx, _output_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);

        tokio::spawn(async move {
            if let Err(e) = evt_loop(input_rx, output_tx, topic).await {
                eprintln!("Headless p2p event loop error: {}", e);
            }
        });

        let _ = input_tx;
        tokio::time::sleep(Duration::from_millis(100)).await;
        return Ok(());
    }

    run_chat_session(sub_command_args, username_for_session).await
}

fn chat_topic(sub_command_args: &ChatSubCommands) -> gossipsub::IdentTopic {
    gossipsub::IdentTopic::new(
        sub_command_args
            .topic
            .clone()
            .unwrap_or_else(|| "gnostr".to_string()),
    )
}

async fn run_chat_session(sub_command_args: &ChatSubCommands, username: String) -> Result<()> {
    let topic_name = sub_command_args
        .topic
        .clone()
        .unwrap_or_else(|| "gnostr".to_string());
    let topic = chat_topic(sub_command_args);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);
    let (output_tx, output_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);

    tokio::spawn(async move {
        let _ = evt_loop(input_rx, output_tx, topic).await;
    });

    run_chat_tui(topic_name, username, input_tx, output_rx)?;
    Ok(())
}
