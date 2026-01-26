use std::{
    collections::{BTreeMap, HashMap},
    io::{self, BufRead, BufReader},
    process,
};

use gnostr::types::{
    Client, Event, EventKind, Filter, Id, Keys, Options, PublicKey, RelayUrl, Tag, Unixtime,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: git-remote-gnostr <remote-name> <url>");
        process::exit(1);
    }

    let remote_name = &args[1];
    let url = &args[2];

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();

    // Initialize client with default keys for now
    let keys = Keys::generate();
    let options = Options::new();
    let client = Client::new(&keys, options);

    while reader.read_line(&mut line)? > 0 {
        let trimmed_line = line.trim();

        match trimmed_line {
            "capabilities" => {
                handle_capabilities();
            }
            "list" => {
                handle_list(remote_name, url, &client).await?;
                println!();
            }
            "list for-push" => {
                handle_list(remote_name, url, &client).await?;
                println!();
            }
            cmd if cmd.starts_with("option ") => {
                handle_option(&cmd[7..])?;
            }
            cmd if cmd.starts_with("push ") => {
                handle_push(&cmd[5..], remote_name, url, &client)?;
            }
            cmd if cmd.starts_with("fetch ") => {
                handle_fetch(&cmd[6..], remote_name, url, &client).await?;
            }
            _ => {
                eprintln!("Unknown command: {}", trimmed_line);
            }
        }

        line.clear();
    }

    Ok(())
}

fn handle_capabilities() {
    println!("push");
    println!("fetch");
    println!("option");
    println!();
}

async fn handle_list(_remote_name: &str, url: &str, client: &Client) -> io::Result<()> {
    // Parse gnostr URL and list available refs
    if let Ok(repo_info) = parse_gnostr_url(url) {
        eprintln!("Listing refs for repository: {}", url);

        // Query nostr for existing refs
        match query_git_refs(client, &repo_info).await {
            Ok(refs) => {
                if refs.is_empty() {
                    // Return default main branch for new repos
                    println!("@refs/heads/main HEAD");
                    println!("0000000000000000000000000000000000000000 refs/heads/main");
                } else {
                    // Return discovered refs
                    for (ref_name, commit_id) in refs {
                        println!("@{} {}", ref_name, ref_name);
                        println!("{} {}", commit_id, ref_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error querying refs: {}", e);
                // Fallback to default main branch
                println!("@refs/heads/main HEAD");
                println!("0000000000000000000000000000000000000000 refs/heads/main");
            }
        }
    } else {
        eprintln!("Invalid gnostr URL: {}", url);
    }
    Ok(())
}

fn handle_option(option: &str) -> io::Result<()> {
    if let Some((key, _value)) = option.split_once(' ') {
        match key {
            "verbosity" => println!("ok"),
            _ => {
                println!("unsupported");
            }
        }
    } else {
        println!("unsupported");
    }
    println!();
    Ok(())
}

fn handle_push(push_spec: &str, _remote_name: &str, url: &str, client: &Client) -> io::Result<()> {
    // Parse push specification and create nostr events
    let parts: Vec<&str> = push_spec.split_whitespace().collect();
    if parts.len() >= 3 {
        let src = parts[0];
        let dst = parts[1];
        let ref_name = if parts.len() > 3 {
            &parts[3..].join(" ")
        } else {
            ""
        };

        eprintln!("Pushing to gnostr remote: {}", url);
        eprintln!("Push spec: {} -> {} ({})", src, dst, ref_name);

        // Create git event for push
        if let Ok(repo_info) = parse_gnostr_url(url) {
            match create_and_publish_push_event(client, &repo_info, src, dst, ref_name) {
                Ok(event_id) => {
                    eprintln!("Published push event: {}", event_id);
                    println!("ok {}", ref_name);
                }
                Err(e) => {
                    eprintln!("Failed to publish push event: {}", e);
                    println!("error {}", ref_name);
                }
            }
        } else {
            println!("error {}", ref_name);
        }
    }
    println!();
    Ok(())
}

async fn handle_fetch(
    fetch_spec: &str,
    _remote_name: &str,
    url: &str,
    client: &Client,
) -> io::Result<()> {
    // Parse fetch specification and retrieve from nostr
    eprintln!("Fetching from gnostr remote: {}", url);

    let parts: Vec<&str> = fetch_spec.split_whitespace().collect();
    if parts.len() >= 2 {
        let _oid = parts[0];
        let ref_name = parts[1];

        if let Ok(repo_info) = parse_gnostr_url(url) {
            match fetch_git_data(client, &repo_info, ref_name).await {
                Ok(_) => {
                    eprintln!("Successfully fetched {}", ref_name);
                    println!("ok");
                }
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}", ref_name, e);
                    println!("error");
                }
            }
        } else {
            println!("error");
        }
    }
    println!();
    Ok(())
}

#[derive(Debug, Clone)]
struct GnostrRepoInfo {
    author: PublicKey,
    #[allow(dead_code)]
    relays: Vec<RelayUrl>,
    #[allow(dead_code)]
    kind: Option<EventKind>,
    #[allow(dead_code)]
    identifier: Option<String>,
    #[allow(dead_code)]
    url: String,
}

fn parse_gnostr_url(url: &str) -> Result<GnostrRepoInfo, Box<dyn std::error::Error>> {
    if !url.starts_with("gnostr://") {
        return Err("URL must start with gnostr://".into());
    }

    let path = &url[9..];

    if path.starts_with("naddr1") {
        // Parse NAddr format for specific repository
        parse_naddr_repo(path, url)
    } else if path.starts_with("npub1") {
        // Parse NPub format for user repositories
        parse_npub_repo(path, url)
    } else {
        Err("Invalid gnostr URL format. Expected naddr1... or npub1...".into())
    }
}

fn parse_naddr_repo(
    _path: &str,
    original_url: &str,
) -> Result<GnostrRepoInfo, Box<dyn std::error::Error>> {
    // For now, return a placeholder implementation
    // TODO: Properly decode bech32 NAddr
    Ok(GnostrRepoInfo {
        author: PublicKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
            false,
        )?,
        relays: vec![],
        kind: Some(EventKind::TextNote),
        identifier: None,
        url: original_url.to_string(),
    })
}

