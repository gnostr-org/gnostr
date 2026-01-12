use std::{
    collections::{BTreeMap, HashMap},
    io::{self, BufRead, BufReader},
    process,
};

use gnostr::types::{
    Client, Event, EventKind, Filter, Id, Keys, Options, PublicKey, RelayUrl, Tag, Unixtime,
};

fn main() -> io::Result<()> {
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
                handle_list(remote_name, url, &client)?;
                println!();
            }
            "list for-push" => {
                handle_list(remote_name, url, &client)?;
                println!();
            }
            cmd if cmd.starts_with("option ") => {
                handle_option(&cmd[7..])?;
            }
            cmd if cmd.starts_with("push ") => {
                handle_push(&cmd[5..], remote_name, url, &client)?;
            }
            cmd if cmd.starts_with("fetch ") => {
                handle_fetch(&cmd[6..], remote_name, url, &client)?;
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

fn handle_list(_remote_name: &str, url: &str, client: &Client) -> io::Result<()> {
    // Parse gnostr URL and list available refs
    if let Ok(repo_info) = parse_gnostr_url(url) {
        eprintln!("Listing refs for repository: {}", url);

        // Query nostr for existing refs
        match query_git_refs(client, &repo_info) {
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

fn handle_fetch(
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
            match fetch_git_data(client, &repo_info, ref_name) {
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
    relays: Vec<RelayUrl>,
    kind: Option<EventKind>,
    identifier: Option<String>,
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
    path: &str,
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
    path: &str,
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
                    refs.insert(ref_name, event.id.to_hex());
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
    filter.authors = Some(vec![repo_info.author]);

    // Filter by event kind
    filter.kinds = Some(vec![EventKind::TextNote]);

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
    client: &Client,
    repo_info: &GnostrRepoInfo,
    src: &str,
    dst: &str,
    ref_name: &str,
) -> Result<Id, Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let private_key = &keys.private_key;

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
    let event = Event::sign_with_private_key(
        gnostr::types::PreEvent::new(
            keys.public_key(),
            Unixtime::now(),
            EventKind::TextNote,
            tags,
            format!("Git push to {}: {} -> {}", ref_name, src, dst),
        ),
        private_key,
    )?;

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
    filter.authors = Some(vec![repo_info.author]);

    // Filter by event kind
    filter.kinds = Some(vec![EventKind::TextNote]);

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

    #[test]
    fn test_parse_gnostr_url() {
        // Test with naddr format
        let result = parse_gnostr_url(
            "gnostr://naddr1qqzynhx9qcrqcpzamhxue69uhkumttpwfjhxqgr0ys8qsqqqqqqpqqqqqyqumfnqv3xcm5v93qcrqcpzamhxue69uhkumttpwfjhxqgr0ys8qsqqqqqqpqqqqqyqumfnqv3xcm5v9",
        );
        assert!(result.is_ok());

        // Test with npub format
        let result = parse_gnostr_url("gnostr://npub1test");
        assert!(result.is_ok());

        // Test with invalid URL
        let result = parse_gnostr_url("not-gnostr://test");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_ref_filter() {
        let repo_info = GnostrRepoInfo {
            author: PublicKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000001",
                false,
            )
            .unwrap(),
            relays: vec![],
            kind: Some(EventKind::TextNote),
            identifier: None,
            url: "gnostr://test".to_string(),
        };

        let filter = create_ref_filter(&repo_info);
        assert!(filter.authors.is_some());
        assert!(filter.kinds.is_some());
        assert!(!filter.tags.is_empty());
    }
}
