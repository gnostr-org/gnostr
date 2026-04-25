use std::{collections::HashMap, error::Error as StdError, io, io::Write, time::SystemTime};

use crate::crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_asyncgit::sync::{
    get_commit_details, get_head, is_workdir_clean, status::get_status, status::StatusType, RepoPath,
};
use gnostr_legit::gitminer::{self, Gitminer};
use once_cell::sync::OnceCell;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use gnostr_asyncgit::types::{
    Event, EventKind, KeySigner, PreEvent, PrivateKey, Signer, Tag, UncheckedUrl, Unixtime,
};

pub async fn run_legit_command(mut opts: gitminer::Options) -> io::Result<()> {
    let _start = SystemTime::now();
    let _system_time = SystemTime::now();
    let repo_path = RepoPath::from(opts.repo.as_str());

    let kind = &opts.kind;
    debug!("gnostr legit:kind={:?}", &kind);

    if !is_workdir_clean(&repo_path, None)
        .map_err(|e| io::Error::other(format!("failed to inspect repository state: {e}")))?
    {
        let status = get_status(&repo_path, StatusType::WorkingDir, None)
            .map_err(|e| io::Error::other(format!("failed to read repository status: {e}")))?;
        debug!("repository is dirty: {:?}", status);
    }

    if opts.message.is_empty() {
        let message = get_status(&repo_path, StatusType::WorkingDir, None)
            .map_err(|e| io::Error::other(format!("failed to build default legit message: {e}")))?
            .into_iter()
            .map(|item| format!("{:?}: {}", item.status, item.path))
            .collect::<Vec<_>>()
            .join("\n");
        opts.message = vec![message];
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
    match gnostr_legit_event(opts.kind, repo_path).await {
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

    info!(
        "gnostr/src/lib/legit/command.rs:224:signed_event={}",
        serde_json::to_string_pretty(&signed_event)?
    );

    let (queue_tx, _queue_rx) = mpsc::channel(100); // Create a channel for internal events
    let mut client = crate::nostr_client::NostrClient::new(queue_tx.clone());

    for relay in BOOTSTRAP_RELAYS.iter().cloned() {
        //TODO get from crawler
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
        content: format!(
            "gnostr/src/lib/legit/command.rs:280:\n{}",
            pubkey_keys.as_hex_string()
        )
        .to_string(),
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

pub async fn gnostr_legit_event(
    kind: Option<u16>,
    repo_path: RepoPath,
) -> Result<(), Box<dyn StdError>> {
    let head_id = get_head(&repo_path)?;
    let details = get_commit_details(&repo_path, head_id)?;
    let commit_hash = details.hash.clone();
    let author_name = details.author.name.clone();
    let author_email = details.author.email.clone();
    let author_time = details.author.time;
    let committer_name = details.committer.as_ref().map(|committer| committer.name.clone());
    let committer_email = details
        .committer
        .as_ref()
        .map(|committer| committer.email.clone());
    let commit_message = details
        .message
        .clone()
        .map(|message| message.combine())
        .unwrap_or_default();
    let serialized_commit = serde_json::to_string(&json!({
        "id": commit_hash,
        "author_name": author_name,
        "author_email": author_email,
        "committer_name": committer_name,
        "committer_email": committer_email,
        "message": commit_message,
        "time": author_time,
    }))?;
    debug!("Serialized commit details:\n{}", serialized_commit);

    let padded_private_key = PrivateKey::try_from_hex_string(&details.padded_hash()).unwrap();
    let padded_keys = KeySigner::from_private_key(padded_private_key, "", 1).unwrap();

    let mut custom_tags = HashMap::new();
    custom_tags.insert("serialized_commit".to_string(), vec![serialized_commit.clone()]);
    custom_tags.insert("gnostr".to_string(), vec!["git".to_string()]);
    custom_tags.insert("GIT".to_string(), vec!["GNOSTR".to_string()]);
    custom_tags.insert(
        padded_keys.clone().public_key().as_hex_string(),
        vec!["GNOSTR".to_string()],
    );

    let empty_hash_private_key = PrivateKey::try_from_hex_string(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    )
    .unwrap();
    let empty_hash_keys = KeySigner::from_private_key(empty_hash_private_key, "", 1).unwrap();
    let custom_tags_clone = custom_tags.clone();
    global_rt().spawn(async move {
        let signed_event = create_event(
            empty_hash_keys,
            custom_tags_clone,
            "gnostr/src/lib/legit/command.rs:asyncgit:event",
        )
        .await;
        println!("signed_event:\n{:?}", signed_event);
        io::stdout().flush().unwrap();
    });

    let serialized_commit_for_kind_event = serialized_commit.clone();
    global_rt().spawn(async move {
        let result: anyhow::Result<()> = async {
            let create_event_result = create_event(
                padded_keys.clone(),
                custom_tags,
                "gnostr/src/lib/legit/command.rs:asyncgit:event",
            )
            .await?;
            println!(
                "Commit-based create_event result:\n{:?}",
                create_event_result
            );
            io::stdout().flush().unwrap();

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

    Ok(())
}