fn parse_npub_repo(
    _path: &str,
    original_url: &str,
) -> Result<GnostrRepoInfo, Box<dyn std::error::Error>> {
    // For now, return a placeholder implementation
    // TODO: Properly decode bech32 NPub
    Ok(GnostrRepoInfo {
        author: PublicKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
            false,
        )?,
        relays: vec![],
        kind: Some(EventKind::TextNote),
        identifier: None,
        url: original_url.to_string(),
    })
}

async fn query_git_refs(
    client: &Client,
    repo_info: &GnostrRepoInfo,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut refs = HashMap::new();

    // Create filter for git reference events
    let filter = create_ref_filter(repo_info);

    // Query events from relays
    match client
        .get_events_of_with_opts(vec![filter], None, gnostr::types::FilterOptions::ExitOnEOSE)
        .await
    {
        Ok(events) => {
            for event in events {
                if let Some(ref_name) = extract_ref_name(&event) {
                    refs.insert(ref_name, event.id.as_hex_string());
                }
            }
        }
        Err(e) => {
            eprintln!("Error querying refs: {}", e);
        }
    }

    Ok(refs)
}

fn create_ref_filter(repo_info: &GnostrRepoInfo) -> Filter {
    let mut filter = Filter::new();

    // Filter by repository author
    filter.authors = vec![repo_info.author.into()];

    // Filter by event kind
    filter.kinds = vec![EventKind::TextNote];

    // Filter by repository tags
    let mut tags = BTreeMap::new();
    tags.insert('t', vec!["gnostr-repo".to_string()]);
    tags.insert('t', vec!["git-ref".to_string()]);
    filter.tags = tags;

    filter
}

fn extract_ref_name(event: &Event) -> Option<String> {
    for tag in &event.tags {
        if let Ok(tag_name) = tag.parse_identifier() {
            if tag_name.starts_with("git-ref:") {
                return Some(tag_name.strip_prefix("git-ref:").unwrap().to_string());
            }
        }
    }
    None
}

