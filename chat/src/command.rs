use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use gnostr_asyncgit::types::PrivateKey;
use proctitle::set_title;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::{
    session::{ChatNotification, ChatSession},
    tui::run_chat_tui,
};

#[derive(Debug, Clone, Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help = "Examples:\n  gnostr chat --topic gnostr-dev --name copilot\n  gnostr chat --topic gnostr-dev --name copilot --oneshot \"hello\"\n  gnostr chat --headless --topic gnostr-dev --nsec <hex-key>"
)]
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

    let filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env()
        .expect("Failed to build EnvFilter from environment")
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
    tracing::debug!(
        "chat startup: topic={:?} headless={} oneshot={:?} gitdir={:?}",
        sub_command_args.topic,
        sub_command_args.headless,
        sub_command_args.oneshot,
        sub_command_args.gitdir
    );
    tracing::trace!("\n{:?}\n", &sub_command_args);
    tracing::debug!("\n{:?}\n", &sub_command_args);
    tracing::info!("\n{:?}\n", &sub_command_args);

    let topic_name = chat_topic(sub_command_args);
    let mut session = ChatSession::connect(topic_name.clone()).await?;
    tracing::info!("local p2p relay service started for chat");

    if let Some(message_input) = sub_command_args.oneshot.clone() {
        if sub_command_args.headless {
            println!("headless conflicts with oneshot!");
            return Ok(());
        }

        run_oneshot(&mut session, message_input).await?;
        return Ok(());
    }

    if sub_command_args.headless {
        let topic_name = sub_command_args
            .topic
            .clone()
            .unwrap_or_else(|| "gnostr".to_string());
        let process_title = format!("gnostr-chat-{topic_name}");
        set_title(&process_title);
        println!("Headless mode enabled:");
        tracing::info!("running event loop in background.");
        tracing::info!("Process name set to: {}", process_title);

        let printer = spawn_notification_printer(session.subscribe());
        tracing::info!("Headless mode is running; waiting for shutdown.");
        tokio::signal::ctrl_c().await?;
        tracing::debug!("headless chat received ctrl-c and is exiting");
        drop(printer);
        return Ok(());
    }

    run_chat_session(sub_command_args, username_for_session, &session).await
}

fn chat_topic(sub_command_args: &ChatSubCommands) -> String {
    sub_command_args
        .topic
        .clone()
        .unwrap_or_else(|| "gnostr".to_string())
}

async fn run_chat_session(sub_command_args: &ChatSubCommands, username: String, session: &ChatSession) -> Result<()> {
    let topic_name = sub_command_args
        .topic
        .clone()
        .unwrap_or_else(|| "gnostr".to_string());

    run_chat_tui(topic_name, username, session)?;
    Ok(())
}

async fn run_oneshot(session: &mut ChatSession, message_input: String) -> Result<()> {
    tracing::info!("Oneshot mode: sending message '{}'", message_input);
    let printer = spawn_notification_printer(session.subscribe());
    println!("Initializing network and discovering peers...");
    tracing::debug!("chat oneshot: waiting for a connected peer");
    session.wait_for_connected(Duration::from_secs(30)).await?;
    session.send_text(message_input).await?;
    println!("Oneshot message sent. Waiting for propagation...");

    tokio::time::sleep(Duration::from_secs(2)).await;
    drop(printer);
    tracing::info!("Oneshot operation complete.");
    Ok(())
}

fn spawn_notification_printer(
    mut output_rx: tokio::sync::broadcast::Receiver<ChatNotification>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(event) = output_rx.recv().await {
            match event {
                ChatNotification::ChatMessage(msg) => println!("{msg}"),
                ChatNotification::Error(text) => eprintln!("{text}"),
                ChatNotification::Info(text) => println!("{text}"),
                ChatNotification::Connected { peer_id, endpoint } => {
                    println!("Connected to peer {peer_id} via {endpoint}")
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_topic_defaults_to_gnostr() {
        let args = ChatSubCommands {
            nsec: None,
            password: None,
            name: None,
            topic: None,
            hash: None,
            disable_cli_spinners: false,
            info: false,
            debug: false,
            trace: false,
            headless: false,
            workdir: None,
            gitdir: None,
            oneshot: None,
        };

        assert_eq!(chat_topic(&args), "gnostr");
    }

    #[test]
    fn chat_topic_uses_explicit_topic() {
        let args = ChatSubCommands {
            topic: Some("gnostr-dev".to_string()),
            ..ChatSubCommands {
                nsec: None,
                password: None,
                name: None,
                topic: None,
                hash: None,
                disable_cli_spinners: false,
                info: false,
                debug: false,
                trace: false,
                headless: false,
                workdir: None,
                gitdir: None,
                oneshot: None,
            }
        };

        assert_eq!(chat_topic(&args), "gnostr-dev");
    }
}
