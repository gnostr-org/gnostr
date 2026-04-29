use crate::query::ConfigBuilder;
use anyhow::{anyhow, bail};
use log::{debug, error};
use serde_json::{json, to_string, Value};
use url::Url;

pub use crate::query::cli::QuerySubCommand;

/// Handles the 'query' subcommand functionality.
/// It takes the parsed command-line arguments and executes the query.
pub async fn launch(args: &QuerySubCommand) -> anyhow::Result<()> {
    let (filt, limit_check) = build_filter_map(args)?;
    let search_term = search_term(args);
    let _config = ConfigBuilder::new()
        .host("localhost")
        .port(8080)
        .use_tls(true)
        .retries(5)
        .authors("")
        .ids("")
        .limit(limit_check)
        .generic("", "")
        .hashtag("")
        .mentions("")
        .references("")
        .kinds("")
        .search("", "")
        .build()
        .map_err(|e| {
            error!("Failed to build config: {}", e);
            anyhow!("Failed to build config: {}", e)
        })?;

    let q = json!(["REQ", "gnostr-query", filt]);
    let query_string = to_string(&q)?;
    debug!("{}", query_string);

    let relays = if search_term.is_some() {
        search_relays_for_nip50()?
    } else if args.relay.is_empty() {
        debug!("Using bootstrap relays.");
        crate::crawler::bootstrap_relays()
            .iter()
            .filter_map(|s| Url::parse(s).ok())
            .collect()
    } else {
        let relays = parse_relays(&args.relay)?;
        debug!("Using specified relays: {:?}", relays);
        relays
    };
    if relays.is_empty() {
        return Err(anyhow!("No valid relay URLs available"));
    }

    let vec_result = crate::query::send(query_string.clone(), relays, Some(limit_check))
        .await
        .map_err(|e| {
            error!("Failed to send query: {}", e);
            anyhow!("Failed to send query: {}", e)
        })?;
    debug!("Received query result.");

    let mut json_result: Vec<String> = vec![];
    debug!("json_result={:?}", json_result);
    debug!("json_result.len()={:?}", json_result.len());
    for element in vec_result {
        debug!("element={}", element);
        json_result.push(element);
    }

    for element in json_result {
        print!("{}", element);
    }

    Ok(())
}

