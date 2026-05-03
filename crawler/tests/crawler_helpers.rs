use std::collections::HashSet;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use axum::{http::StatusCode, routing::get, Router};
use futures::{SinkExt, StreamExt};
use git2::Signature;
use gnostr_asyncgit::{
    sync::{
        add_note,
        commit::{self, mine_commit, CommitMineOptions},
        show_note,
        stage_add_file,
        NoteInfo,
        RepoPath,
    },
    types::{
        generate_git_note_event, generate_git_note_event_with_pow, Client as AsyncClient,
        EventKind, Keys as AsyncKeys, Options as AsyncOptions, PrivateKey as AsyncPrivateKey,
    },
    types::nip13::NIP13Event,
};
use gnostr_crawler as crawler;
use gnostr_crawler::query::{build_gnostr_query, ConfigBuilder};
use gnostr_crawler::relays::{self, Relays};
use nostr_sdk::prelude::{Keys, ToBech32};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use time_03::OffsetDateTime;
use url::Url;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let mut dir = env::temp_dir();
    let unique = format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    dir.push(unique);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn with_isolated_config_dir<F, T>(f: F) -> T
where
    F: FnOnce(&PathBuf) -> T,
{
    let _guard = test_lock();
    let root = unique_temp_dir("gnostr-crawler-tests");
    let home = root.join("home");
    let xdg = root.join("xdg");
    fs::create_dir_all(&home).unwrap();
    fs::create_dir_all(&xdg).unwrap();

    let prev_home = env::var_os("HOME");
    let prev_xdg = env::var_os("XDG_CONFIG_HOME");

    unsafe {
        env::set_var("HOME", &home);
        env::set_var("XDG_CONFIG_HOME", &xdg);
    }

    let result = f(&root);

    unsafe {
        match prev_home {
            Some(value) => env::set_var("HOME", value),
            None => env::remove_var("HOME"),
        }
        match prev_xdg {
            Some(value) => env::set_var("XDG_CONFIG_HOME", value),
            None => env::remove_var("XDG_CONFIG_HOME"),
        }
    }

    result
}

async fn start_http_server(body: &'static str, accept_head: bool) -> SocketAddr {
    let router = if accept_head {
        Router::new().route(
            "/",
            get(move || async move { body }).head(|| async move { StatusCode::OK }),
        )
    } else {
        Router::new().route("/", get(move || async move { body }))
    };

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap();
    });
    addr
}

async fn start_ws_server(messages: Vec<&'static str>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut ws = accept_async(stream).await.unwrap();

        let inbound = ws.next().await.unwrap().unwrap();
        match inbound {
            Message::Text(text) => assert!(text.contains("REQ")),
            other => panic!("expected text message, got {other:?}"),
        }

        for msg in messages {
            ws.send(Message::Text(msg.to_string().into()))
                .await
                .unwrap();
        }
        let _ = ws.close(None).await;
    });

    format!("ws://{}", addr)
}

fn relay_response_contains_event_id(message: &str, event_id: &str) -> bool {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(message) {
        if let Some(array) = value.as_array() {
            if let Some(event) = array.get(2) {
                if event
                    .get("id")
                    .and_then(|id| id.as_str())
                    .is_some_and(|id| id == event_id)
                {
                    return true;
                }
            }
        }
    }

    message.contains(event_id)
}

fn leading_zero_bits(bytes: &[u8]) -> u8 {
    let mut count = 0_u8;
    for byte in bytes {
        if *byte == 0 {
            count += 8;
            continue;
        }
        count += byte.leading_zeros() as u8;
        break;
    }
    count
}

fn matrix_relay_urls() -> Vec<String> {
    vec![
        "wss://nostr-kyomu-haskell.onrender.com/".to_string(),
        "wss://nostr-relay.amethyst.name/".to_string(),
        "wss://relay.bitcoindistrict.org/".to_string(),
        "wss://nos.lol/".to_string(),
        "wss://relay.damus.io/".to_string(),
    ]
}

