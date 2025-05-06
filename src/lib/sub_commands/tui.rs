#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

//use crate::sub_commands::chat::Utc;

use crate::tui::create_event;
use crate::tui::msg::*;
use crate::tui::p2p::evt_loop;
use crate::tui::parse_json;
use crate::tui::split_json_string;
use crate::tui::ui;
use crate::tui::TuiCli;
use crate::tui::*;
use crate::tui::tui;
use crate::tui::tui_chat;
//use crate::tui::tui;
//use crate::tui::tui::tui;
use crate::global_rt::global_rt;
//use crate::sub_commands::tui;
use anyhow::Result;
use clap::{Parser /*, Subcommand*/};
use git2::{ObjectType, Repository};
use gnostr_asyncgit::sync::commit::{deserialize_commit, serialize_commit};
use serde::ser::StdError;

use chrono::Utc;

use libp2p::gossipsub;
use log;
use nostr_sdk_0_37_0::prelude::*;
use nostr_sdk_0_37_0::Client;
use nostr_sdk_0_37_0::EventBuilder;
use nostr_sdk_0_37_0::Keys;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::{error::Error, time::Duration};
use tracing::{debug, info, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub async fn tui(sub_command_args: &TuiSubCommands) -> Result<(), Box<dyn StdError>> {
    run(sub_command_args).await?;
    Ok(())
}

pub async fn run(sub_command_args: &TuiSubCommands) -> Result<(), Box<dyn StdError>> {
    let sub_command_args = sub_command_args;
    if let Some(name) = sub_command_args.name.clone() {
        use std::env;
        env::set_var("USER", &name);
    };

    let level = if sub_command_args.debug {
        Level::DEBUG
    } else if sub_command_args.trace {
        Level::TRACE
    } else if sub_command_args.info {
        Level::INFO
    } else {
        Level::WARN
    };
    //TODO chat specific filters
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
        //
        .add_directive("libp2p_gossipsub::behaviour=off".parse().unwrap());

    //    let subscriber = Registry::default()
    //        .with(fmt::layer().with_writer(std::io::stdout))
    //        .with(filter);

    let subscriber = Registry::default()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                //.with_timer(fmt::time::Utc::rfc_3339()) // Corrected line
                .with_thread_ids(true),
        )
        .with(filter);

    let _ = subscriber.try_init();
    tracing::trace!("\n{:?}\n", &sub_command_args);
    tracing::debug!("\n{:?}\n", &sub_command_args);
    tracing::info!("\n{:?}\n", &sub_command_args);
    //print!("{:?}", &sub_command_args);

    if sub_command_args.debug || sub_command_args.trace {
        if sub_command_args.nsec.clone().is_some() {
            let keys = Keys::parse(&sub_command_args.nsec.clone().unwrap().clone()).unwrap();
            debug!(
                "{{\"private_key\":\"{}\"}}",
                keys.secret_key().display_secret()
            );
            debug!("{{\"public_key\":\"{}\"}}", keys.public_key());
        }
    }

    //let chat = crate::chat::chat(sub_command_args);
    let chat = tui(sub_command_args);
    //let tui = run(sub_command_args).await;
    //let tui = tui_chat(sub_command_args).await;
    //let tui = run(sub_command_args).await;


    //    //parse keys from sha256 hash
    //    let empty_hash_keys =
    //        Keys::parse("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap();
    //
    //    //create a HashMap of custom_tags
    //    //used to insert commit tags
    //    let mut custom_tags = HashMap::new();
    //    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    //    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    //
    //    global_rt().spawn(async move {
    //        //send to create_event function with &"custom content"
    //        let signed_event = create_event(empty_hash_keys, custom_tags, &"gnostr-chat:event").await;
    //        info!("signed_event:\n{:?}", signed_event);
    //    });
    //
    //    if args.nsec.is_some() {
    //        //let keys = Keys::parse(&args.nsec.unwrap().clone()).unwrap();
    //        let keys = Keys::parse(&args.nsec.clone().unwrap().clone()).unwrap();
    //
    //        info!(
    //            "{{\"private_key\":\"{}\"}}",
    //            keys.secret_key().display_secret()
    //        );
    //        info!("{{\"public_key\":\"{}\"}}", keys.public_key());
    //
    //        let mut custom_tags = HashMap::new();
    //        custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    //        custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    //
    //        global_rt().spawn(async move {
    //            //send to create_event function with &"custom content"
    //            let signed_event = create_event(keys, custom_tags, &"gnostr-chat:event").await;
    //            info!("signed_event:\n{:?}", signed_event);
    //        });
    //    }
    //
    //    //initialize git repo
    //    let repo = Repository::discover(".")?;
    //
    //    //gather some repo info
    //    //find HEAD
    //    let head = repo.head()?;
    //    let obj = head.resolve()?.peel(ObjectType::Commit)?;
    //
    //    //read top commit
    //    let commit = obj.peel_to_commit()?;
    //    let commit_id = commit.id().to_string();
    //    //some info wrangling
    //    info!("416:commit_id:\n{}", commit_id);
    //    let padded_commit_id = format!("{:0>64}", commit_id);
    //    info!("418:padded_commit_id:\n{}", padded_commit_id);
    //
    //    //// commit based keys
    //    //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    //    //info!("keys.public_key():\n{}", keys.public_key());
    //
    //    //parse keys from sha256 hash
    //    let padded_keys = Keys::parse(padded_commit_id).unwrap();
    //
    //    //create a HashMap of custom_tags
    //    //used to insert commit tags
    //    let mut custom_tags = HashMap::new();
    //    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    //    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    //    custom_tags.insert(
    //        padded_keys.clone().public_key().to_string(),
    //        vec!["GNOSTR".to_string()],
    //    );
    //
    //    let serialized_commit = serialize_commit(&commit)?;
    //    info!("476:Serialized commit:\n{}", serialized_commit);
    //
    //    global_rt().spawn(async move {
    //        //send to create_event function with &"custom content"
    //        let signed_event = create_event(padded_keys.clone(), custom_tags, &serialized_commit).await;
    //        info!("467:signed_event:\n{:?}", signed_event);
    //    });
    //
    //    //TODO config metadata
    //
    //    //access some git info
    //    let serialized_commit = serialize_commit(&commit)?;
    //    info!("476:Serialized commit:\n{}", serialized_commit);
    //
    //    let binding = serialized_commit.clone();
    //    let deserialized_commit = deserialize_commit(&repo, &binding)?;
    //    info!("480:Deserialized commit:\n{:?}", deserialized_commit);
    //
    //    //access commit summary in the deserialized commit
    //    info!("481:Original commit ID:\n{}", commit_id);
    //    info!("482:Deserialized commit ID:\n{}", deserialized_commit.id());
    //
    //    //additional checking
    //    if commit.id() != deserialized_commit.id() {
    //        debug!("Commit IDs do not match!");
    //    } else {
    //        debug!("Commit IDs match!");
    //    }
    //
    //    let value: Value = parse_json(&serialized_commit)?;
    //    //info!("value:\n{}", value);
    //
    //    // Accessing object elements.
    //    if let Some(id) = value.get("id") {
    //        info!("id:\n{}", id.as_str().unwrap_or(""));
    //    }
    //    if let Some(tree) = value.get("tree") {
    //        info!("tree:\n{}", tree.as_str().unwrap_or(""));
    //    }
    //    // Accessing parent commits (merge may be array)
    //    if let Some(parent) = value.get("parents") {
    //        if let Value::Array(arr) = parent {
    //            if let Some(parent) = arr.get(0) {
    //                info!("parent:\n{}", parent.as_str().unwrap_or("initial commit"));
    //            }
    //            if let Some(parent) = arr.get(1) {
    //                info!("parent:\n{}", parent.as_str().unwrap_or(""));
    //            }
    //        }
    //    }
    //    if let Some(author_name) = value.get("author_name") {
    //        info!("author_name:\n{}", author_name.as_str().unwrap_or(""));
    //    }
    //    if let Some(author_email) = value.get("author_email") {
    //        info!("author_email:\n{}", author_email.as_str().unwrap_or(""));
    //    }
    //    if let Some(committer_name) = value.get("committer_name") {
    //        info!("committer_name:\n{}", committer_name.as_str().unwrap_or(""));
    //    }
    //    if let Some(committer_email) = value.get("committer_email") {
    //        info!(
    //            "committer_email:\n{}",
    //            committer_email.as_str().unwrap_or("")
    //        );
    //    }
    //
    //    //split the commit message into a Vec<String>
    //    if let Some(message) = value.get("message") {
    //        let parts = split_json_string(&message, "\n");
    //        for part in parts {
    //            info!("\n{}", part);
    //        }
    //        debug!("message:\n{}", message.as_str().unwrap_or(""));
    //    }
    //    if let Value::Number(time) = &value["time"] {
    //        info!("time:\n{}", time);
    //    }
    //
    //    //initialize git repo
    //    let repo = Repository::discover(".").expect("");
    //
    //    //gather some repo info
    //    //find HEAD
    //    let head = repo.head().expect("");
    //    let obj = head
    //        .resolve()
    //        .expect("")
    //        .peel(ObjectType::Commit)
    //        .expect("");
    //
    //    //read top commit
    //    let commit = obj.peel_to_commit().expect("");
    //    let commit_id = commit.id().to_string();
    //    //some info wrangling
    //    info!("commit_id:\n{}", commit_id);
    //    let padded_commit_id = format!("{:0>64}", commit_id.clone());
    //    //// commit based keys
    //    //use gnostr::chat::generate_nostr_keys_from_commit_hash;
    //    //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    //    //info!("keys.public_key():\n{}", keys.public_key());
    //
    //    //parse keys from sha256 hash
    //    let padded_keys = Keys::parse(padded_commit_id).unwrap();
    //    //create nostr client with commit based keys
    //    //let client = Client::new(keys);
    //    let client = Client::new(padded_keys.clone());
    //    global_rt().spawn(async move {
    //        client
    //            .add_relay("wss://relay.damus.io")
    //            .await
    //            .expect("failed to add damus relay");
    //        client
    //            .add_relay("wss://nos.lol")
    //            .await
    //            .expect("failed to add nos.lol relay");
    //        client.connect().await; // connect() likely doesn't return a Result you can match on
    //        let builder = EventBuilder::text_note(serialized_commit.clone());
    //        let output = client
    //            .send_event_builder(builder)
    //            .await
    //            .expect("589:failed to send event");
    //        info!("Event ID: {}", output.id());
    //        info!(
    //            "Event ID BECH32: {}",
    //            output
    //                .id()
    //                //.public_key()
    //                .to_bech32()
    //                .expect("failed to convert to bech32")
    //        );
    //        info!("Sent to: {:?}", output.success);
    //        info!("Not sent to: {:?}", output.failed);
    //    });
    //
    //    //std::process::exit(0);
    //
    //    //P2P CHAT
    //    let mut app = ui::App::default();
    //
    //    //TODO
    //    //for line in TITLE.lines() {
    //    //    app.add_message(
    //    //        Msg::default()
    //    //            .set_content(line.to_string())
    //    //            .set_kind(MsgKind::Raw),
    //    //    );
    //    //}
    //
    //    use crate::chat::generate_nostr_keys_from_commit_hash;
    //    let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    //    info!("keys.public_key():\n{}", keys.public_key());
    //    app.add_message(
    //        Msg::default()
    //            .set_content(keys.public_key().to_string())
    //            .set_kind(MsgKind::Raw),
    //    );
    //    app.add_message(
    //        Msg::default()
    //            .set_content(String::from("second message"))
    //            .set_kind(MsgKind::Raw),
    //    );
    //    app.add_message(
    //        Msg::default()
    //            .set_content(String::from("third message"))
    //            .set_kind(MsgKind::Raw),
    //    );
    //    app.add_message(
    //        Msg::default()
    //            .set_content(String::from("fourth message"))
    //            .set_kind(MsgKind::Raw),
    //    );
    //
    //    let (peer_tx, mut peer_rx) = tokio::sync::mpsc::channel::<Msg>(100);
    //    let (input_tx, input_rx) = tokio::sync::mpsc::channel::<Msg>(100);
    //
    //    // let input_loop_fut = input_loop(input_tx);
    //    let input_tx_clone = input_tx.clone();
    //    app.on_submit(move |m| {
    //        debug!("sent: {:?}", m);
    //        input_tx_clone.blocking_send(m).unwrap();
    //    });
    //
    //    let topic = if args.topic.is_some() {
    //        args.topic.clone()
    //    } else {
    //        Some(String::from(commit_id.to_string()))
    //    };
    //
    //    app.topic = topic.clone().unwrap();
    //
    //    let topic = gossipsub::IdentTopic::new(format!("{}", app.topic.clone()));
    //
    //    global_rt().spawn(async move {
    //        evt_loop(input_rx, peer_tx, topic).await.unwrap();
    //    });
    //
    //    // recv from peer
    //    let mut tui_msg_adder = app.add_msg_fn();
    //    global_rt().spawn(async move {
    //        while let Some(m) = peer_rx.recv().await {
    //            debug!("recv: {:?}", m);
    //            tui_msg_adder(m);
    //        }
    //    });
    //
    //    // say hi
    //    let input_tx_clone = input_tx.clone();
    //    global_rt().spawn(async move {
    //        tokio::time::sleep(Duration::from_millis(1000)).await;
    //        input_tx_clone
    //            .send(Msg::default().set_kind(MsgKind::Join))
    //            .await
    //            .unwrap();
    //    });
    //
    //    app.run()?;
    //
    //    // say goodbye
    //    input_tx.blocking_send(Msg::default().set_kind(MsgKind::Leave))?;
    //    std::thread::sleep(Duration::from_millis(500));
    //
    Ok(())
}
