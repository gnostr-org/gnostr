use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use git2::{ObjectType, Repository};

use self::msg::{Msg, MsgKind};
use crate::queue::InternalEvent;
use crate::types::nip28::CREATE_CHANNEL_MESSAGE;
use crate::types::{Error, EventV3, Id, Metadata, Signer, UncheckedUrl};
use gnostr_asyncgit::sync::commit::padded_commit_id;
use gnostr_asyncgit::sync::RepoPath;
use libp2p::gossipsub;
use once_cell::sync::OnceCell;
use serde_json; // Explicitly added for clarity

use std::path::PathBuf;
use std::{error::Error as StdError, time::Duration};
use textwrap::{fill, Options};
use uuid::Uuid;
use crate::types::metadata::{DEFAULT_AVATAR, DEFAULT_BANNER};
//use async_std::path::PathBuf;

use tokio::{io, io::AsyncBufReadExt};
use tracing::{debug, info};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

pub mod msg;
pub use msg::*;
pub mod p2p;
pub use p2p::evt_loop;
pub mod tests;
pub mod ui;

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

    #[arg(short, long, value_name = "NSEC", help = "gnostr --nsec <sha256>",
		action = clap::ArgAction::Append,
		default_value = "0000000000000000000000000000000000000000000000000000000000000001")]
    pub nsec: Option<String>,

    #[arg(long, value_name = "HASH", help = "gnostr --hash <string>")]
    pub hash: Option<String>,

    #[arg(long, value_name = "CHAT", help = "gnostr chat")]
    pub chat: Option<String>,

    #[arg(long, value_name = "TOPIC", help = "gnostr --topic <string>")]
    pub topic: Option<String>,

    #[arg(short, long, value_name = "RELAYS", help = "gnostr --relays <string>",
		action = clap::ArgAction::Append,
		default_values_t = ["wss://relay.damus.io".to_string(),"wss://nos.lol".to_string(), "wss://nostr.band".to_string()])]
    pub relays: Vec<String>,
    /// Enable debug logging
    #[arg(
        long,
        value_name = "DEBUG",
        help = "gnostr --debug",
        default_value = "false"
    )]
    pub debug: bool,
    /// Enable info logging
    #[arg(
        long,
        value_name = "INFO",
        help = "gnostr --info",
        default_value = "false"
    )]
    pub info: bool,
    /// Enable trace logging
    #[arg(
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
    // nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    // password to decrypt nsec
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    #[arg(long, global = true)]
    pub name: Option<String>,
    // chat topic
    #[arg(long, global = true)]
    pub topic: Option<String>,
    // chat hash
    #[arg(long, global = true)]
    pub hash: Option<String>,
    // disable spinner animations
    #[arg(long, default_value_t = false)]
    pub disable_cli_spinners: bool,
    #[arg(long, default_value_t = false)]
    pub info: bool,
    #[arg(long)]
    pub debug: bool,
    #[arg(long)]
    pub trace: bool,
    /// workdir
    pub workdir: Option<String>,
    #[arg(
        long,
        value_name = "GITDIR",
        default_value = ".",
        help = "gnostr --gitdir '<string>'"
    )]
    /// gitdir
    pub gitdir: Option<RepoPath>,
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
        .add_directive("hickory_proto=off".parse().unwrap())
        .add_directive("hickory_proto::rr=off".parse().unwrap())
        .add_directive("hickory_proto::rr::record_data=off".parse().unwrap())
        .add_directive("libp2p_mdns=off".parse().unwrap())
        //.add_directive("gnostr::p2p=off".parse().unwrap())
        //.add_directive("gnostr::p2p::chat=off".parse().unwrap())
        //.add_directive("gnostr::p2p::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap());

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(filter);

    let _ = subscriber.try_init();

    // Determine the KeySigner to use
    let nsec_hex = if let Some(nsec) = args.nsec.clone() {
        nsec
    } else if let Some(hash) = args.hash.clone() {
        format!("{:0>64}", hash)
    } else {
        //args.nsec = padded_commit_id("0".to_string())

        // Fallback to generate a new key if no nsec or hash is provided.
        // For now, use a fixed dummy private key for simplicity in testing.
        // TODO: Implement proper key generation.
        "0000000000000000000000000000000000000000000000000000000000000001".to_string()
    };
    let private_key = crate::types::PrivateKey::try_from_hex_string(&nsec_hex).unwrap();
    let keys = crate::types::KeySigner::from_private_key(private_key, "", 1).unwrap();
    let public_key = keys.public_key();

    // Initialize NostrClient and channels
    let (peer_tx, _peer_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
    let client = crate::types::nostr_client::NostrClient::new(peer_tx.clone());

    // Send NIP-01 metadata event
    let name = args
        .name
        .clone()
        .unwrap_or_else(|| public_key.as_hex_string().chars().take(8).collect());
    let metadata = {
        let mut m = Metadata::default();
        m.name = Some(name);
        m.picture = Some(DEFAULT_AVATAR.to_string());
        m.other.insert("banner".to_string(), serde_json::Value::String(DEFAULT_BANNER.to_string()));
        m
    };

    let pre_event = crate::types::PreEvent {
        pubkey: public_key,
        created_at: crate::types::Unixtime::now(),
        kind: crate::types::EventKind::Metadata,
        tags: vec![],
        content: serde_json::to_string(&metadata).unwrap(),
    };

    let id = pre_event.hash().unwrap();
    let sig = keys.sign_id(id).unwrap();

    let metadata_event = EventV3 {
        id,
        pubkey: pre_event.pubkey,
        created_at: pre_event.created_at,
        kind: pre_event.kind,
        tags: pre_event.tags,
        content: pre_event.content,
        sig,
    };

    client.send_event(metadata_event).await?;
    info!("NIP-01 metadata event sent successfully.");

    // Define topic outside oneshot block
    let topic = gossipsub::IdentTopic::new(
        args.topic.clone().unwrap_or_else(|| "gnostr".to_string()), // Default topic
    );

    if let Some(message_input) = args.oneshot {
        info!("Oneshot mode: sending message '{}'", message_input);

        let _p2p_handle = tokio::spawn(async move {
            if let Err(e) = evt_loop(input_rx, peer_tx, topic.clone()).await {
                eprintln!("p2p event loop error: {}", e);
            }
        });

        // Allow time for network initialization and peer discovery.
        println!("Initializing network and discovering peers...");
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Detect if message_input is a git diff
        let mut msg_kind = MsgKind::OneShot;
        if message_input.contains("diff --git") || (message_input.contains("--- a/") && message_input.contains("+++ b/")) {
            msg_kind = MsgKind::GitDiff;
        }

        // Create a single Msg object with the entire message_input
        let msg = Msg::default()
            .set_kind(msg_kind)
            .set_content(message_input.clone(), 0); // Use message_input directly

        if input_tx
            .send(InternalEvent::ChatMessage(msg))
            .await
            .is_err()
        {
            eprintln!("Failed to send message to event loop.");
        } else {
            println!("Oneshot message sent. Waiting for propagation...");
        }
        
        // Allow time for the message to propagate.
        tokio::time::sleep(Duration::from_secs(2)).await;

        println!("Oneshot operation complete.");
        return Ok(());
    }

    tokio::task::spawn_blocking(move || {
        let search_path: PathBuf = match &args.gitdir {
            Some(repo_path) => match repo_path {
                RepoPath::Path(p) => p.clone(),
                _ => panic!("Unsupported RepoPath variant"),
            },
            // If no gitdir arg was provided, default to current directory "."
            None => PathBuf::from("."), //TODO $HOME/.gnostr
        };

        let repo = Repository::discover(search_path)?;
        let head = repo.head()?;
        let obj = head.resolve()?.peel(ObjectType::Commit)?;
        let commit = obj.peel_to_commit()?;
        let commit_id = commit.id().to_string();

        // TODO
        let _padded_commit_id = format!("{:0>64}", commit_id.clone());

        let mut app = ui::App {
            topic: args.topic.unwrap_or_else(|| commit_id.to_string()),
            ..Default::default()
        };

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

        let topic = gossipsub::IdentTopic::new(app.topic.clone().to_string());

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
                .send(InternalEvent::ChatMessage(
                    Msg::default().set_kind(MsgKind::Join),
                ))
                .await
                .unwrap();
        });

        app.run().map_err(|e| anyhow!(e.to_string()))?;

        let _ = input_tx.send(InternalEvent::ChatMessage(
            Msg::default().set_kind(MsgKind::Leave),
        ));
        std::thread::sleep(Duration::from_millis(500));
        Ok(())
    })
    .await?
}

pub async fn input_loop(
    self_input: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> Result<(), Box<dyn StdError>> {
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = stdin.next_line().await? {
        let msg = Msg::default().set_content(line, 0);
        if let Ok(b) = serde_json::to_vec(&msg) {
            self_input.send(b).await?;
        }
    }
    Ok(())
}
