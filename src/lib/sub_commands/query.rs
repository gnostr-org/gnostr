use anyhow::{anyhow, bail, Context};
use clap::Args;
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_query::ConfigBuilder;
use log::{debug, error, info, warn};
use serde_json::{json, to_string};
use url::Url;

/// Arguments for the 'query' subcommand.
#[derive(Debug, Args, Clone)]
pub struct QuerySubCommand {
    /// Filter by author public keys (comma-separated).
    #[arg(long)]
    pub authors: Option<String>,
    /// Filter by event IDs (comma-separated).
    #[arg(long)]
    pub ids: Option<String>,
    /// Maximum number of events to return.
    #[arg(long, default_value = "1")]
    pub limit: Option<i32>,
    /// Generic filters in the format '#<tag> <value>'. Expects two space-separated values.
    /// Example: --generic "#t" "general,nostr"
    #[arg(num_args = 2, value_delimiter = ' ', long)]
    pub generic: Option<Vec<String>>,
    /// Filter by hashtags (comma-separated).
    #[arg(long)]
    pub hashtag: Option<String>,
    /// Filter by mentions (public keys, comma-separated).
    #[arg(long)]
    pub mentions: Option<String>,
    /// Filter by referenced event IDs (comma-separated).
    #[arg(long)]
    pub references: Option<String>,
    /// Filter by event kinds (comma-separated integers).
    #[arg(long)]
    pub kinds: Option<String>,
    /// Search for text within event content. Can take multiple values, but only the first is used.
    /// Example: --search "keyword1,keyword2"
    #[arg(num_args = 1.., long)]
    pub search: Option<Vec<String>>,
    /// Specify a relay URL to connect to.
    #[arg(long, default_value = "wss://relay.damus.io")]
    pub relay: Option<String>,
}

/// Handles the 'query' subcommand functionality.
/// It takes the parsed command-line arguments and executes the query.
pub async fn launch(args: &QuerySubCommand) -> anyhow::Result<()> {
    debug!("Launching query subcommand with args: {:?}", args);
    debug!("Launching query subcommand with args: {:?}", args);

    let (filt, limit_check) = build_filter_map(args)?;

    // ConfigBuilder usage from original main.
    // These values might need to be configurable or passed from the main app.
    // For now, using defaults similar to the original bin.
    debug!("Building gnostr_query config.");
    debug!("Building gnostr_query config.");
    let config = ConfigBuilder::new()
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
    debug!("Constructed query string: {}", query_string);
    debug!("Constructed query string: {}", query_string);

    let relays = if let Some(relay_str) = &args.relay {
        debug!("Using specified relay: {}", relay_str);
        debug!("Using specified relay: {}", relay_str);
        vec![Url::parse(relay_str)?]
    } else {
        debug!("Using bootstrap relays.");
        debug!("Using bootstrap relays.");
        BOOTSTRAP_RELAYS
            .iter()
            .filter_map(|s| Url::parse(s).ok())
            .collect()
    };

    debug!("Sending query to relays: {:?}", relays);
    debug!("Sending query to relays: {:?}", relays);
    // Convert the error from gnostr_query::send to anyhow::Error before propagating
    let vec_result = gnostr_query::send(query_string.clone(), relays, Some(limit_check)).await
        .map_err(|e| {
            error!("Failed to send query: {}", e);
            anyhow!("Failed to send query: {}", e)
        })?;
    debug!("Received query result.");
    debug!("Received query result.");

    let mut json_result: Vec<String> = vec![];
    debug!("json_result={:?}", json_result);
    debug!("json_result.len()={:?}", json_result.len());
    for element in vec_result {
        debug!("element={}", element);
        json_result.push(element);
    }

    // In a library function, we should just print and return Ok(()).
    // The exit code logic is usually handled by the main binary.
    for element in json_result {
        print!("{}", element); // output to terminal
    }

    debug!("Query subcommand finished successfully.");
    Ok(())
}

