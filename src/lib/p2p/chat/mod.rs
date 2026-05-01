//! Chat command plumbing for `gnostr`.
//!
//! This module binds the chat CLI, terminal UI, local swarm lifecycle, and the
//! Blossom-backed file transfer / git-clone path into one command surface.

use std::{error::Error as StdError, fs, path::PathBuf, process::Command, time::Duration};

use anyhow::{anyhow, Context, Result};
use clap::{Args, Parser};
use git2::{ObjectType, Repository};
use gnostr_asyncgit::sync::{commit::padded_commit_id, resolve_repo_path, RepoPath};
use libp2p::gossipsub;
use once_cell::sync::OnceCell;
use proctitle::set_title;
use serde_json; // Explicitly added for clarity
use textwrap::{fill, Options};
//use async_std::path::PathBuf;
use tokio::{io, io::AsyncBufReadExt};
use tracing::{debug, info};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use uuid::Uuid;

pub use gnostr_chat::{msg, p2p};
pub use gnostr_chat::evt_loop;
pub use gnostr_chat::ChatEvent;
use gnostr_chat::msg::{Msg, MsgKind};

use crate::queue::InternalEvent;
use gnostr_asyncgit::{
    //queue::InternalEvent,
    types::{
        metadata::{DEFAULT_AVATAR, DEFAULT_BANNER},
        nip28::CREATE_CHANNEL_MESSAGE,
        Error, EventV3, Id, Metadata, Signer, TagV3, UncheckedUrl,
    },
};
use sha2::{Digest, Sha256};

// pub mod ui; // disabled while ratatui/crossterm are being aligned

fn to_transport_event(event: InternalEvent) -> Option<ChatEvent> {
    match event {
        InternalEvent::ChatMessage(msg) => Some(ChatEvent::ChatMessage(msg)),
        InternalEvent::ShowErrorMsg(text) => Some(ChatEvent::ShowErrorMsg(text)),
        InternalEvent::ShowInfoMsg(text) => Some(ChatEvent::ShowInfoMsg(text)),
        _ => None,
    }
}

fn from_transport_event(event: ChatEvent) -> InternalEvent {
    match event {
        ChatEvent::ChatMessage(msg) => InternalEvent::ChatMessage(msg),
        ChatEvent::ShowErrorMsg(text) => InternalEvent::ShowErrorMsg(text),
        ChatEvent::ShowInfoMsg(text) => InternalEvent::ShowInfoMsg(text),
    }
}

/// Simple CLI application to interact with nostr
/// Top-level CLI flags for launching `gnostr chat`.
///
/// These flags cover identity, relay selection, logging, and detached/headless
/// operation so the chat runtime can be used in interactive and workflow
/// contexts.
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

    #[arg(
        long,
        value_name = "TOPIC",
        help = "gnostr --topic <string>",
        default_value = "gnostr"
    )]
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
    /// Run in headless mode (no TUI)
    #[arg(long, default_value_t = false, help = "Run in headless mode (no TUI)")]
    pub headless: bool,
    #[arg(long = "cfg", default_value = "")]
    pub config: String,
}

/// Options carried into the chat runtime after Clap parsing.
///
/// These values control swarm identity, topic scoping, shell commands, and
/// whether a local Blossom server is started for P2P cloning and transfer.
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
    /// Chat topic used to scope the local P2P swarm and Blossom workspace.
    #[arg(long, global = true, default_value = "gnostr")]
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
    /// Run in headless mode (no TUI)
    #[arg(long, default_value_t = false, help = "Run in headless mode (no TUI)")]
    pub headless: bool,
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
    /// Send a single message to the topic and exit after propagation.
    /// Use `-1` as a short alias for `--oneshot`.
    #[arg(short = '1', long, global = true, requires = "topic")]
    pub oneshot: Option<String>,
}

/// Return the shared Tokio runtime used by background chat tasks.
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalChatCommand {
    GitClone {
        url: String,
        destination: Option<String>,
    },
}

