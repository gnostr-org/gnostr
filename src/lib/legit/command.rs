use std::{
    borrow::Cow,
    collections::HashMap,
    error::Error as StdError,
    io,
    io::Write,
    process::Command,
    time::{Duration, SystemTime},
};

use anyhow::anyhow;
use git2::{ObjectType, Repository, RepositoryState};
use gnostr_asyncgit::sync::commit::{deserialize_commit, serialize_commit};
use crate::crawler::processor::BOOTSTRAP_RELAYS;
use crate::nostr_client;
use gnostr_legit::gitminer::{self, Gitminer};
use once_cell::sync::OnceCell;
use serde_json::{self, Value};
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use gnostr_asyncgit::{
    types::{
        Event, EventKind, KeySigner, PreEvent, PrivateKey, PublicKey, Signer, Tag, UncheckedUrl,
        Unixtime,
    },
};
use crate::utils::{parse_json, split_json_string};

pub async fn run_legit_command(mut opts: gitminer::Options) -> io::Result<()> {
    let _start = SystemTime::now();
    let _system_time = SystemTime::now();

    let kind = &opts.kind;
    debug!("gnostr legit:kind={:?}", &kind);

	for message in &opts.message {
		println!("----------->>>\nopts.message={}\n", message);
	}

    let repo = Repository::discover(&opts.repo).expect("Couldn't open repository");

    if repo.state() != RepositoryState::Clean {
        let repo_state = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "git status"])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg("git")
                .arg("status")
                .output()
                .expect("failed to execute process")
        };

        let _state = String::from_utf8(repo_state.stdout)
            .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
            .unwrap();
    }



    if opts.message.is_empty() {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "git status"])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg("git")
                .arg("diff")
                .output()
                .expect("failed to execute process")
        };

        //TODO if message contains "Nothing to commit"
        let message = String::from_utf8(output.stdout)
            .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
            .unwrap();
        opts.message = [message.to_string()].to_vec();
    }

    //TODO create nostr event and inject into multi line message BEFORE mining
    //TODO --event-pow for nostr event creation
    let mut miner = Gitminer::new(opts.clone())
        .map_err(|e| io::Error::other(format!("Failed to start git miner: {}", e)))?;
    debug!("Gitminer options: {:?}", opts);

    let _hash = miner
        .mine()
        .map_err(|e| io::Error::other(format!("Failed to generate commit: {}", e)))?;

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
            Err(io::Error::other(e.to_string()))
        }
    }
}