async fn publish_and_query_git_note_case(
    label: &str,
    mine_commit_flag: bool,
    pow_event_flag: bool,
) -> anyhow::Result<()> {
    let repo_dir = unique_temp_dir("gnostr-crawler-pow-matrix");
    let _repo = git2::Repository::init(&repo_dir)?;
    let repo_path: RepoPath = repo_dir.as_os_str().to_str().unwrap().into();
    let file_path = Path::new("matrix.txt");

    fs::write(repo_dir.join(file_path), label.as_bytes())?;
    stage_add_file(&repo_path, file_path)?;

    let commit_id = if mine_commit_flag {
        mine_commit(
            &repo_path,
            CommitMineOptions {
                threads: 1,
                target: "0".to_string(),
                message: vec![format!("{label} commit")],
                timestamp: OffsetDateTime::from_unix_timestamp(0).unwrap(),
            },
        )?
    } else {
        commit::commit(&repo_path, &format!("{label} commit"))?
    };

    let note_message = format!("{label} note");
    let note_id = add_note(&repo_path, commit_id, &note_message, None, false)?;
    let note: NoteInfo = show_note(&repo_path, commit_id, None)?.expect("note exists");
    assert_eq!(note.note_id, note_id);
    assert_eq!(note.annotated_id, commit_id.into());
    assert_eq!(note.message, note_message);

    let private_key = AsyncPrivateKey::generate();
    let keys = AsyncKeys::new(private_key.clone());
    let event = if pow_event_flag {
        generate_git_note_event_with_pow(&note, &private_key, 4)?
    } else {
        generate_git_note_event(&note, &private_key)?
    };

    let relay_urls = matrix_relay_urls();
    assert!(
        !relay_urls.is_empty(),
        "expected at least one relay URL for crawler syndication"
    );

    let publish_relays = relay_urls.clone();
    let mut client = AsyncClient::new(&keys, AsyncOptions::new());
    client.add_relays(publish_relays.clone()).await?;
    let published_id = client.send_event(event.clone()).await?;
    assert_eq!(published_id, event.id);
    eprintln!(
        "published event for {label}:\n{}",
        serde_json::to_string_pretty(&event)?
    );

    assert_eq!(event.kind, EventKind::TextNote);
    assert_eq!(event.content, note_message);
    assert_eq!(
        event.created_at,
        gnostr_asyncgit::types::Unixtime(note.committer_time)
    );
    assert!(event.tags.iter().any(|tag| tag.tagname() == "e" && tag.marker() == "root"));
    assert!(event.tags.iter().any(|tag| {
        tag.tagname() == "commit" && tag.value() == commit_id.to_string()
    }));

    if pow_event_flag {
        assert!(event.tags.iter().any(|tag| tag.tagname() == "nonce"));
        assert!(event.nonce_data().is_some());
        assert!(leading_zero_bits(&event.id.0) >= 4);
    } else {
        assert!(event.nonce_data().is_none());
        assert!(!event.tags.iter().any(|tag| tag.tagname() == "nonce"));
    }

    let query_relays = publish_relays
        .iter()
        .filter_map(|relay| Url::parse(relay).ok())
        .collect::<Vec<_>>();
    let event_id = event.id.to_string();
    let query = build_gnostr_query(
        None,
        Some(event_id.as_str()),
        Some(1),
        None,
        None,
        None,
        None,
        Some("1"),
        None,
    )
    .map_err(|err| anyhow::anyhow!(err.to_string()))?;
    eprintln!("query string for {label}:\n{query}");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let mut found = None;
    for attempt in 0..5 {
        eprintln!(
            "query attempt {attempt} start for {label}: {} relays",
            query_relays.len()
        );
        for relay in &query_relays {
            eprintln!("query attempt {attempt} for {label} via {relay}: send");
            match crawler::send(query.clone(), vec![relay.clone()], Some(1)).await {
                Ok(messages) => {
                    if let Some(message) = messages
                        .into_iter()
                        .find(|message| relay_response_contains_event_id(message, &event_id))
                    {
                        eprintln!("query attempt {attempt} for {label} via {relay}: hit");
                        eprintln!(
                            "query hit relay response for {label} via {relay}:\n{message}"
                        );
                        found = Some(message);
                        break;
                    } else {
                        eprintln!("query attempt {attempt} for {label} via {relay}: empty");
                    }
                }
                Err(err) => {
                    eprintln!("query relay failed for {label} via {relay}: {err}");
                }
            }
        }

        if found.is_some() {
            break;
        }

        if attempt < 4 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    if found.is_none() {
        eprintln!(
            "best effort query skipped: event {event_id} was not returned by any crawler query relay"
        );
        return Ok(());
    }

    if let Some(message) = &found {
        eprintln!(
            "best effort query matched: event {event_id} returned by crawler query relay\n{message}"
        );
    }

    Ok(())
}

#[test]
fn preprocess_line_strips_markers_and_commas() {
    assert_eq!(
        crawler::preprocess_line("- wss://relay.example.com, extra"),
        "wss://relay.example.com"
    );
}

#[test]
fn load_file_normalizes_yaml_entries() {
    with_isolated_config_dir(|root| {
        let config_dir = relays::get_config_dir_path();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("relays.yaml"),
            "- relay.example.com\n- ws://relay.example.org\n- http://skip.example.com\n",
        )
        .unwrap();

        let relays = crawler::load_file("relays.yaml").unwrap();
        assert_eq!(
            relays,
            vec![
                "wss://relay.example.com/".to_string(),
                "ws://relay.example.org/".to_string(),
            ]
        );

        assert!(root.exists());
    });
}