fn parse_local_chat_command(input: &str) -> Result<Option<LocalChatCommand>> {
    if !input.starts_with('/') {
        return Ok(None);
    }

    let parts = shellwords::split(input).context("parse chat command")?;
    let Some(command) = parts.first().map(String::as_str) else {
        return Ok(None);
    };

    let parsed = match (
        command,
        parts.get(1).map(String::as_str),
        parts.get(2),
        parts.get(3),
    ) {
        ("/clone", Some(url), dest, None) => Some(LocalChatCommand::GitClone {
            url: url.to_string(),
            destination: dest.cloned(),
        }),
        ("/git", Some("clone"), Some(url), dest) => Some(LocalChatCommand::GitClone {
            url: url.to_string(),
            destination: dest.cloned(),
        }),
        ("/blossom", Some("clone"), Some(url), dest) => Some(LocalChatCommand::GitClone {
            url: url.to_string(),
            destination: dest.cloned(),
        }),
        _ => None,
    };

    Ok(parsed)
}

fn run_local_chat_command(command: LocalChatCommand, cwd: PathBuf) -> Result<String> {
    match command {
        LocalChatCommand::GitClone { url, destination } => {
            let mut cmd = Command::new("git");
            cmd.arg("clone").arg(&url);
            if let Some(destination) = destination.as_ref() {
                cmd.arg(destination);
            }
            cmd.current_dir(&cwd);

            let output = cmd.output().context("run git clone")?;
            if output.status.success() {
                Ok(format!(
                    "git clone started from chat: {}{}",
                    url,
                    destination
                        .as_ref()
                        .map(|dest| format!(" -> {dest}"))
                        .unwrap_or_default()
                ))
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if stderr.is_empty() {
                    Err(anyhow!("git clone failed with status {}", output.status))
                } else {
                    Err(anyhow!("git clone failed: {stderr}"))
                }
            }
        }
    }
}