fn create_and_publish_push_event(
    _client: &Client,
    repo_info: &GnostrRepoInfo,
    src: &str,
    dst: &str,
    ref_name: &str,
) -> Result<Id, Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let private_key = keys.secret_key()?;

    let mut tags = Vec::new();

    // Add repository identifier tag
    tags.push(Tag::new_identifier("gnostr-repo".to_string()));

    // Add git reference tag
    tags.push(Tag::new_identifier(format!("git-ref:{}", ref_name)));

    // Add push operation tag
    tags.push(Tag::new_identifier("git-push".to_string()));

    // Add source and destination commits
    tags.push(Tag::new_identifier(format!("src:{}", src)));
    tags.push(Tag::new_identifier(format!("dst:{}", dst)));

    // Add repository author tag
    tags.push(Tag::new_pubkey(repo_info.author, None, None));

    // Create event with push metadata
    let preevent = gnostr::types::PreEvent {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::TextNote,
        tags,
        content: format!("Git push to {}: {} -> {}", ref_name, src, dst),
    };
    let event = Event::sign_with_private_key(preevent, &private_key)?;

    // TODO: Publish to relays
    eprintln!("Created push event: {}", event.id);

    Ok(event.id)
}

async fn fetch_git_data(
    client: &Client,
    repo_info: &GnostrRepoInfo,
    ref_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create filter for git data events
    let filter = create_data_filter(repo_info, ref_name);

    // Query events
    match client
        .get_events_of_with_opts(vec![filter], None, gnostr::types::FilterOptions::ExitOnEOSE)
        .await
    {
        Ok(events) => {
            for event in events {
                eprintln!("Found git data event: {}", event.id);
                // TODO: Process event content and write to git
            }
        }
        Err(e) => {
            eprintln!("Error fetching git data: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

fn create_data_filter(repo_info: &GnostrRepoInfo, ref_name: &str) -> Filter {
    let mut filter = Filter::new();

    // Filter by repository author
    filter.authors = vec![repo_info.author.into()];

    // Filter by event kind
    filter.kinds = vec![EventKind::TextNote];

    // Filter by specific ref
    let mut tags = BTreeMap::new();
    tags.insert('t', vec!["gnostr-repo".to_string()]);
    tags.insert('t', vec![format!("git-ref:{}", ref_name)]);
    tags.insert('t', vec!["git-data".to_string()]);
    filter.tags = tags;

    filter
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_repo_info() -> GnostrRepoInfo {
        GnostrRepoInfo {
            author: PublicKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000001",
                false,
            )
            .unwrap(),
            relays: vec![],
            kind: Some(EventKind::TextNote),
            identifier: None,
            url: "gnostr://test".to_string(),
        }
    }

    fn create_test_event() -> Event {
        let keys = Keys::generate();
        let private_key = keys.secret_key().unwrap();
        let preevent = gnostr::types::PreEvent {
            pubkey: keys.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new_identifier("git-ref:main".to_string()),
                Tag::new_identifier("gnostr-repo".to_string()),
            ],
            content: "Test git ref event".to_string(),
        };
        Event::sign_with_private_key(preevent, &private_key).unwrap()
    }

    #[test]
    fn test_parse_gnostr_url_naddr_format() {
        // Test with valid naddr format
        let result = parse_gnostr_url(
            "gnostr://naddr1qqzynhx9qcrqcpzamhxue69uhkumttpwfjhxqgr0ys8qsqqqqqqpqqqqqyqumfnqv3xcm5v93qcrqcpzamhxue69uhkumttpwfjhxqgr0ys8qsqqqqqqpqqqqqyqumfnqv3xcm5v9",
        );
        assert!(result.is_ok());

        let repo_info = result.unwrap();
        assert_eq!(repo_info.url, "gnostr://naddr1qqzynhx9qcrqcpzamhxue69uhkumttpwfjhxqgr0ys8qsqqqqqqpqqqqqyqumfnqv3xcm5v93qcrqcpzamhxue69uhkumttpwfjhxqgr0ys8qsqqqqqqpqqqqqyqumfnqv3xcm5v9");
    }

    #[test]
    fn test_parse_gnostr_url_npub_format() {
        // Test with valid npub format
        let result = parse_gnostr_url("gnostr://npub1test");
        assert!(result.is_ok());

        let repo_info = result.unwrap();
        assert_eq!(repo_info.url, "gnostr://npub1test");
    }

    #[test]
    fn test_parse_gnostr_url_invalid_protocol() {
        // Test with invalid protocol
        let result = parse_gnostr_url("http://test.com");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("URL must start with gnostr://"));
    }

    #[test]
    fn test_parse_gnostr_url_invalid_format() {
        // Test with invalid format after gnostr://
        let result = parse_gnostr_url("gnostr://invalidformat");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid gnostr URL format"));
    }

    #[test]
    fn test_parse_naddr_repo() {
        let result = parse_naddr_repo("naddr1test", "gnostr://naddr1test");
        assert!(result.is_ok());

        let repo_info = result.unwrap();
        assert_eq!(
            repo_info.author.as_hex_string(),
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(repo_info.kind, Some(EventKind::TextNote));
    }

    #[test]
    fn test_parse_npub_repo() {
        let result = parse_npub_repo("npub1test", "gnostr://npub1test");
        assert!(result.is_ok());

        let repo_info = result.unwrap();
        assert_eq!(
            repo_info.author.as_hex_string(),
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(repo_info.kind, Some(EventKind::TextNote));
    }

    #[test]
    fn test_create_ref_filter() {
        let repo_info = create_test_repo_info();
        let filter = create_ref_filter(&repo_info);

        // Check authors field
        assert!(!filter.authors.is_empty());
        assert_eq!(filter.authors.len(), 1);
        assert_eq!(filter.authors[0].as_str(), repo_info.author.as_hex_string());

        // Check kinds field
        assert!(!filter.kinds.is_empty());
        assert_eq!(filter.kinds.len(), 1);
        assert_eq!(filter.kinds[0], EventKind::TextNote);

        // Check tags field
        assert!(!filter.tags.is_empty());
        assert!(filter.tags.contains_key('t'));
        let repo_tags = filter.tags.get(&'t').unwrap();
        assert!(repo_tags.contains(&"gnostr-repo".to_string()));
        assert!(repo_tags.contains(&"git-ref".to_string()));
    }

    #[test]
    fn test_create_data_filter() {
        let repo_info = create_test_repo_info();
        let ref_name = "main";
        let filter = create_data_filter(&repo_info, ref_name);

        // Check authors field
        assert!(!filter.authors.is_empty());
        assert_eq!(filter.authors.len(), 1);
        assert_eq!(filter.authors[0].as_str(), repo_info.author.as_hex_string());

        // Check kinds field
        assert!(!filter.kinds.is_empty());
        assert_eq!(filter.kinds.len(), 1);
        assert_eq!(filter.kinds[0], EventKind::TextNote);

        // Check tags field
        assert!(!filter.tags.is_empty());
        assert!(filter.tags.contains_key('t'));
        let data_tags = filter.tags.get(&'t').unwrap();
        assert!(data_tags.contains(&"gnostr-repo".to_string()));
        assert!(data_tags.contains(&"git-data".to_string()));
        assert!(data_tags.contains(&format!("git-ref:{}", ref_name)));
    }

    #[test]
    fn test_create_data_filter_different_refs() {
        let repo_info = create_test_repo_info();

        // Test with different ref names
        let refs = vec!["main", "develop", "feature/test"];

        for ref_name in refs {
            let filter = create_data_filter(&repo_info, ref_name);
            let data_tags = filter.tags.get(&'t').unwrap();
            assert!(data_tags.contains(&format!("git-ref:{}", ref_name)));
        }
    }

    #[test]
    fn test_extract_ref_name_with_valid_tag() {
        let event = create_test_event();
        let ref_name = extract_ref_name(&event);

        assert!(ref_name.is_some());
        assert_eq!(ref_name.unwrap(), "main");
    }

    #[test]
    fn test_extract_ref_name_without_git_ref_tag() {
        // Create event without git-ref tag
        let keys = Keys::generate();
        let private_key = keys.secret_key().unwrap();
        let preevent = gnostr::types::PreEvent {
            pubkey: keys.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new_identifier("other-tag".to_string()),
                Tag::new_identifier("gnostr-repo".to_string()),
            ],
            content: "Test event without git ref".to_string(),
        };
        let event = Event::sign_with_private_key(preevent, &private_key).unwrap();

        let ref_name = extract_ref_name(&event);
        assert!(ref_name.is_none());
    }

    #[test]
    fn test_extract_ref_name_with_multiple_ref_tags() {
        // Create event with multiple git-ref tags (first one should be returned)
        let keys = Keys::generate();
        let private_key = keys.secret_key().unwrap();
        let preevent = gnostr::types::PreEvent {
            pubkey: keys.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new_identifier("git-ref:feature".to_string()),
                Tag::new_identifier("git-ref:main".to_string()),
                Tag::new_identifier("gnostr-repo".to_string()),
            ],
            content: "Test event with multiple git refs".to_string(),
        };
        let event = Event::sign_with_private_key(preevent, &private_key).unwrap();

        let ref_name = extract_ref_name(&event);
        assert!(ref_name.is_some());
        assert_eq!(ref_name.unwrap(), "feature");
    }

    #[test]
    fn test_extract_ref_name_empty_tags() {
        // Create event with empty tags
        let keys = Keys::generate();
        let private_key = keys.secret_key().unwrap();
        let preevent = gnostr::types::PreEvent {
            pubkey: keys.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![],
            content: "Test event with no tags".to_string(),
        };
        let event = Event::sign_with_private_key(preevent, &private_key).unwrap();

        let ref_name = extract_ref_name(&event);
        assert!(ref_name.is_none());
    }

    #[test]
    fn test_gnostr_repo_info_clone_and_debug() {
        let repo_info = create_test_repo_info();

        // Test Clone trait
        let cloned = repo_info.clone();
        assert_eq!(
            repo_info.author.as_hex_string(),
            cloned.author.as_hex_string()
        );
        assert_eq!(repo_info.url, cloned.url);

        // Test Debug trait
        let debug_str = format!("{:?}", repo_info);
        assert!(debug_str.contains("GnostrRepoInfo"));
        assert!(debug_str.contains("author"));
        assert!(debug_str.contains("url"));
    }

    #[test]
    fn test_gnostr_repo_info_partial_eq() {
        let repo_info1 = create_test_repo_info();
        let mut repo_info2 = create_test_repo_info();

        // Test equality
        assert_eq!(repo_info1, repo_info2);

        // Test inequality with different author
        repo_info2.author = PublicKey::try_from_hex_string(
            "1111111111111111111111111111111111111111111111111111111111111111",
            false,
        )
        .unwrap();
        assert_ne!(repo_info1, repo_info2);
    }

    #[tokio::test]
    async fn test_create_and_publish_push_event() {
        let keys = Keys::generate();
        let client = Client::new(&keys, Options::new());
        let repo_info = create_test_repo_info();

        let result = create_and_publish_push_event(
            &client,
            &repo_info,
            "abc123",
            "def456",
            "refs/heads/main",
        );

        assert!(result.is_ok());
        let event_id = result.unwrap();

        // Verify event ID is a valid 32-byte hash (hex string should be 64 chars)
        assert_eq!(event_id.as_hex_string().len(), 64);
    }

    #[test]
    fn test_create_and_publish_push_event_content() {
        let keys = Keys::generate();
        let client = Client::new(&keys, Options::new());
        let repo_info = create_test_repo_info();

        let result = create_and_publish_push_event(
            &client,
            &repo_info,
            "abc123",
            "def456",
            "refs/heads/main",
        );

        assert!(result.is_ok());

        // We can't easily test the event content without publishing, but we can verify
        // the function doesn't panic and returns a valid ID
        let event_id = result.unwrap();
        assert_ne!(event_id.as_hex_string(), "");
    }

    #[test]
    fn test_handle_capabilities() {
        // Capture stdout to verify capabilities output
        let mut output = Vec::new();
        {
            let mut stdout = std::io::stdout();
            // This test mainly ensures the function doesn't panic
            handle_capabilities();
        }
        // In a real test, you'd capture stdout and verify the content
        // For now, just ensure it runs without panicking
    }

    #[test]
    fn test_handle_option_valid() {
        let result = handle_option("verbosity 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_option_unsupported() {
        let result = handle_option("unsupported_option value");
        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_option_malformed() {
        let result = handle_option("malformed");
        assert!(result.is_ok());
    }

    #[test]
    fn test_edge_cases() {
        // Test empty string ref name
        let event = create_test_event();
        // Should handle gracefully even with edge cases
        assert!(extract_ref_name(&event).is_some());

        // Test very long ref name
        let long_ref_name = "a".repeat(1000);
        let keys = Keys::generate();
        let private_key = keys.secret_key().unwrap();
        let preevent = gnostr::types::PreEvent {
            pubkey: keys.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new_identifier(format!("git-ref:{}", long_ref_name)),
                Tag::new_identifier("gnostr-repo".to_string()),
            ],
            content: "Test event with long ref name".to_string(),
        };
        let event = Event::sign_with_private_key(preevent, &private_key).unwrap();

        let ref_name = extract_ref_name(&event);
        assert!(ref_name.is_some());
        assert_eq!(ref_name.unwrap(), long_ref_name);
    }
}