fn build_filter_map(args: &QuerySubCommand) -> anyhow::Result<(serde_json::Map<String, serde_json::Value>, i32)> {
    let mut filt = serde_json::Map::new();
    let mut limit_check: i32 = 0;

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

    if let Some(limit) = args.limit {
        debug!("Applying limit filter: {}", limit);
        filt.insert("limit".to_string(), json!(limit));
        limit_check = limit;
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
        if !search_vec.is_empty() {
            let search_string = "search".to_string();
            // The original bin code only used the first element of search if multiple were provided.
            // Let's stick to that behavior.
            let val = search_vec[0].clone();
            debug!("Applying search filter: {}", val);
            filt.insert(search_string, json!(val));
        }
    }
    Ok((filt, limit_check))
}

    #[cfg(test)]
    mod tests {
        use super::*;
        use clap::{Parser, Subcommand};
        use serde_json::json;
        use gnostr_crawler::processor::BOOTSTRAP_RELAYS;

        #[derive(Parser)]
        #[clap(name = "gnostr", about = "A test CLI for gnostr")]
        struct Cli {
            #[clap(subcommand)]
            command: Commands,
        }

        #[derive(Subcommand)]
        enum Commands {
            Query(QuerySubCommand),
        }

        // Helper function to create QuerySubCommand from args
        fn create_query_subcommand(args: &[&str]) -> QuerySubCommand {
            let full_args = std::iter::once("gnostr").chain(std::iter::once("query")).chain(args.iter().cloned());
            let cli = Cli::parse_from(full_args);
            match cli.command {
                Commands::Query(query_subcommand) => query_subcommand,
            }
        }

        // Helper function to launch a query with a specific relay
        async fn launch_with_relay(args: &QuerySubCommand, relay_url: &str) -> anyhow::Result<()> {
            let mut modified_args = args.clone();
            modified_args.relay = Some(relay_url.to_string());
            launch(&modified_args).await
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

            assert_eq!(
                filt.get("authors").unwrap(),
                &json!(["pubkey1", "pubkey2"])
            );
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

            assert_eq!(
                filt.get("#t").unwrap(),
                &json!("general,nostr")
            );
            Ok(())
        }

        #[test]
        fn test_build_filter_map_with_hashtag() -> anyhow::Result<()> {
            let args = create_query_subcommand(&["--hashtag", "rust,programming"]);
            let (filt, _) = build_filter_map(&args)?;

            assert_eq!(
                filt.get("#t").unwrap(),
                &json!(["rust", "programming"])
            );
            Ok(())
        }

        #[test]
        fn test_build_filter_map_with_mentions() -> anyhow::Result<()> {
            let args = create_query_subcommand(&["--mentions", "mention1,mention2"]);
            let (filt, _) = build_filter_map(&args)?;

            assert_eq!(
                filt.get("#p").unwrap(),
                &json!(["mention1", "mention2"])
            );
            Ok(())
        }

        #[test]
        fn test_build_filter_map_with_references() -> anyhow::Result<()> {
            let args = create_query_subcommand(&["--references", "ref1,ref2"]);
            let (filt, _) = build_filter_map(&args)?;

            assert_eq!(
                filt.get("#e").unwrap(),
                &json!(["ref1", "ref2"])
            );
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
        fn test_build_filter_map_with_single_kind() -> anyhow::Result<()> {
            let args = create_query_subcommand(&["--kinds", "1630"]);
            let (filt, _) = build_filter_map(&args)?;

            assert_eq!(filt.get("kinds").unwrap(), &json!([1630]));
            Ok(())
        }

        #[test]
        fn test_build_filter_map_with_multiple_specific_kinds() -> anyhow::Result<()> {
            let args = create_query_subcommand(&["--kinds", "1630,1632,1621,30618,1633,1631,1617,30617"]);
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

            // The current implementation allows duplicates, which is acceptable for a filter list.
            assert_eq!(filt.get("kinds").unwrap(), &json!([1, 2, 1]));
            Ok(())
        }

        #[tokio::test]
        async fn test_launch_no_panic_with_all_bootstrap_relays() {
            let base_args = create_query_subcommand(&[]);
            for relay_url in BOOTSTRAP_RELAYS.iter() {
                debug!("Testing launch with relay: {}", relay_url);
                let result = launch_with_relay(&base_args, relay_url).await;
                assert!(result.is_ok(), "Launch failed for relay {}: {:?}", relay_url, result.err());
            }
        }
    }
