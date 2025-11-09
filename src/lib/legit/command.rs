extern crate chrono;

use anyhow::{anyhow, Result as AnyhowResult};
use clap::{Args, Parser};
use git2::{Commit, ObjectType, Oid, Repository};
use crate::queue::InternalEvent;
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use libp2p::gossipsub;
use nostr_sdk_0_37_0::prelude::*;
use nostr_sdk_0_37_0::prelude::Tag;
//use nostr_sdk_0_37_0::EventBuilder;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::{Result as SerdeJsonResult, Value};
//use sha2::Digest;
//use tokio::time::Duration;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write;
use std::{error::Error, time::Duration};
use tokio::{io as TokioIo, io::AsyncBufReadExt};
use tracing_subscriber::util::SubscriberInitExt;
//use tracing::debug;
use tracing::{debug, info, error};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

use gnostr_asyncgit::sync::commit::{SerializableCommit, serialize_commit, deserialize_commit};
use crate::utils::{generate_nostr_keys_from_commit_hash, parse_json, split_json_string};

use std::process::Command;
use chrono::offset::Utc;
use chrono::DateTime;
use std::io::{Result as IoResult};
use std::env;
use std::time::{SystemTime};
use std::thread::sleep;
use std::convert::TryInto;
use std::any::type_name;
use std::{io, thread};
use argparse::{ArgumentParser,Store};
use gnostr_legit::gitminer::Gitminer;
use git2::*;
use sha2::{Sha256, Digest};
use pad::{PadStr, Alignment};
use ::time::OffsetDateTime;
use ::time::macros::datetime;

use gnostr_legit::{gitminer, repo, worker};

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

