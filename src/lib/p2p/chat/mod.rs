use anyhow::{anyhow, Result};
use clap::{Args, Parser};
use git2::{Commit, ObjectType, Oid, Repository};
use crate::legit::command::create_event;
use crate::queue::InternalEvent;
use gnostr_asyncgit::sync::commit::SerializableCommit;
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use libp2p::gossipsub;
use nostr_sdk_0_37_0::prelude::*;
//use nostr_sdk_0_37_0::EventBuilder;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::{Result as SerdeJsonResult, Value};
//use sha2::Digest;
//use tokio::time::Duration;

use std::collections::HashMap;
use std::fmt::Write;
use std::{error::Error, time::Duration};
use tokio::{io, io::AsyncBufReadExt};
use tracing_subscriber::util::SubscriberInitExt;
//use tracing::debug;
use tracing::{debug, info};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use gnostr_asyncgit::sync::commit::{serialize_commit, deserialize_commit};
use crate::utils::{generate_nostr_keys_from_commit_hash, parse_json, split_json_string};

pub mod msg;
pub use msg::*;
pub mod p2p;
pub use p2p::evt_loop;
pub mod ui;
pub mod tests;

const TITLE: &str = include_str!("./title.txt");


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
}

//async tasks
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

pub fn chat(sub_command_args: &ChatSubCommands) -> Result<(), Box<dyn Error>> {
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

    //parse keys from sha256 hash
    let empty_hash_keys =
        Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();

    //create a HashMap of custom_tags
    //used to insert commit tags
    let mut custom_tags = HashMap::new();
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);

    global_rt().spawn(async move {
        let client = Client::new(empty_hash_keys);
        for relay in BOOTSTRAP_RELAYS.to_vec() {
            debug!("{}", relay);
            client.add_relay(relay).await.expect("");
        }
        client.connect().await;

        let builder = EventBuilder::text_note("gnostr-chat:event");
        let output = client.send_event_builder(builder).await.expect("");
        debug!("Event ID: {}", output.id());
    });

    //initialize git repo
    let repo = Repository::discover(".")?;

    //gather some repo info
    //find HEAD
    let head = repo.head()?;
    let obj = head.resolve()?.peel(ObjectType::Commit)?;

    //read top commit
    let commit = obj.peel_to_commit()?;
    let commit_id = commit.id().to_string();
    //some info wrangling
    debug!("commit_id:\n{}", commit_id);
    let padded_commit_id = format!("{:0>64}", commit_id);

    //// commit based keys
    //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    //info!("keys.public_key():\n{}", keys.public_key());

    //parse keys from sha256 hash
    let padded_keys = Keys::parse(padded_commit_id).unwrap();

    //create a HashMap of custom_tags
    //used to insert commit tags
    let mut custom_tags = HashMap::new();
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    custom_tags.insert(
        padded_keys.clone().public_key().to_string(),
        vec!["GNOSTR".to_string()],
    );

    global_rt().spawn(async move {
        let client = Client::new(padded_keys);
        for relay in BOOTSTRAP_RELAYS.to_vec() {
            debug!("{}", relay);
            client.add_relay(relay).await.expect("");
        }
        client.connect().await;

        let builder = EventBuilder::text_note("gnostr-chat:event");
        let output = client.send_event_builder(builder).await.expect("");
        debug!("Event ID: {}", output.id());
    });

    //TODO config metadata

    //access some git info
    let serialized_commit = serialize_commit(&commit).expect("Failed to serialize commit");

    let binding = serialized_commit.clone();
    let deserialized_commit = deserialize_commit(&repo, &binding).expect("Failed to deserialize commit");

    //access commit summary in the deserialized commit
    debug!("Original commit ID:\n{}", commit_id);
    debug!("Deserialized commit ID:\n{}", deserialized_commit.id());

    //additional checking
    if commit.id() != deserialized_commit.id() {
        debug!("Commit IDs do not match!");
    } else {
        debug!("Commit IDs match!");
    }

    let serialized_commit = serialize_commit(&commit)?;
    let value: Value = parse_json(&serialized_commit.clone()).expect("Failed to parse JSON");
    if let Some(id) = value.get("id") {
        debug!("id:\n{}", id.as_str().unwrap_or(""));
    }
    if let Some(tree) = value.get("tree") {
        debug!("tree:\n{}", tree.as_str().unwrap_or(""));
    }
    // Accessing parent commits (merge may be array)
    if let Some(parent) = value.get("parents") {
        if let Value::Array(arr) = parent {
            if let Some(parent) = arr.get(0) {
                debug!("parent:\n{}", parent.as_str().unwrap_or("initial commit"));
            }
            if let Some(parent) = arr.get(1) {
                debug!("parent:\n{}", parent.as_str().unwrap_or(""));
            }
        }
    }
    if let Some(author_name) = value.get("author_name") {
        debug!("author_name:\n{}", author_name.as_str().unwrap_or(""));
    }
    if let Some(author_email) = value.get("author_email") {
        debug!("author_email:\n{}", author_email.as_str().unwrap_or(""));
    }
    if let Some(committer_name) = value.get("committer_name") {
        debug!("committer_name:\n{}", committer_name.as_str().unwrap_or(""));
    }
    if let Some(committer_email) = value.get("committer_email") {
        debug!(
            "committer_email:\n{}",
            committer_email.as_str().unwrap_or("")
        );
    }

    //split the commit message into a Vec<String>
    if let Some(message) = value.get("message") {
        let parts = split_json_string(&message, "\n");
        if let Value::Number(time) = &value["time"] {
            debug!("time:\n{}", time);
        }
    }

    // // Accessing array elements.
    // if let Some(items) = value.get("items") {
    //     if let Value::Array(arr) = items {
    //         if let Some(first_item) = arr.get(0) {
    //             info!("First item: {}", first_item);
    //         }
    //         if let Some(second_item) = arr.get(1){
    //             info!("second item: {}", second_item.as_str().unwrap_or(""));
    //         }
    //     }
    // }

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
    debug!("commit_id:\n{}", commit_id);
    let padded_commit_id = format!("{:0>64}", commit_id.clone());
    global_rt().spawn(async move {
        //// commit based keys
        //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
        //info!("keys.secret_key():\n{:?}", keys.secret_key());
        //info!("keys.public_key():\n{}", keys.public_key());

        //parse keys from sha256 hash
        let padded_keys = Keys::parse(padded_commit_id).unwrap();
        //create nostr client with commit based keys
        //let client = Client::new(keys);
        let client = Client::new(padded_keys.clone());

        for relay in BOOTSTRAP_RELAYS.to_vec() {
            debug!("{}", relay);
            client.add_relay(relay).await.expect("");
        }
        client.connect().await;

        //build git gnostr event
        let _builder = EventBuilder::text_note(serialized_commit.clone());

        //send git gnostr event
        //let output = client.send_event_builder(builder).await.expect("");

        //some reporting
        //info!("Event ID: {}", output.id());
        //info!("Event ID BECH32: {}", output.id().to_bech32().expect(""));
        //info!("Sent to: {:?}", output.success);
        //info!("Not sent to: {:?}", output.failed);
    });

    let mut app = ui::App::default();

    //TODO
    for line in TITLE.lines() {
        app.add_message(
            Msg::default()
                .set_content(line.to_string(), 80 as usize)
                .set_kind(MsgKind::Raw),
        );
    }

    //TODO construct git commit message header

    let serialized_commit = serialize_commit(&commit)?;
    let value: Value = parse_json(&serialized_commit.clone())?;
    //info!("value:\n{}", value);
    let pretty_json = serde_json::to_string_pretty(&value)?;
    for line in pretty_json.lines() {
        app.add_message(
            Msg::default()
                .set_content(line.to_string(), 80 as usize)
                .set_kind(MsgKind::Raw),
        );
    }

    // Accessing object elements.
    if let Some(id) = value.get("id") {
        debug!("id:\n{}", id.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(id.as_str().unwrap_or("")), 0)
                .set_kind(MsgKind::GitCommitId),
        );
    }
    if let Some(tree) = value.get("tree") {
        debug!("tree:\n{}", tree.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(tree.as_str().unwrap_or("")), 0)
                .set_kind(MsgKind::GitCommitTree),
        );
    }
    // Accessing parent commits (merge may be array)
    if let Some(parent) = value.get("parents") {
        if let Value::Array(arr) = parent {
            if let Some(parent) = arr.get(0) {
                debug!("parent:\n{}", parent.as_str().unwrap_or("initial commit"));
                app.add_message(
                    Msg::default()
                        .set_content(String::from(parent.as_str().unwrap_or("")), 0)
                        .set_kind(MsgKind::GitCommitParent),
                );
            }
            if let Some(parent) = arr.get(1) {
                debug!("parent:\n{}", parent.as_str().unwrap_or(""));
                app.add_message(
                    Msg::default()
                        .set_content(String::from(parent.as_str().unwrap_or("")), 0)
                        .set_kind(MsgKind::GitCommitParent),
                );
            }
        }
    }
    if let Some(author_name) = value.get("author_name") {
        debug!("author_name:\n{}", author_name.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(author_name.as_str().unwrap_or("")), 0)
                .set_kind(MsgKind::GitCommitAuthor),
        );
    }
    if let Some(author_email) = value.get("author_email") {
        debug!("author_email:\n{}", author_email.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(author_email.as_str().unwrap_or("")), 0)
                .set_kind(MsgKind::GitCommitEmail),
        );
    }
    if let Some(committer_name) = value.get("committer_name") {
        debug!("committer_name:\n{}", committer_name.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(committer_name.as_str().unwrap_or("")), 0)
                .set_kind(MsgKind::GitCommitName),
        );
    }
    if let Some(committer_email) = value.get("committer_email") {
        debug!(
            "committer_email:\n{}",
            committer_email.as_str().unwrap_or("")
        );
        app.add_message(
            Msg::default()
                .set_content(String::from(committer_email.as_str().unwrap_or("")), 0)
                .set_kind(MsgKind::GitCommitEmail),
        );
    }

    //split the commit message into a Vec<String>
    //if let Some(message) = value.get("message") {
    //    let parts = split_json_string(&message, "\n");
    //    for part in parts {
    //        debug!("\n{}", part);

    //        app.add_message(
    //            Msg::default()
    //                .set_content(String::from(part))
    //                .set_kind(MsgKind::GitCommitMessagePart),
    //        );
    //    }
    //    debug!("message:\n{}", message.as_str().unwrap_or(""));
    //}
    if let Value::Number(time) = &value["time"] {
        debug!("time:\n{}", time);

        app.add_message(
            Msg::default()
                .set_content(time.to_string(), 0)
                .set_kind(MsgKind::GitCommitTime),
        );
    }
    app.add_message(
        Msg::default()
            .set_content("additional RAW message value".to_string(), 0)
            .set_kind(MsgKind::Raw),
    );

    let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    info!("keys.public_key():\n{}", keys.public_key());
    //app.add_message(
    //    Msg::default()
    //        .set_content(keys.public_key().to_string())
    //        .set_kind(MsgKind::GitCommitHeader),
    //);
    ////app.add_message(
    ////    Msg::default()
    ////        .set_content(String::from(serialize_commit))
    ////        .set_kind(MsgKind::GitCommitHeader),
    ////);
    //app.add_message(
    //    Msg::default()
    //        .set_content(String::from("third message"))
    //        .set_kind(MsgKind::GitCommitHeader),
    //);
    //app.add_message(
    //    Msg::default()
    //        .set_content(String::from("fourth message"))
    //        .set_kind(MsgKind::GitCommitHeader),
    //);

    let (peer_tx, mut peer_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<InternalEvent>(100);

    // let input_loop_fut = input_loop(input_tx);
    let input_tx_clone = input_tx.clone();

    let value = input_tx_clone.clone();
    app.on_submit(move |m| {
        let value = value.clone();
        global_rt().spawn(async move {
            debug!("sent: {:?}", m);
            value.send(InternalEvent::ChatMessage(m)).await.unwrap();
        });
    });

    //
    let mut topic = String::from(commit_id.to_string());
    if !args.topic.is_none() {
        topic = args.topic.expect("");
    } else {
    }

    app.topic = topic.clone();

    let topic = gossipsub::IdentTopic::new(format!("{}", app.topic.clone()));

    global_rt().spawn(async move {
        evt_loop(input_rx, peer_tx, topic).await.unwrap();
    });

    // recv from peer
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

    // say hi
    let input_tx_clone = input_tx.clone();
    global_rt().spawn(async move {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        input_tx_clone
            .send(InternalEvent::ChatMessage(Msg::default().set_kind(MsgKind::Join)))
            .await
            .unwrap();
    });

    app.run()?;

    // say goodbye
    // input_tx.blocking_send(Msg::default().set_kind(MsgKind::Leave))?;
    let _ = input_tx.send(InternalEvent::ChatMessage(Msg::default().set_kind(MsgKind::Leave)));
    std::thread::sleep(Duration::from_millis(500));

    Ok(())
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
