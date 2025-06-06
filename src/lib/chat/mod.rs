use clap::{Args, Parser};
use libp2p::gossipsub;
use once_cell::sync::OnceCell;
use std::{error::Error, time::Duration};
use tokio::{io, io::AsyncBufReadExt};
use tracing_subscriber::util::SubscriberInitExt;
//use tracing::debug;
use tracing::{debug, info};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;

use anyhow::{anyhow, Result};
use git2::{Commit, ObjectType, Oid, Repository};
use nostr_sdk_0_37_0::prelude::*;
//use nostr_sdk_0_37_0::EventBuilder;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::{Result as SerdeJsonResult, Value};
use sha2::Digest;
//use tokio::time::Duration;

pub mod p2p;
pub mod ui;
pub use p2p::evt_loop;
pub mod msg;
pub use msg::*;

//const TITLE: &str = include_str!("./title.txt");

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableCommit {
    id: String,
    tree: String,
    parents: Vec<String>,
    author_name: String,
    author_email: String,
    committer_name: String,
    committer_email: String,
    message: String,
    time: i64,
}

pub fn byte_array_to_hex_string(byte_array: &[u8; 32]) -> String {
    let mut hex_string = String::new();
    for byte in byte_array {
        write!(&mut hex_string, "{:02x}", byte).unwrap(); // Use unwrap for simplicity, handle errors in production.
    }
    hex_string
}

pub async fn create_event_with_custom_tags(
    keys: &Keys,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> Result<Event> {
    let mut builder = EventBuilder::new(Kind::TextNote, content);

    for (tag_name, tag_values) in custom_tags {
        info!("tag_name={:?}", tag_name);
        info!("tag_values={:?}", tag_values);
        //pops &tag_values[0]
        let tag: Tag = Tag::parse([&tag_name, &tag_values[0]]).unwrap();
        builder = builder.tag(tag);
    }

    let unsigned_event = builder.build(keys.public_key()); // Build the unsigned event
    let signed_event = unsigned_event.sign(keys); // Sign the event
    Ok(signed_event.await?)
}

pub async fn create_event(
    keys: Keys,
    custom_tags: HashMap<String, Vec<String>>,
    content: &str,
) -> Result<()> {
    //let content = "Hello, Nostr with custom tags!";

    let signed_event = create_event_with_custom_tags(&keys, content, custom_tags).await?;
    info!("{}", serde_json::to_string_pretty(&signed_event)?);

    let opts = Options::new().gossip(true);
    let client = Client::builder().signer(keys.clone()).opts(opts).build();

    client.add_discovery_relay("wss://relay.damus.io").await?;
    client.add_discovery_relay("wss://purplepag.es").await?;
    client
        .add_discovery_relay("ws://oxtrdevav64z64yb7x6rjg4ntzqjhedm5b5zjqulugknhzr46ny2qbad.onion")
        .await?;

    // add some relays
    // TODO get_relay_list here
    client.add_relay("wss://relay.damus.io").await?;
    client.add_relay("wss://e.nos.lol").await?;
    client.add_relay("wss://nos.lol").await?;

    // Connect to the relays.
    client.connect().await;

    // client.send_event - signed_event
    client.send_event(signed_event.clone()).await?;

    info!("{}", serde_json::to_string_pretty(&signed_event)?);
    info!("signed_event sent:\n{:?}", signed_event);

    // Publish a text note
    let pubkey = keys.public_key();

    info!("pubkey={}", keys.public_key());
    let builder = EventBuilder::text_note(format!("Hello Worlds {}", pubkey))
        .tag(Tag::public_key(pubkey))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2 3 4 11 22 33 44".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2 3 4 11 22 33".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2 3 4 11 22".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2 3 4 11".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2 3 4".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2 3".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1 2".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "1".chars(),
        ))
        .tag(Tag::custom(
            TagKind::Custom(Cow::from("gnostr")),
            "".chars(),
        ));

    //send from send_event_builder
    let output = client.send_event_builder(builder).await?;
    info!("Event ID: {}", output.to_bech32()?);

    info!("Sent to:");
    for url in output.success.into_iter() {
        info!("- {url}");
    }

    info!("Not sent to:");
    for (url, reason) in output.failed.into_iter() {
        info!("- {url}: {reason:?}");
    }

    // Get events
    let filter_one = Filter::new().author(pubkey).kind(Kind::TextNote).limit(10);
    let events = client
        .fetch_events(vec![filter_one], Some(Duration::from_secs(10)))
        .await?;

    for event in events.into_iter() {
        info!("{}", event.as_json());
    }

    // another filter
    let test_author_pubkey =
        PublicKey::parse("npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet")?;

    info!("test_author_pubkey={}", test_author_pubkey);

    let filter_test_author = Filter::new()
        .author(test_author_pubkey)
        .kind(Kind::TextNote)
        .limit(10);
    let events = client
        .fetch_events(vec![filter_test_author], Some(Duration::from_secs(10)))
        .await?;

    for event in events.into_iter() {
        info!("test_author:\n\n{}", event.as_json());
    }

    Ok(())
}