pub async fn create_event_with_custom_tags(
    keys: &KeySigner,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> anyhow::Result<(Event, PreEvent)> {
    let mut tags = Vec::new();
    for (tag_name, tag_values) in custom_tags {
        info!("tag_name={:?}", tag_name);
        info!("tag_values={:?}", tag_values);
        // Use the first value for now, similar to how nostr_sdk's Tag::parse might
        // behave in this context
        if let Some(value) = tag_values.get(0) {
            tags.push(Tag::new(&[&tag_name, value]));
        }
    }

    let pre_event = PreEvent {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::TextNote, //TODO kind 1617 patch default
        tags,
        content: content.to_string(),
    };

    let id = pre_event.hash().unwrap();
    let sig = keys.sign_id(id).unwrap();

    let signed_event = Event {
        id,
        pubkey: pre_event.pubkey,
        created_at: pre_event.created_at,
        kind: pre_event.kind,
        tags: pre_event.tags.clone(), // Clone tags for the signed event
        content: pre_event.content.clone(), // Clone content for the signed event
        sig,
    };

    Ok((signed_event, pre_event))
}

pub async fn create_unsigned_event(
    keys: &KeySigner,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> anyhow::Result<PreEvent> {
    let mut tags = Vec::new();
    for (tag_name, tag_values) in custom_tags {
        if let Some(value) = tag_values.get(0) {
            tags.push(Tag::new(&[&tag_name, value]));
        }
    }

    let pre_event = PreEvent {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::TextNote,
        tags,
        content: content.to_string(),
    };
    Ok(pre_event)
}

pub async fn create_kind_event(
    keys: &KeySigner,
    kind: u16,
    content: &str,
    custom_tags: HashMap<String, Vec<String>>,
) -> anyhow::Result<(Event, PreEvent)> {
    let mut tags = Vec::new();
    for (tag_name, tag_values) in custom_tags {
        if let Some(value) = tag_values.get(0) {
            tags.push(Tag::new(&[&tag_name, value]));
        }
    }

    let pre_event = PreEvent {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(kind as u32),
        tags,
        content: content.to_string(),
    };

    let id = pre_event.hash().unwrap();
    let sig = keys.sign_id(id).unwrap();

    let signed_event = Event {
        id,
        pubkey: pre_event.pubkey,
        created_at: pre_event.created_at,
        kind: pre_event.kind,
        tags: pre_event.tags.clone(),
        content: pre_event.content.clone(),
        sig,
    };

    Ok((signed_event, pre_event))
}

pub async fn create_event(
    keys: KeySigner,
    custom_tags: HashMap<String, Vec<String>>,
    content: &str,
) -> anyhow::Result<Event> {
    // Changed return type

    //let content = "Hello, Nostr with custom tags!";

    let (signed_event, _unsigned_event) =
        create_event_with_custom_tags(&keys, content, custom_tags).await?;

    info!("gnostr/src/lib/legit/command.rs:224:signed_event={}", serde_json::to_string_pretty(&signed_event)?);

    let (queue_tx, _queue_rx) = mpsc::channel(100); // Create a channel for internal events
    let mut client = crate::nostr_client::NostrClient::new(queue_tx.clone());

    for relay in BOOTSTRAP_RELAYS.iter().cloned() { //TODO get from crawler
        debug!("gnostr/src/lib/legit/command.rs:230:relay={}", relay);
        client
            .connect_relay(UncheckedUrl(relay.to_string()))
            .await?;
    }

    // Connect to the relays.
    // client.send_event - signed_event
    client.send_event(signed_event.clone()).await?;

    info!("{}", serde_json::to_string_pretty(&signed_event)?);

    print!("signed_event sent:\n{:?}", signed_event);

    debug!("signed_event.content: {}", signed_event.content);

    debug!(
        "signed_event.pubkey: {}",
        signed_event.pubkey.as_hex_string()
    );

    debug!("signed_event.kind: {:?}", signed_event.kind);

    debug!("signed_event.tags: {:?}", signed_event.tags);

    //

    // Publish a text note

	//TODO test tags
    let pubkey_keys = keys.public_key();
    info!("pubkey={}", pubkey_keys.as_hex_string());

    let mut tags: Vec<Tag> = Vec::new();
    tags.push(Tag::new_pubkey(pubkey_keys, None, None));
    tags.push(Tag::new(&["gnostr", "1 2 3 4 11 22 33 44"]));
    tags.push(Tag::new(&["gnostr", "1 2 3 4 11 22 33"]));
    tags.push(Tag::new(&["gnostr", "1 2 3 4 11 22"]));
    tags.push(Tag::new(&["gnostr", "1 2 3 4 11"]));
    tags.push(Tag::new(&["gnostr", "1 2 3 4"]));
    tags.push(Tag::new(&["gnostr", "1 2 3"]));
    tags.push(Tag::new(&["gnostr", "1 2"]));
    tags.push(Tag::new(&["gnostr", "1"]));
    tags.push(Tag::new(&["gnostr", ""]));

    let pre_event = PreEvent {
        pubkey: pubkey_keys,
        created_at: Unixtime::now(),
        kind: EventKind::TextNote,
        tags,
        content: format!("gnostr/src/lib/legit/command.rs:280:\n{}", pubkey_keys.as_hex_string()).to_string(),
    };

    let id = pre_event.hash().unwrap();
    let sig = keys.sign_id(id).unwrap();

    let text_note_event = Event {
        id,
        pubkey: pre_event.pubkey,
        created_at: pre_event.created_at,
        kind: pre_event.kind,
        tags: pre_event.tags,
        content: pre_event.content,
        sig,
    };

    let output_send_event = client.send_event(text_note_event.clone()).await?;
    print!("output_send_event={:?}", output_send_event);

    let mut filter_one = gnostr_asyncgit::types::Filter::new();
    filter_one
        .add_author(&pubkey_keys.into())
        .add_event_kind(gnostr_asyncgit::types::EventKind::TextNote);
    filter_one.limit = Some(10);

    // let events = client
    //     .fetch_events(vec![filter_one], Some(Duration::from_secs(10)))
    //     .await?;

    // for event in events.into_iter() {
    //     println!("{}", serde_json::to_string_pretty(&event)?);
    // }

    // another filter

    let test_author_pubkey = gnostr_asyncgit::types::PublicKey::try_from_bech32_string(
        "npub1drvpzev3syqt0kjrls50050uzf25gehpz9vgdw08hvex7e0vgfeq0eseet",
        true,
    )
    .unwrap();

    info!(
        "test_author_pubkey={}",
        test_author_pubkey.as_bech32_string()
    );

    let mut filter_test_author = gnostr_asyncgit::types::Filter::new();
    filter_test_author
        .add_author(&test_author_pubkey.into())
        .add_event_kind(gnostr_asyncgit::types::EventKind::TextNote);
    filter_test_author.limit = Some(10);

    // let events = client
    //     .fetch_events(vec![filter_test_author], Some(Duration::from_secs(10)))
    //     .await?;

    // for event in events.into_iter() {
    //     info!("test_author:\n\n{}", serde_json::to_string_pretty(&event)?);
    // }

    Ok(signed_event)
}

//async tasks
pub fn global_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

pub async fn gnostr_legit_event(kind: Option<u16>) -> Result<(), Box<dyn StdError>> {
    // gnostr_legit_event
    let empty_hash_private_key = PrivateKey::try_from_hex_string(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )
    .unwrap();
    let empty_hash_keys = KeySigner::from_private_key(empty_hash_private_key, "", 1).unwrap();

    //create a HashMap of custom_tags
    //used to insert commit tags
    let mut custom_tags = HashMap::new();
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    let custom_tags_clone = custom_tags.clone();
    global_rt().spawn(async move {
        //send to create_event function with &"custom content"
        let signed_event = create_event(empty_hash_keys, custom_tags_clone, "gnostr/src/lib/legit/command.rs365:gnostr-legit:event").await;
        println!("signed_event:\n{:?}", signed_event);
        io::stdout().flush().unwrap(); // Flush stdout
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
    custom_tags.insert("serialized_commit".to_string(), vec![serialized_commit.clone()]);
    let commit_id = commit.id().to_string();
    //some info wrangling
    debug!("commit_id:\n{}", commit_id);
    let padded_commit_id = format!("{:0>64}", commit_id);

    //// commit based keys
    //let keys = generate_nostr_keys_from_commit_hash(&commit_id)?;
    //info!("keys.secret_key():\n{:?}", keys.secret_key());
    //info!("keys.public_key():\n{}", keys.public_key());

    //parse keys from sha256 hash
    let padded_private_key = PrivateKey::try_from_hex_string(&padded_commit_id).unwrap();
    let padded_keys = KeySigner::from_private_key(padded_private_key, "", 1).unwrap();
    let serialized_commit_for_kind_event = serialized_commit.clone();

    //create a HashMap of custom_tags
    //used to insert commit tags
    let mut custom_tags = HashMap::new();
    custom_tags.insert("serialized_commit".to_string(), vec![serialized_commit_for_kind_event.clone().to_string()]);
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    custom_tags.insert(
        padded_keys.clone().public_key().as_hex_string(),
        vec!["GNOSTR".to_string()],
    );

    global_rt().spawn(async move {
        let result: anyhow::Result<()> = async {
            //send to create_event function with &"custom content"
            //send to create_event function with &"custom content"
            let create_event_result =
                create_event(padded_keys.clone(), custom_tags, "gnostr/src/lib/legit/command.rs:412:gnostr-legit:event").await?;
            println!(
                "Commit-based create_event result:\n{:?}",
                create_event_result
            );
            io::stdout().flush().unwrap(); // Flush stdout

            // The existing error logging for create_kind_event remains.
            if let Err(e) = create_kind_event(
                &padded_keys,
                kind.unwrap_or(1),
                &serialized_commit_for_kind_event,
                HashMap::new(),
            )
            .await
            {
                error!("Failed to create kind event: {:?}", e);
            }

            if let Err(e) = create_kind_event(
                &padded_keys,
                kind.unwrap_or(1),
                &serialized_commit_for_kind_event,
                HashMap::new(),
            )
            .await
            {
                error!("Failed to create kind event: {:?}", e);
            }
            Ok(())
        }
        .await;
        if let Err(e) = result {
            error!("Error in gnostr_legit_event spawned task: {:?}", e);
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
    if let Some(Value::Array(arr)) = value.get("parents") {
        if let Some(parent) = arr.first() {
            debug!("parent:\n{}", parent.as_str().unwrap_or("initial commit"));
        }
        if let Some(parent) = arr.get(1) {
            debug!("parent:\n{}", parent.as_str().unwrap_or(""));
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
        let parts = split_json_string(message, "\n");
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
        let result: anyhow::Result<()> = async {
            //parse keys from sha256 hash
            let padded_private_key = PrivateKey::try_from_hex_string(&padded_commit_id).unwrap();
            let padded_keys = KeySigner::from_private_key(padded_private_key, "", 1).unwrap();
            //create nostr client with commit based keys
            //let client = Client::new(keys);
            let (queue_tx, _queue_rx) = mpsc::channel(100); // Create a channel for internal events
            //note nostr_client not in gnostr_asyncgit::types
            let mut client = crate::nostr_client::NostrClient::new(queue_tx.clone());

            for relay in BOOTSTRAP_RELAYS.iter().cloned() {
                debug!("{}", relay);
                client
                    .connect_relay(UncheckedUrl(relay.to_string()))
                    .await?;
            }

            //build git gnostr event
            let pre_event = PreEvent {
                pubkey: padded_keys.public_key(),
                created_at: Unixtime::now(),
                kind: EventKind::TextNote,
                tags: vec![], //insert tag serialized_commit?
                content: serialized_commit.clone(),
            };

            let id = pre_event.hash().unwrap();
            let sig = padded_keys.sign_id(id).unwrap();

            let git_gnostr_event = Event {
                id,
                pubkey: pre_event.pubkey,
                created_at: pre_event.created_at,
                kind: pre_event.kind,
                tags: pre_event.tags,
                content: pre_event.content,
                sig,
            };

            //send git gnostr event
            let output = client.send_event(git_gnostr_event.clone()).await.expect("");
            println!("\ngnostr/src/lib/legit/command.rs:591:\n{:?}\n", output);
            Ok(())
        }
        .await;
        if let Err(e) = result {
            error!("Error in gnostr_legit_event spawned task: {:?}", e);
        }
    });

    Ok(())
}