fn build_filter_map(
    args: &QuerySubCommand,
) -> anyhow::Result<(serde_json::Map<String, serde_json::Value>, i32)> {
    let mut filt = serde_json::Map::new();
    let limit_check = args.limit.unwrap_or(1);
    filt.insert("limit".to_string(), json!(limit_check));

    if let Some(authors) = &args.authors {
        debug!("Applying authors filter: {}", authors);
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(ids) = &args.ids {
        debug!("Applying IDs filter: {}", ids);
        filt.insert(
            "ids".to_string(),
            json!(ids.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(generic_vec) = &args.generic {
        if generic_vec.len() == 2 {
            let tag = format!("#{}", generic_vec[0]);
            let val = generic_vec[1].clone();
            debug!("Applying generic filter: tag={} val={}", tag, val);
            filt.insert(tag, json!(val));
        }
    }

    if let Some(hashtag) = &args.hashtag {
        debug!("Applying hashtag filter: {}", hashtag);
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(mentions) = &args.mentions {
        debug!("Applying mentions filter: {}", mentions);
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(references) = &args.references {
        debug!("Applying references filter: {}", references);
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(kinds) = &args.kinds {
        debug!("Applying kinds filter: {}", kinds);
        if let Ok(kind_ints) = kinds
            .split(',')
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<i64>, _>>()
        {
            filt.insert("kinds".to_string(), json!(kind_ints));
        } else {
            error!("Error parsing kinds: {}. Ensure they are integers.", kinds);
            bail!("Error parsing kinds. Ensure they are integers.");
        }
    }

    if let Some(search_vec) = &args.search {
        if let Some(search) = search_vec.first() {
            if !search.is_empty() {
                debug!("Applying search filter: {}", search);
                filt.insert("search".to_string(), json!(search));
            }
        }
    }
    Ok((filt, limit_check))
}

fn search_term(args: &QuerySubCommand) -> Option<String> {
    args.search
        .as_ref()
        .and_then(|search_vec| search_vec.first())
        .cloned()
        .filter(|search| !search.is_empty())
}

pub fn search_relays_for_nip50() -> anyhow::Result<Vec<Url>> {
    let raw = crate::get_relays_by_nip("50").map_err(anyhow::Error::msg)?;
    parse_relay_urls(&raw)
}

fn parse_relay_urls(raw: &str) -> anyhow::Result<Vec<Url>> {
    let value: Value = serde_json::from_str(raw)?;
    match value {
        Value::Array(items) => items.into_iter().map(parse_relay_url_value).collect(),
        Value::Object(mut obj) => {
            if let Some(items) = obj.remove("relays").or_else(|| obj.remove("data")).or_else(|| obj.remove("items")) {
                match items {
                    Value::Array(items) => items.into_iter().map(parse_relay_url_value).collect(),
                    other => parse_relay_url_value(other).map(|url| vec![url]),
                }
            } else if let Some(url) = obj.remove("url") {
                parse_relay_url_value(url).map(|url| vec![url])
            } else {
                Err(anyhow!("Unexpected relay list shape: {}", raw))
            }
        }
        _ => Err(anyhow!("Unexpected relay list shape: {}", raw)),
    }
}

fn parse_relay_url_value(value: Value) -> anyhow::Result<Url> {
    match value {
        Value::String(url) => Ok(Url::parse(&url)?),
        Value::Object(mut obj) => {
            if let Some(Value::String(url)) = obj.remove("url") {
                Ok(Url::parse(&url)?)
            } else {
                Err(anyhow!("Unexpected relay item shape"))
            }
        }
        _ => Err(anyhow!("Unexpected relay item shape")),
    }
}

fn parse_relays(relay_args: &[String]) -> anyhow::Result<Vec<Url>> {
    let mut relays = Vec::new();
    for relay_arg in relay_args {
        for relay in relay_arg.split(',').map(str::trim).filter(|relay| !relay.is_empty()) {
            relays.push(Url::parse(relay)?);
        }
    }
    Ok(relays)
}

#[cfg(test)]
mod tests {
    use clap::{Parser, Subcommand};
    use serde_json::json;
    use std::sync::Once;

    use super::*;

    static INIT: Once = Once::new();

    fn setup_rustls() {
        INIT.call_once(|| {
            let _ = rustls::crypto::ring::default_provider().install_default();
        });
    }

    #[derive(Parser)]
    #[command(name = "gnostr", about = "A test CLI for gnostr")]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(Subcommand)]
    enum Commands {
        Query(QuerySubCommand),
    }

    // Helper function to create QuerySubCommand from args
    fn create_query_subcommand(args: &[&str]) -> QuerySubCommand {
        let full_args = std::iter::once("gnostr")
            .chain(std::iter::once("query"))
            .chain(args.iter().cloned());
        let cli = Cli::parse_from(full_args);
        match cli.command {
            Commands::Query(query_subcommand) => query_subcommand,
        }
    }

    // Helper function to launch a query with a specific relay
    async fn launch_with_relay(args: &QuerySubCommand, relay_url: &str) -> anyhow::Result<()> {
        let mut modified_args = args.clone();
        modified_args.relay = vec![relay_url.to_string()];
        launch(&modified_args).await
    }

    #[test]
    fn test_parse_relay_flags_and_csv() -> anyhow::Result<()> {
        setup_rustls();
        let args = create_query_subcommand(&[
            "-r",
            "wss://relay.damus.io",
            "-r",
            "wss://blossom.gnostr.cloud",
            "-r",
            "wss://nos.lol,wss://relay.nos.social",
        ]);

        assert_eq!(
            args.relay,
            vec![
                "wss://relay.damus.io".to_string(),
                "wss://blossom.gnostr.cloud".to_string(),
                "wss://nos.lol".to_string(),
                "wss://relay.nos.social".to_string(),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_build_filter_map_default_limit() -> anyhow::Result<()> {
        let args = create_query_subcommand(&[]);
        let (filt, limit_check) = build_filter_map(&args)?;

        assert_eq!(limit_check, 1); // Default limit
        assert_eq!(filt.get("limit").unwrap(), &json!(1));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_authors() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--authors", "pubkey1,pubkey2"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("authors").unwrap(), &json!(["pubkey1", "pubkey2"]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_ids() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--ids", "id1,id2"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("ids").unwrap(), &json!(["id1", "id2"]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_custom_limit() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--limit", "10"]);
        let (filt, limit_check) = build_filter_map(&args)?;

        assert_eq!(limit_check, 10);
        assert_eq!(filt.get("limit").unwrap(), &json!(10));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_generic() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--generic", "t", "general,nostr"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("#t").unwrap(), &json!("general,nostr"));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_hashtag() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--hashtag", "rust,programming"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("#t").unwrap(), &json!(["rust", "programming"]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_mentions() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--mentions", "mention1,mention2"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("#p").unwrap(), &json!(["mention1", "mention2"]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_references() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--references", "ref1,ref2"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("#e").unwrap(), &json!(["ref1", "ref2"]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_kinds() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--kinds", "1,2,3"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("kinds").unwrap(), &json!([1, 2, 3]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_invalid_kinds() {
        let args = create_query_subcommand(&["--kinds", "1,abc,3"]);
        let result = build_filter_map(&args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Error parsing kinds. Ensure they are integers."
        );
    }

    #[test]
    fn test_build_filter_map_with_search() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--search", "keyword1,keyword2"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("search").unwrap(), &json!("keyword1,keyword2"));
        Ok(())
    }

    #[test]
    fn test_parse_relay_urls_array() -> anyhow::Result<()> {
        let relays = parse_relay_urls(r#"["wss://relay.example","wss://relay2.example"]"#)?;

        assert_eq!(
            relays,
            vec![
                Url::parse("wss://relay.example")?,
                Url::parse("wss://relay2.example")?
            ]
        );
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_single_kind() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--kinds", "1630"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("kinds").unwrap(), &json!([1630]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_multiple_specific_kinds() -> anyhow::Result<()> {
        let args =
            create_query_subcommand(&["--kinds", "1630,1632,1621,30618,1633,1631,1617,30617"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(
            filt.get("kinds").unwrap(),
            &json!([1630, 1632, 1621, 30618, 1633, 1631, 1617, 30617])
        );
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_kinds_and_authors() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--kinds", "1,2", "--authors", "pubkeyA"]);
        let (filt, _) = build_filter_map(&args)?;

        assert_eq!(filt.get("kinds").unwrap(), &json!([1, 2]));
        assert_eq!(filt.get("authors").unwrap(), &json!(["pubkeyA"]));
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_empty_kinds() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--kinds", ""]);
        let result = build_filter_map(&args);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Error parsing kinds. Ensure they are integers."
        );
        Ok(())
    }

    #[test]
    fn test_build_filter_map_with_duplicate_kinds() -> anyhow::Result<()> {
        let args = create_query_subcommand(&["--kinds", "1,2,1"]);
        let (filt, _) = build_filter_map(&args)?;

        // The current implementation allows duplicates, which is acceptable for a
        // filter list.
        assert_eq!(filt.get("kinds").unwrap(), &json!([1, 2, 1]));
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn test_launch_no_panic_with_all_bootstrap_relays() {
        setup_rustls();
        let base_args = create_query_subcommand(&[]);
        let bootstrap_relays = crate::crawler::bootstrap_relays();
        for relay_url in bootstrap_relays
            .iter()
            .filter(|&r| r != &bootstrap_relays[0] && r != &bootstrap_relays[2])
        {
            debug!("Testing launch with relay: {}", relay_url);
            if let Err(err) = launch_with_relay(&base_args, relay_url).await {
                debug!("Launch returned an error for relay {}: {:?}", relay_url, err);
            }
        }
    }

    #[test]
    fn test_build_filter_map_with_nostr_url_bech32_conversion() -> anyhow::Result<()> {
        // Test equivalent to: gnostr query --authors $(gnostr bech32-to-any
        // nostr://npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy/
        // nostr.cro.social/gnostr --raw) The nostr URL contains the same npub,
        // which should convert to the same hex pubkey
        let expected_hex_pubkey =
            "86a254249e6321386a1dcca7356a9a0792e21e8cc5a2b490266532d44a48d72c";

        // This simulates what command substitution would produce
        let args = create_query_subcommand(&["--authors", expected_hex_pubkey]);
        let (filt, limit_check) = build_filter_map(&args)?;

        assert_eq!(limit_check, 1); // Default limit
        assert_eq!(filt.get("authors").unwrap(), &json!([expected_hex_pubkey]));
        assert_eq!(filt.get("limit").unwrap(), &json!(1));
        Ok(())
    }

    #[test]
    #[cfg(feature = "long_tests")]
    fn test_bech32_to_any_with_nostr_url() -> anyhow::Result<()> {
        use std::process::Command;

        // Test the bech32-to-any command with nostr URL directly
        let nostr_url = "nostr://npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy/nostr.cro.social/gnostr";

        // Set environment to use gnostr binary for CliTester
        unsafe { std::env::set_var("CARGO_BIN_EXE_ngit", "gnostr") };

        let bech32_output = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "gnostr",
                "--",
                "bech32-to-any",
                nostr_url,
                "--raw",
            ])
            .output()
            .expect("Failed to run bech32-to-any command");

        assert!(
            bech32_output.status.success(),
            "bech32-to-any should succeed"
        );

        let hex_pubkey = String::from_utf8(bech32_output.stdout)
            .expect("Output should be valid UTF-8")
            .trim()
            .to_string();

        // Verify the hex pubkey matches expected value
        assert_eq!(
            hex_pubkey,
            "86a254249e6321386a1dcca7356a9a0792e21e8cc5a2b490266532d44a48d72c"
        );

        // Now verify this hex pubkey works in query filter map
        let args = create_query_subcommand(&["--authors", &hex_pubkey]);
        let (filt, limit_check) = build_filter_map(&args)?;

        assert_eq!(limit_check, 1); // Default limit
        assert_eq!(filt.get("authors").unwrap(), &json!([hex_pubkey]));
        assert_eq!(filt.get("limit").unwrap(), &json!(1));
        Ok(())
    }
}
