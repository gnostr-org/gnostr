use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use git2::{ObjectType, Repository};
use crate::queue::InternalEvent;
use libp2p::gossipsub;
use nostr_sdk_0_37_0::prelude::*;
//use nostr_sdk_0_37_0::EventBuilder;
use once_cell::sync::OnceCell;
use serde_json;
//use sha2::Digest;
//use tokio::time::Duration;

use std::{error::Error, time::Duration};
use tokio::{io, io::AsyncBufReadExt};
use tracing_subscriber::util::SubscriberInitExt;
//use tracing::debug;
use tracing::{debug, info};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};



pub mod msg;
pub use msg::*;
pub mod p2p;
pub use p2p::evt_loop;
pub mod ui;
pub mod tests;



/// Simple CLI application to interact with nostr
#[derive(Debug, Parser)]
#[command(name = "gnostr")]
#[command(author = "gnostr <admin@gnostr.org>, 0xtr. <oxtrr@protonmail.com")]
#[command(version = "0.0.1")]
#[command(author, version, about, long_about = "long_about")]
pub struct ChatCli {
    /// Name of the person to greet
    #[arg(
        long,
        value_name = "NAME",
        help = "gnostr --name <string>",
        /*default_value = ""*/ //decide whether to allow env var $USER as default
    )]
    pub name: Option<String>,
    ///
    #[arg(short, long, value_name = "NSEC", help = "gnostr --nsec <sha256>",
		action = clap::ArgAction::Append,
		default_value = "0000000000000000000000000000000000000000000000000000000000000001")]
    pub nsec: Option<String>,
    ///
    #[arg(long, value_name = "HASH", help = "gnostr --hash <string>")]
    pub hash: Option<String>,
    ///
    #[arg(long, value_name = "CHAT", help = "gnostr chat")]
    pub chat: Option<String>,
    ///
    #[arg(long, value_name = "TOPIC", help = "gnostr --topic <string>")]
    pub topic: Option<String>,
    ///
    #[arg(short, long, value_name = "RELAYS", help = "gnostr --relays <string>",
		action = clap::ArgAction::Append,
		default_values_t = ["wss://relay.damus.io".to_string(),"wss://nos.lol".to_string(), "wss://nostr.band".to_string()])]
    pub relays: Vec<String>,
    /// Enable debug logging
    #[clap(
        long,
        value_name = "DEBUG",
        help = "gnostr --debug",
        default_value = "false"
    )]
    pub debug: bool,
    /// Enable info logging
    #[clap(
        long,
        value_name = "INFO",
        help = "gnostr --info",
        default_value = "false"
    )]
    pub info: bool,
    /// Enable trace logging
    #[clap(
        long,
        value_name = "TRACE",
        help = "gnostr --trace",
        default_value = "false"
    )]
    pub trace: bool,
    #[arg(long = "cfg", default_value = "")]
    pub config: String,
}

#[derive(Args, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct ChatSubCommands {
    //#[command(subcommand)]
    //command: ChatCommands,
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
    /// Send a single message to a topic and exit
    #[arg(long, global = true, requires = "topic")]
    pub oneshot: Option<String>,
}

//async tasks
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

pub async fn chat(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    let args = sub_command_args.clone();

    if let Some(hash) = args.hash.clone() {
        debug!("hash={}", hash);
    };

    if let Some(name) = args.name.clone() {
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
        .add_directive("gnostr::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap())
        .add_directive("gnostr::nostr_proto=off".parse().unwrap());

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(filter);

    let _ = subscriber.try_init();

    if let Some(message) = args.oneshot {
        info!("Oneshot mode: sending message '{}'", message);

        let topic_str = args.topic.expect("--topic is required with --oneshot");
        let topic = gossipsub::IdentTopic::new(topic_str);

        let (peer_tx, _peer_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
        let (input_tx, input_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);

        let _p2p_handle = tokio::spawn(async move {
            if let Err(e) = evt_loop(input_rx, peer_tx, topic).await {
                eprintln!("p2p event loop error: {}", e);
            }
        });

        // Allow time for network initialization and peer discovery.
        println!("Initializing network and discovering peers...");
        tokio::time::sleep(Duration::from_secs(3)).await;

        let msg = Msg::default().set_content(message, 0);
        if input_tx.send(InternalEvent::ChatMessage(msg)).await.is_err() {
            eprintln!("Failed to send message to event loop.");
        } else {
            println!("Message sent. Waiting for propagation...");
        }

        // Allow time for the message to propagate.
        tokio::time::sleep(Duration::from_secs(2)).await;

        println!("Oneshot operation complete.");
        return Ok(());
    }

    tokio::task::spawn_blocking(move || {
        let repo = Repository::discover(".")?;
        let head = repo.head()?;
        let obj = head.resolve()?.peel(ObjectType::Commit)?;
        let commit = obj.peel_to_commit()?;
        let commit_id = commit.id().to_string();

        let mut app = ui::App::default();
        app.topic = args.topic.unwrap_or_else(|| commit_id.to_string());

        let (peer_tx, mut peer_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
        let (input_tx, input_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);

        let value = input_tx.clone();
        app.on_submit(move |m| {
            let value = value.clone();
            global_rt().spawn(async move {
                debug!("sent: {:?}", m);
                value.send(InternalEvent::ChatMessage(m)).await.unwrap();
            });
        });

        let topic = gossipsub::IdentTopic::new(format!("{}", app.topic.clone()));

        global_rt().spawn(async move {
            evt_loop(input_rx, peer_tx, topic).await.unwrap();
        });

        let mut tui_msg_adder = app.add_msg_fn();
        global_rt().spawn(async move {
            while let Some(event) = peer_rx.recv().await {
                debug!("recv: {:?}", event);
                if let InternalEvent::ChatMessage(m) = event {
                    tui_msg_adder(m);
                } else {
                    debug!("Received non-chat message event: {:?}", event);
                }
            }
        });

        let input_tx_clone = input_tx.clone();
        global_rt().spawn(async move {
            tokio::time::sleep(Duration::from_millis(1000)).await;
            input_tx_clone
                .send(InternalEvent::ChatMessage(Msg::default().set_kind(MsgKind::Join)))
                .await
                .unwrap();
        });

        app.run().map_err(|e| anyhow!(e.to_string()))?;

        let _ = input_tx.send(InternalEvent::ChatMessage(Msg::default().set_kind(MsgKind::Leave)));
        std::thread::sleep(Duration::from_millis(500));
        Ok(())
    }).await?
}

pub async fn input_loop(
    self_input: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = stdin.next_line().await? {
        let msg = Msg::default().set_content(line, 0);
        if let Ok(b) = serde_json::to_vec(&msg) {
            self_input.send(b).await?;
        }
    }
    Ok(())
}