pub fn serialize_commit(commit: &Commit) -> Result<String> {
    let id = commit.id().to_string();
    let tree = commit.tree_id().to_string();
    let parents = commit.parent_ids().map(|oid| oid.to_string()).collect();
    let author = commit.author();
    let committer = commit.committer();
    let message = commit
        .message()
        .ok_or(anyhow!("No commit message"))?
        .to_string();
    debug!("message:\n{:?}", message);
    let time = commit.time().seconds();
    debug!("time: {:?}", time);

    let serializable_commit = SerializableCommit {
        id,
        tree,
        parents,
        author_name: author.name().unwrap_or_default().to_string(),
        author_email: author.email().unwrap_or_default().to_string(),
        committer_name: committer.name().unwrap_or_default().to_string(),
        committer_email: committer.email().unwrap_or_default().to_string(),
        message,
        time,
    };

    let serialized = serde_json::to_string(&serializable_commit)?;
    debug!("serialized_commit: {:?}", serialized);
    Ok(serialized)
}

pub fn deserialize_commit<'a>(repo: &'a Repository, data: &'a str) -> Result<Commit<'a>> {
    //we serialize the commit data
    //easier to grab the commit.id
    let serializable_commit: SerializableCommit = serde_json::from_str(data)?;
    //grab the commit.id
    let oid = Oid::from_str(&serializable_commit.id)?;
    //oid used to search the repo
    let commit_obj = repo.find_object(oid, Some(ObjectType::Commit))?;
    //grab the commit
    let commit = commit_obj.peel_to_commit()?;
    //confirm we grabbed the correct commit
    if commit.id().to_string() != serializable_commit.id {
        return Err(anyhow!("Commit ID mismatch during deserialization"));
    }
    //return the commit
    Ok(commit)
}

pub fn generate_nostr_keys_from_commit_hash(commit_id: &str) -> Result<Keys> {
    let padded_commit_id = format!("{:0>64}", commit_id);
    info!("padded_commit_id:{:?}", padded_commit_id);
    let keys = Keys::parse(&padded_commit_id);
    Ok(keys.unwrap())
}

pub fn parse_json(json_string: &str) -> SerdeJsonResult<Value> {
    serde_json::from_str(json_string)
}

pub fn split_value_by_newline(json_value: &Value) -> Option<Vec<String>> {
    if let Value::String(s) = json_value {
        let lines: Vec<String> = s.lines().map(|line| line.to_string()).collect();
        Some(lines)
    } else {
        None // Return None if the Value is not a string
    }
}

pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