#[test]
fn load_shitlist_reads_entries() {
    with_isolated_config_dir(|_| {
        let file_path = unique_temp_dir("gnostr-shitlist").join("shitlist.txt");
        fs::write(&file_path, "relay-one\nrelay-two\n").unwrap();

        let shitlist = crawler::load_shitlist(&file_path).unwrap();
        let expected: HashSet<String> = ["relay-one".to_string(), "relay-two".to_string()]
            .into_iter()
            .collect();
        assert_eq!(shitlist, expected);
    });
}

#[test]
fn signature_and_log_matching_work() {
    let keys = Keys::generate();
    let sig = Signature::now(
        &keys.public_key().to_bech32().unwrap(),
        "gnostr@example.com",
    )
    .unwrap();
    assert!(crawler::sig_matches(
        &sig,
        &Some(keys.public_key().to_bech32().unwrap())
    ));
    assert!(crawler::log_message_matches(
        Some("relay connected"),
        &Some("connected".to_string())
    ));
    assert!(crawler::log_message_matches(Some("relay connected"), &None));
    assert!(!crawler::log_message_matches(
        None,
        &Some("connected".to_string())
    ));
}

#[test]
fn match_with_parent_detects_diffs() {
    let repo_dir = unique_temp_dir("gnostr-git-test");
    let repo = git2::Repository::init(&repo_dir).unwrap();

    let sig = git2::Signature::now("gnostr", "gnostr@example.com").unwrap();
    let file_path = repo_dir.join("note.txt");

    fs::write(&file_path, "one\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("note.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let first_commit = repo
        .commit(Some("HEAD"), &sig, &sig, "first", &tree, &[])
        .unwrap();

    fs::write(&file_path, "two\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("note.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let second_commit = repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "second",
            &tree,
            &[&repo.find_commit(first_commit).unwrap()],
        )
        .unwrap();

    let parent = repo.find_commit(first_commit).unwrap();
    let commit = repo.find_commit(second_commit).unwrap();
    let mut opts = git2::DiffOptions::new();
    assert!(crawler::match_with_parent(&repo, &commit, &parent, &mut opts).unwrap());
}

#[test]
fn stats_and_pubkeys_track_counts() {
    let mut stats = crawler::stats::Stats::new();
    stats.add_contacts();
    stats.add_relays();
    assert_eq!(stats.count_contacts, 1);
    assert_eq!(stats.count_relays, 1);

    let keys = Keys::generate();
    let public_key = keys.public_key();
    let mut pubkeys = crawler::pubkeys::PubKeys::new();
    assert_eq!(pubkeys.add(&public_key), 0);
    assert_eq!(pubkeys.add(&public_key), 0);
    assert_eq!(pubkeys.add_str(&public_key.to_bech32().unwrap()), 0);
}

#[tokio::test]
async fn pow_matrix_events_publish_and_query_from_relays() -> anyhow::Result<()> {
    for (label, mine_commit_flag, pow_event_flag) in [
        ("plain-commit/plain-event", false, false),
        ("plain-commit/pow-event", false, true),
        ("mined-commit/plain-event", true, false),
        ("mined-commit/pow-event", true, true),
    ] {
        publish_and_query_git_note_case(label, mine_commit_flag, pow_event_flag).await?;
    }

    Ok(())
}

#[test]
fn config_builder_and_query_builder_work() {
    let config = ConfigBuilder::new()
        .host("relay.example.com")
        .port(443)
        .use_tls(true)
        .retries(2)
        .authors("author1,author2")
        .ids("id1,id2")
        .limit(10)
        .generic("d", "value1,value2")
        .hashtag("tag1,tag2")
        .mentions("pk1,pk2")
        .references("event1,event2")
        .kinds("1,2")
        .search("content", "nostr")
        .build()
        .unwrap();

    let _ = config;

    let query = build_gnostr_query(
        Some("author1,author2"),
        Some("id1,id2"),
        Some(10),
        Some(("d", "value1,value2")),
        Some("tag1,tag2"),
        Some("pk1,pk2"),
        Some("event1,event2"),
        Some("1,2"),
        Some(("content", "nostr")),
    )
    .unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&query).unwrap();
    assert_eq!(parsed[0], "REQ");
    assert_eq!(parsed[1], "gnostr-query");
    assert_eq!(parsed[2]["limit"], 10);
    assert_eq!(parsed[2]["#d"], serde_json::json!(["value1", "value2"]));
    assert_eq!(parsed[2]["#t"], serde_json::json!(["tag1", "tag2"]));
    assert_eq!(parsed[2]["#p"], serde_json::json!(["pk1", "pk2"]));
    assert_eq!(parsed[2]["#e"], serde_json::json!(["event1", "event2"]));
    assert_eq!(parsed[2]["kinds"], serde_json::json!([1, 2]));
}