pub async fn run_legit_command(mut opts: gitminer::Options) -> io::Result<()> {

    let start = SystemTime::now();
    let system_time = SystemTime::now();

    let repo = Repository::open(&opts.repo).expect("Couldn't open repository");

    if repo.state() != RepositoryState::Clean {
        let repo_state =
            if cfg!(target_os = "windows") {
            Command::new("cmd")
                    .args(["/C", "git status"])
                    .output()
                    .expect("failed to execute process")
            } else {
            Command::new("sh")
                    .arg("-c")
                    .arg("gnostr-git diff")
                    .output()
                    .expect("failed to execute process")
            };

        let state = String::from_utf8(repo_state.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    }

    // GEMINI we need to handle --kind flag from ../sub_commands/legit.rs 
    if opts.message.is_empty() {
        let output =
            if cfg!(target_os = "windows") {
            Command::new("cmd")
                    .args(["/C", "git status"])
                    .output()
                    .expect("failed to execute process")
            } else {
            Command::new("sh")
                    .arg("-c")
                    .arg("git diff")
                    .output()
                    .expect("failed to execute process")
            };

        let message = String::from_utf8(output.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
        opts.message = [message.to_string()].to_vec();
    }

    let mut miner = Gitminer::new(opts.clone()).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to start git miner: {}", e)))?;
    debug!("Gitminer options: {:?}", opts);

    let hash = miner.mine().map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to generate commit: {}", e)))?;

    // Initiate gnostr_legit_event after GitMiner has finished.
    // gnostr_legit_event itself spawns tasks on the global runtime.
    // We don't need to block_on it here, as it manages its own runtime tasks.
    match gnostr_legit_event(opts.kind).await {
        Ok(_) => {
            info!("gnostr_legit_event initiated successfully.");
            Ok(())
        }
        Err(e) => {
            eprintln!("Error initiating gnostr_legit_event: {}", e);
            // Convert the error to an io::Error.
            Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }
}




pub async fn create_event_with_custom_tags(
    keys: &Keys,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> Result<(Event, UnsignedEvent)> {
    let mut builder = EventBuilder::new(Kind::TextNote, content);

    for (tag_name, tag_values) in custom_tags {
        info!("tag_name={:?}", tag_name);
        info!("tag_values={:?}", tag_values);
        //pops &tag_values[0]
        let tag: Tag = Tag::parse([&tag_name, &tag_values[0]]).unwrap();
        builder = builder.tag(tag);
    }

    let unsigned_event = builder.build(keys.public_key().clone()); // Build the unsigned event
    let signed_event = unsigned_event.clone().sign(keys).await.map_err(|e| anyhow!("Failed to sign event: {}", e))?;
    Ok((signed_event, unsigned_event))
}

pub async fn create_unsigned_event(
    keys: &Keys,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> anyhow::Result<UnsignedEvent> {
    let mut builder = EventBuilder::new(Kind::TextNote, content);

    for (tag_name, tag_values) in custom_tags {
        let tag: Tag = Tag::parse([&tag_name, &tag_values[0]]).unwrap();
        builder = builder.tag(tag);
    }

    let unsigned_event = builder.build(keys.public_key());
    Ok(unsigned_event)
}

pub async fn create_kind_event(
    keys: &Keys,
    kind: u16,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> anyhow::Result<(Event, UnsignedEvent)> {
    let mut builder = EventBuilder::new(Kind::Custom(kind), content);

    for (tag_name, tag_values) in custom_tags {
        let tag: Tag = Tag::parse([&tag_name, &tag_values[0]]).unwrap();
        builder = builder.tag(tag);
    }

    let unsigned_event = builder.build(keys.public_key());
    let signed_event = unsigned_event.clone().sign(keys);
    Ok((signed_event.await?, unsigned_event))
}


pub async fn create_event(
    keys: Keys,
    custom_tags: HashMap<String, Vec<String>>,
    content: &str,
) -> Result<()> {
    //let content = "Hello, Nostr with custom tags!";

    let (signed_event, _unsigned_event) = create_event_with_custom_tags(&keys, content, custom_tags).await?;
    info!("{}", serde_json::to_string_pretty(&signed_event)?);

    let opts = Options::new().gossip(true);
    let client = Client::builder().signer(keys.clone()).opts(opts).build();
    for relay in BOOTSTRAP_RELAYS.to_vec() {
        debug!("{}", relay);
        client.add_discovery_relay(relay).await.expect("");
    }

    // Connect to the relays.
    client.connect().await;

    // client.send_event - signed_event
    client.send_event(signed_event.clone()).await?;

    info!("{}", serde_json::to_string_pretty(&signed_event)?);
    info!("signed_event sent:\n{:?}", signed_event);


    //

    // Publish a text note
    let pubkey = keys.public_key();

    info!("pubkey={}", keys.public_key());
    let builder = EventBuilder::text_note(format!("gnostr:legit {}", pubkey))
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
        println!("{}", event.as_json());
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



//async tasks
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

pub async fn gnostr_legit_event(kind: Option<u16>) -> Result<(), Box<dyn Error>> {

    // gnostr_legit_event
    let empty_hash_keys =
        Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();

    //create a HashMap of custom_tags
    //used to insert commit tags
    let mut custom_tags = HashMap::new();
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);

    global_rt().spawn(async move {
        //send to create_event function with &"custom content"
        let signed_event = create_event(empty_hash_keys, custom_tags, &"gnostr-legit:event").await;
        println!("signed_event:\n{:?}", signed_event);
    });

    //initialize git repo
    let repo = Repository::discover(".")?;

    //gather some repo info
    //find HEAD
    let head = repo.head()?;
    let obj = head.resolve()?.peel(ObjectType::Commit)?;

    //read top commit
    let commit = obj.peel_to_commit()?;
    let serialized_commit = serialize_commit(&commit)?;
    debug!("Serialized commit:\n{}", serialized_commit.clone());
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
    let serialized_commit_for_kind_event = serialized_commit.clone();

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
        if let Err(e) = create_event(padded_keys.clone(), custom_tags, &"gnostr-legit:event").await {
            error!("Failed to create event: {}", e);
        }

        if let Err(e) = create_kind_event(&padded_keys, kind.unwrap_or(1), &serialized_commit_for_kind_event, HashMap::new()).await {
            error!("Failed to create kind event: {:?}", e);
        }
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

        for relay in BOOTSTRAP_RELAYS.to_vec() {
            debug!("{}", relay);
            client.add_relay(relay).await.expect("");
        }
        client.connect().await;

        //build git gnostr event
        let builder = EventBuilder::text_note(serialized_commit.clone());

        //send git gnostr event
        let output = client.send_event_builder(builder).await.expect("");

        info!("Event ID: {}", output.id());
        info!("Event ID BECH32: {}", output.id().to_bech32().expect(""));
        info!("Sent to: {:?}", output.success);
        info!("Not sent to: {:?}", output.failed);
    });

    Ok(())
}