fn sanitize_topic_dir(topic: &str) -> String {
    topic
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Return the per-topic on-disk root used for chat state and artifacts.
fn chat_repo_root(topic: &str) -> Result<PathBuf> {
    let dirs = crate::get_dirs()?;
    Ok(dirs
        .data_local_dir()
        .join("chat")
        .join(sanitize_topic_dir(topic)))
}

/// Build a Blossom server command line for the current chat peer.
///
/// Each peer gets its own port and storage directory so headless chat sessions
/// can coexist without stepping on one another.
fn chat_blossom_server_args(args: &ChatSubCommands) -> Result<Vec<String>> {
    let topic = args.topic.clone().unwrap_or_else(|| "gnostr".to_string());
    let peer_seed = format!(
        "{}:{}:{}",
        topic,
        args.name.clone().unwrap_or_default(),
        args.hash.clone().unwrap_or_default()
    );
    let digest = Sha256::digest(peer_seed.as_bytes());
    let port_offset = u16::from_be_bytes([digest[0], digest[1]]) % 10_000;
    let port = 3_000u16 + port_offset;
    let server_root = chat_repo_root(&topic)?
        .join("blossom")
        .join(format!("{port}"));
    let data_dir = server_root.join("data");
    let db_path = server_root.join("blossom.db");
    let service_name = sanitize_topic_dir(&format!(
        "chat-{}-{}",
        topic,
        args.name
            .as_deref()
            .or(args.hash.as_deref())
            .unwrap_or("peer")
    ));

    fs::create_dir_all(&data_dir)?;
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(vec![
        "--detach".to_string(),
        "--name".to_string(),
        service_name,
        "--bind".to_string(),
        format!("0.0.0.0:{port}"),
        "--base-url".to_string(),
        format!("http://localhost:{port}"),
        "--data-dir".to_string(),
        data_dir.to_string_lossy().into_owned(),
        "--db-path".to_string(),
        db_path.to_string_lossy().into_owned(),
    ])
}

/// Start the Blossom server that backs chat file transfer and clone support.
fn start_chat_blossom_server(args: &ChatSubCommands) -> Result<()> {
    crate::server::run_with_args(chat_blossom_server_args(args)?)
        .map_err(|e| anyhow!(e.to_string()))
}

#[macro_export]
macro_rules! chat_oneshot {
    ($topic:expr, $message:expr) => {
        $crate::p2p::chat::ChatSubCommands {
            nsec: None,
            password: None,
            name: None,
            topic: Some($topic.into()),
            hash: None,
            disable_cli_spinners: false,
            info: false,
            debug: false,
            trace: false,
            headless: false,
            workdir: None,
            gitdir: None,
            oneshot: Some($message.into()),
        }
    };
}

#[macro_export]
macro_rules! chat_oneshot_named {
    ($topic:expr, $message:expr) => {{
        $crate::chat_oneshot!(
            $topic,
            format!("{}::{} {}", module_path!(), function_name!(), $message)
        )
    }};
}

#[macro_export]
macro_rules! chat_debug {
    ($message:expr) => {{
        $crate::p2p::chat::msg::Msg::default()
            .set_content($crate::introspection_debug!($message), 0)
            .set_kind($crate::p2p::chat::msg::MsgKind::Debug)
    }};
}

/// Run the chat command lifecycle.
///
/// This initializes identity, optionally starts the detached headless wrapper,
/// starts the Blossom server used for P2P transfer/cloning, and then launches
/// the main chat UI or event loop.
pub async fn chat(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    let args = sub_command_args.clone();
    const DETACHED_ENV: &str = "GNOSTR_CHAT_DETACHED";

    if let Some(hash) = args.hash.clone() {
        debug!("hash={}", hash);
    };

    if let Some(name) = args.name.clone() {
        use std::env;
        unsafe { env::set_var("USER", &name) };
    };
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
    let private_key = gnostr_asyncgit::types::PrivateKey::try_from_hex_string(&nsec_hex).unwrap();
    let keys = crate::types::KeySigner::from_private_key(private_key, "", 1).unwrap();
    let public_key = keys.public_key();

    // Initialize NostrClient and channels
    let (peer_tx, _peer_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
    let client = crate::nostr_client::NostrClient::new(peer_tx.clone());

    if sub_command_args.headless && std::env::var_os(DETACHED_ENV).is_none() {
        let topic_name = args.topic.clone().unwrap_or_else(|| "gnostr".to_string());
        let process_title = format!("gnostr-chat-{}", topic_name);
        let pid = crate::utils::detach::spawn_detached_current_exe_named_with_env(
            Some(process_title.as_str()),
            std::env::args_os().skip(1),
            [(DETACHED_ENV, "1")],
        )?;
        tracing::info!("Spawned detached headless chat process (pid: {pid})");
        return Ok(());
    }

    start_chat_blossom_server(&args)?;

    // Send NIP-01 metadata event
    let name = args
        .name
        .clone()
        .unwrap_or_else(|| public_key.as_hex_string().chars().take(8).collect());
    let metadata = {
        let mut m = Metadata::default();
        m.name = Some(name);
        m.picture = Some(DEFAULT_AVATAR.to_string());
        m.other.insert(
            "banner".to_string(),
            serde_json::Value::String(DEFAULT_BANNER.to_string()),
        );
        m
    };

    let pre_event = gnostr_asyncgit::types::PreEvent {
        pubkey: public_key,
        created_at: gnostr_asyncgit::types::Unixtime::now(),
        kind: gnostr_asyncgit::types::EventKind::Metadata,
        tags: vec![TagV3::new(&["gnostr"])],
        content: serde_json::to_string(&metadata).unwrap(),
    };

    tracing::info!("\n{:?}\n", &sub_command_args);
    println!(
        "pre_event={:?}",
        Into::<gnostr_asyncgit::types::PublicKeyHex>::into(pre_event.pubkey)
    );

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
    tracing::info!("NIP-01 metadata event sent successfully.");

    // Define topic outside oneshot block
    let topic = gossipsub::IdentTopic::new(
        args.topic.clone().unwrap_or_else(|| "gnostr".to_string()), // Default topic
    );

    let (chat_input_tx, chat_input_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);
    let (chat_output_tx, mut chat_output_rx) = tokio::sync::mpsc::channel::<ChatEvent>(100);

    {
        let peer_tx = peer_tx.clone();
        global_rt().spawn(async move {
            while let Some(event) = chat_output_rx.recv().await {
                let _ = peer_tx.send(from_transport_event(event)).await;
            }
        });
    }

    {
        let chat_input_tx = chat_input_tx.clone();
        global_rt().spawn(async move {
            let mut input_rx = input_rx;
            while let Some(event) = input_rx.recv().await {
                if let Some(event) = to_transport_event(event) {
                    let _ = chat_input_tx.send(event).await;
                }
            }
        });
    }

    global_rt().spawn(async move {
        if let Err(e) = evt_loop(chat_input_rx, chat_output_tx, topic.clone()).await {
            tracing::error!("chat event loop error: {e}");
        }
    });

    if let Some(message_input) = args.oneshot {
        if !args.headless {
            tracing::info!("Oneshot mode: sending message '{}'", message_input);

            // Allow time for network initialization and peer discovery.
            println!("Initializing network and discovering peers...");
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Detect if message_input is a git diff
            let mut msg_kind = MsgKind::OneShot;
            if message_input.contains("diff --git")
                || (message_input.contains("--- a/") && message_input.contains("+++ b/"))
            {
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
            tracing::info!("Oneshot operation complete.");
        } else {
            println!("headless conflicts with oneshot!");
        }
        return Ok(());
    }

    // In the detached child, run the event loop directly and keep the process alive.

    if sub_command_args.headless {
        let topic_name = args.topic.clone().unwrap_or_else(|| "gnostr".to_string());
        let process_title = format!("gnostr-chat-{}", topic_name);
        set_title(&process_title);
        println!("Headless mode enabled:");
        tracing::info!("running event loop in background.");
        tracing::info!("Process name set to: {}", process_title);

        std::future::pending::<()>().await;
        return Ok(());
    }

    tokio::task::spawn_blocking(move || {
        let search_path: PathBuf = match &args.gitdir {
            Some(repo_path) => resolve_repo_path(repo_path)?.as_path().to_path_buf(),
            // If no gitdir arg was provided, default to current directory "."
            None => PathBuf::from("."), //TODO $HOME/.gnostr
        };

        let (repo_root, repo) = match Repository::discover(&search_path) {
            Ok(repo) => {
                let repo_root = repo
                    .workdir()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| search_path.clone());
                (repo_root, repo)
            }
            Err(_) => {
                if let Some(topic) = args.topic.clone() {
                    let repo_root = chat_repo_root(&topic)?;
                    fs::create_dir_all(&repo_root)?;
                    let repo = Repository::init(&repo_root)?;
                    (repo_root, repo)
                } else {
                    return Err(anyhow!(
                        "not inside a git repository; run `git init` or start chat with `--topic`"
                    ));
                }
            }
        };

        let commit_id = if args.topic.is_some() {
            args.topic.clone().unwrap_or_default()
        } else {
            let head = repo.head()?;
            let obj = head.resolve()?.peel(ObjectType::Commit)?;
            let commit = obj.peel_to_commit()?;
            commit.id().to_string()
        };

        // TODO
        let _padded_commit_id = format!("{:0>64}", commit_id.clone());

        let mut app = ui::App {
            topic: args.topic.clone().unwrap_or_else(|| commit_id.to_string()),
            ..Default::default()
        };

        let (peer_tx, mut peer_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
        let (input_tx, input_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);

        let value = input_tx.clone();
        let topic_name = app.topic.clone();
        let topic = gossipsub::IdentTopic::new(topic_name.clone());
        let full_p2p_input_tx = {
            let (tx, rx) = tokio::sync::mpsc::channel::<Msg>(100);
            let (out_tx, mut out_rx) = tokio::sync::mpsc::channel::<Msg>(100);
            let full_p2p_args = args.clone();
            let full_p2p_topic = gossipsub::IdentTopic::new(topic_name.clone());

            let peer_tx_for_full = peer_tx.clone();
            let mut full_p2p_args = full_p2p_args;
            full_p2p_args.gitdir = Some(RepoPath::Path(repo_root.clone()));
            global_rt().spawn(async move {
                while let Some(msg) = out_rx.recv().await {
                    let _ = peer_tx_for_full.send(InternalEvent::ChatMessage(msg)).await;
                }
            });

            global_rt().spawn(async move {
                if let Err(e) =
                    crate::p2p::evt_loop(full_p2p_args, rx, out_tx, full_p2p_topic).await
                {
                    tracing::error!("full-feature p2p event loop error: {e}");
                }
            });

            tx
        };
        let command_tx = peer_tx.clone();
        let command_cwd = search_path.clone();
        app.on_submit(move |m| {
            let value = value.clone();
            let full_p2p_input_tx = full_p2p_input_tx.clone();
            let command_tx = command_tx.clone();
            let command_cwd = command_cwd.clone();
            global_rt().spawn(async move {
                debug!("sent: {:?}", m);
                if matches!(m.kind, MsgKind::Command) {
                    match parse_local_chat_command(&m.content[0]) {
                        Ok(Some(command)) => {
                            match tokio::task::spawn_blocking(move || {
                                run_local_chat_command(command, command_cwd)
                            })
                            .await
                            {
                                Ok(Ok(message)) => {
                                    let _ =
                                        command_tx.send(InternalEvent::ShowInfoMsg(message)).await;
                                }
                                Ok(Err(err)) => {
                                    let _ = command_tx
                                        .send(InternalEvent::ShowErrorMsg(err.to_string()))
                                        .await;
                                }
                                Err(err) => {
                                    let _ = command_tx
                                        .send(InternalEvent::ShowErrorMsg(err.to_string()))
                                        .await;
                                }
                            }
                        }
                        Ok(None) => {
                            let _ = command_tx
                                .send(InternalEvent::ShowErrorMsg(format!(
                                    "unknown chat command: {}",
                                    m.content[0]
                                )))
                                .await;
                        }
                        Err(err) => {
                            let _ = command_tx
                                .send(InternalEvent::ShowErrorMsg(err.to_string()))
                                .await;
                        }
                    }
                } else {
                    let full_msg = m.clone();
                    value.send(InternalEvent::ChatMessage(m)).await.unwrap();
                    let _ = full_p2p_input_tx.send(full_msg).await;
                }
            });
        });

        let mut tui_msg_adder = app.add_msg_fn();
        global_rt().spawn(async move {
            while let Some(event) = peer_rx.recv().await {
                debug!("recv: {:?}", event);
                match event {
                    InternalEvent::ChatMessage(m) => tui_msg_adder(m),
                    InternalEvent::ShowInfoMsg(text) | InternalEvent::ShowErrorMsg(text) => {
                        tui_msg_adder(
                            Msg::default()
                                .set_content(text, 0)
                                .set_kind(MsgKind::System),
                        );
                    }
                    other => {
                        debug!("Received non-chat message event: {:?}", other);
                    }
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

#[cfg(test)]
mod command_tests {
    use super::*;

    #[test]
    fn parses_clone_command() {
        let command = parse_local_chat_command("/clone blossom://example.com/abcd/repo dest")
            .expect("parse command")
            .expect("expected command");

        assert_eq!(
            command,
            LocalChatCommand::GitClone {
                url: "blossom://example.com/abcd/repo".to_string(),
                destination: Some("dest".to_string()),
            }
        );
    }

    #[test]
    fn parses_git_clone_alias() {
        let command = parse_local_chat_command("/git clone blossom://example.com/abcd/repo")
            .expect("parse command")
            .expect("expected command");

        assert_eq!(
            command,
            LocalChatCommand::GitClone {
                url: "blossom://example.com/abcd/repo".to_string(),
                destination: None,
            }
        );
    }

    #[test]
    fn builds_oneshot_subcommand() {
        let args = crate::chat_oneshot!("gnostr-dev", "gnostr main started");

        assert_eq!(args.topic.as_deref(), Some("gnostr-dev"));
        assert_eq!(args.oneshot.as_deref(), Some("gnostr main started"));
    }

    #[function_name::named]
    #[test]
    fn builds_named_oneshot_subcommand() {
        let args = crate::chat_oneshot_named!("gnostr-dev", "gnostr main started");
        let expected = format!(
            "{}::{} {}",
            module_path!(),
            function_name!(),
            "gnostr main started"
        );

        assert_eq!(args.topic.as_deref(), Some("gnostr-dev"));
        assert_eq!(args.oneshot.as_deref(), Some(expected.as_str()));
    }

    #[function_name::named]
    #[test]
    fn builds_debug_message() {
        let msg = crate::chat_debug!("trace ready");
        assert_eq!(msg.kind, MsgKind::Debug);
        assert_eq!(
            msg.content[0],
            format!(
                "[DEBUG] {}::{} {}",
                module_path!(),
                function_name!(),
                "trace ready"
            )
        );
    }
}