pub fn split_json_string(value: &Value, separator: &str) -> Vec<String> {
    if let Value::String(s) = value {
        s.split(&separator).map(|s| s.to_string()).collect()
    } else {
        vec![String::from("")]
    }
}

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
        //send to create_event function with &"custom content"
        let signed_event = create_event(empty_hash_keys, custom_tags, &"gnostr-chat:event").await;
        debug!("signed_event:\n{:?}", signed_event);
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
        //send to create_event function with &"custom content"
        let signed_event =
            create_event(padded_keys.clone(), custom_tags, &"gnostr-chat:event").await;
        debug!("signed_event:\n{:?}", signed_event);
    });

    //TODO config metadata

    //access some git info
    let serialized_commit = serialize_commit(&commit)?;
    debug!("Serialized commit:\n{}", serialized_commit.clone());

    let binding = serialized_commit.clone();
    let deserialized_commit = deserialize_commit(&repo, &binding)?;
    debug!("Deserialized commit:\n{:?}", deserialized_commit);

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
    let value: Value = parse_json(&serialized_commit.clone())?;
    //info!("value:\n{}", value);

    // Accessing object elements.
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
        for part in parts {
            debug!("\n{}", part);
        }
        debug!("message:\n{}", message.as_str().unwrap_or(""));
    }
    if let Value::Number(time) = &value["time"] {
        debug!("time:\n{}", time);
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
        client.add_relay("wss://relay.damus.io").await.expect("");
        client.add_relay("wss://e.nos.lol").await.expect("");
        client.connect().await;

        //build git gnostr event
        let builder = EventBuilder::text_note(serialized_commit.clone());

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
    //for line in TITLE.lines() {
    //    app.add_message(
    //        Msg::default()
    //            .set_content(line.to_string())
    //            .set_kind(MsgKind::Raw),
    //    );
    //}

    //TODO construct git commit message header

    let serialized_commit = serialize_commit(&commit)?;
    let value: Value = parse_json(&serialized_commit.clone())?;
    //info!("value:\n{}", value);

    // Accessing object elements.
    if let Some(id) = value.get("id") {
        debug!("id:\n{}", id.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(id.as_str().unwrap_or("")))
                .set_kind(MsgKind::GitCommitId),
        );
    }
    if let Some(tree) = value.get("tree") {
        debug!("tree:\n{}", tree.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(tree.as_str().unwrap_or("")))
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
                        .set_content(String::from(parent.as_str().unwrap_or("")))
                        .set_kind(MsgKind::GitCommitParent),
                );
            }
            if let Some(parent) = arr.get(1) {
                debug!("parent:\n{}", parent.as_str().unwrap_or(""));
                app.add_message(
                    Msg::default()
                        .set_content(String::from(parent.as_str().unwrap_or("")))
                        .set_kind(MsgKind::GitCommitParent),
                );
            }
        }
    }
    if let Some(author_name) = value.get("author_name") {
        debug!("author_name:\n{}", author_name.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(author_name.as_str().unwrap_or("")))
                .set_kind(MsgKind::GitCommitAuthor),
        );
    }
    if let Some(author_email) = value.get("author_email") {
        debug!("author_email:\n{}", author_email.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(author_email.as_str().unwrap_or("")))
                .set_kind(MsgKind::GitCommitEmail),
        );
    }
    if let Some(committer_name) = value.get("committer_name") {
        debug!("committer_name:\n{}", committer_name.as_str().unwrap_or(""));
        app.add_message(
            Msg::default()
                .set_content(String::from(committer_name.as_str().unwrap_or("")))
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
                .set_content(String::from(committer_email.as_str().unwrap_or("")))
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
                .set_content(time.to_string())
                .set_kind(MsgKind::GitCommitTime),
        );
    }

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

    let (peer_tx, mut peer_rx) = tokio::sync::mpsc::channel::<Msg>(100);
    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<Msg>(100);

    // let input_loop_fut = input_loop(input_tx);
    let input_tx_clone = input_tx.clone();

    let value = input_tx_clone.clone();
    app.on_submit(move |m| {
        let value = value.clone();
        global_rt().spawn(async move {
            debug!("sent: {:?}", m);
            value.send(m).await.unwrap();
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
        while let Some(m) = peer_rx.recv().await {
            debug!("recv: {:?}", m);
            tui_msg_adder(m);
        }
    });

    // say hi
    let input_tx_clone = input_tx.clone();
    global_rt().spawn(async move {
        tokio::time::sleep(Duration::from_millis(1000)).await;
        input_tx_clone
            .send(Msg::default().set_kind(MsgKind::Join))
            .await
            .unwrap();
    });

    app.run()?;

    // say goodbye
    // input_tx.blocking_send(Msg::default().set_kind(MsgKind::Leave))?;
    let _ = input_tx.send(Msg::default().set_kind(MsgKind::Leave));
    std::thread::sleep(Duration::from_millis(500));

    Ok(())
}

pub async fn input_loop(
    self_input: tokio::sync::mpsc::Sender<Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    while let Some(line) = stdin.next_line().await? {
        let msg = Msg::default().set_content(line);
        if let Ok(b) = serde_json::to_vec(&msg) {
            self_input.send(b).await?;
        }
    }
    Ok(())
}