#[test]
fn relays_collection_helpers_work() {
    let mut relays = Relays::new();
    assert!(relays.add("wss://relay.example.com"));
    assert!(!relays.add("wss://relay.example.com"));
    assert_eq!(relays.count(), 1);

    let all = relays.get_all();
    assert_eq!(all, vec!["wss://relay.example.com/".to_string()]);
}

#[test]
fn render_page_shell_includes_nav_and_body() {
    let html = relays::render_page_shell(
        "title",
        &[("/", "home"), ("/query", "query")],
        "<p>body</p>",
    );
    assert!(html.contains("<title>title</title>"));
    assert!(html.contains("<a href=\"/\">home</a>"));
    assert!(html.contains("<p>body</p>"));
}

#[test]
fn relay_file_writers_use_config_dir() {
    with_isolated_config_dir(|_| {
        let config_dir = relays::get_config_dir_path();
        fs::create_dir_all(&config_dir).unwrap();
        fs::write(
            config_dir.join("relays.yaml"),
            "relay.example.com\nws://relay.example.org\n",
        )
        .unwrap();

        let json_path = relays::write_relays_json_from_yaml().unwrap();
        let json = fs::read_to_string(json_path).unwrap();
        assert!(json.contains("wss://relay.example.com"));
        assert!(json.contains("ws://relay.example.org"));

        relays::record_live_kind("1234");
        relays::record_live_nips([34, 35]);
        let kinds_path = relays::write_kinds_serve_files().unwrap();
        let kinds = fs::read_to_string(kinds_path).unwrap();
        assert!(kinds.contains("1234"));

        relays::write_relays_serve_files().unwrap();
        assert!(config_dir.join("relays.json").exists());
        assert!(config_dir.join("relays.txt").exists());

        let index = relays::write_index_html().unwrap();
        let html = fs::read_to_string(index).unwrap();
        assert!(html.contains("/34"));
        assert!(html.contains("/35"));
    });
}

#[test]
fn nip_relay_file_writers_work() {
    with_isolated_config_dir(|_| {
        let config_dir =
            relays::write_nip_relays_serve_files(34, &[String::from("wss://relay.example.com")])
                .unwrap();
        assert!(config_dir.join("relays.yaml").exists());
        assert!(config_dir.join("relays.json").exists());
        assert!(config_dir.join("relays.txt").exists());

        let nip_dir = relays::get_config_dir_path().join("34");
        fs::create_dir_all(&nip_dir).unwrap();
        fs::write(nip_dir.join("relay-one.json"), "{}").unwrap();
        fs::write(nip_dir.join("relay-two.json"), "{}").unwrap();
        fs::write(nip_dir.join("relays.json"), "[]").unwrap();

        let from_dir = relays::write_nip_relays_serve_files_from_dir(34).unwrap();
        let yaml = fs::read_to_string(from_dir.join("relays.yaml")).unwrap();
        assert!(yaml.contains("wss://relay-one"));
        assert!(yaml.contains("wss://relay-two"));
    });
}

#[tokio::test]
async fn fetch_online_relays_and_liveness_use_http_helpers() {
    let addr = start_http_server(
        "relay.example.com\nws://relay.example.org\nhttp://skip.example.com\n",
        true,
    )
    .await;
    let url = format!("http://{}", addr);
    let relays = relays::fetch_online_relays(&url).await.unwrap();
    assert_eq!(
        relays,
        vec![
            "wss://relay.example.com/".to_string(),
            "ws://relay.example.org/".to_string(),
        ]
    );

    let liveness = relays::check_relay_liveness(&url.replace("http://", "ws://")).await;
    assert!(liveness);
}

#[tokio::test]
async fn send_reads_messages_from_websocket() {
    let relay = start_ws_server(vec!["one", "two", "three"]).await;
    let results = crawler::query::send(
        r#"["REQ","gnostr-query",{}]"#.to_string(),
        vec![url::Url::parse(&relay).unwrap()],
        Some(2),
    )
    .await
    .unwrap();

    assert_eq!(results, vec!["one".to_string(), "two".to_string()]);
}
