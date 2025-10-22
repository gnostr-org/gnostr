use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_query::cli::cli;
use gnostr_query::ConfigBuilder;
use log::{debug, error};
use serde_json::{json, to_string};
use url::Url;

/// Usage
/// nip-0034 kinds
/// gnostr-query -k 1630,1632,1621,30618,1633,1631,1617,30617
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    debug!("gnostr-query binary started.");
    let matches = cli().await?;

    let mut filt = serde_json::Map::new();
    let _ = serde_json::Map::new();

    if let Some(authors) = matches.get_one::<String>("authors") {
        debug!("Applying authors filter: {}", authors);
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(ids) = matches.get_one::<String>("ids") {
        debug!("Applying IDs filter: {}", ids);
        filt.insert(
            "ids".to_string(),
            json!(ids.split(',').collect::<Vec<&str>>()),
        );
    }

    let mut limit_check: i32 = 0;
    if let Some(limit) = matches.get_one::<i32>("limit") {
        debug!("Applying limit filter: {}", limit);
        // ["EOSE","gnostr-query"] counts as a message!      + 1
        filt.insert("limit".to_string(), json!(limit.clone() /*+ 1*/));
        limit_check = *limit;
    }

    if let Some(generic) = matches.get_many::<String>("generic") {
        let generic_vec: Vec<&String> = generic.collect();
        if generic_vec.len() == 2 {
            let tag = format!("#{}", generic_vec[0]);
            let val = generic_vec[1].split(',').collect::<String>();
            debug!("Applying generic filter: tag={} val={}", tag, val);
            filt.insert(tag, json!(val));
        }
    }

    if let Some(hashtag) = matches.get_one::<String>("hashtag") {
        debug!("Applying hashtag filter: {}", hashtag);
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(mentions) = matches.get_one::<String>("mentions") {
        debug!("Applying mentions filter: {}", mentions);
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(references) = matches.get_one::<String>("references") {
        debug!("Applying references filter: {}", references);
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(kinds) = matches.get_one::<String>("kinds") {
        debug!("Applying kinds filter: {}", kinds);
        if let Ok(kind_ints) = kinds
            .split(',')
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<i64>, _>>()
        {
            filt.insert("kinds".to_string(), json!(kind_ints));
        } else {
            error!("Error parsing kinds: {}. Ensure they are integers.", kinds);
            eprintln!("Error parsing kinds. Ensure they are integers.");
            std::process::exit(1);
        }
    }
    //["REQ", "", { "search": "orange" }, { "kinds": [1, 2], "search": "purple" }]
    if let Some(search) = matches.get_many::<String>("search") {
        let search_vec: Vec<&String> = search.collect();
        //if search_vec.len() == 2 {
        let search_string = "search".to_string();
        let val = search_vec[0].split(',').collect::<String>();
        debug!("Applying search filter: {}", val);
        filt.insert(search_string, json!(val));
        //}
    }

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
            e
        })?;

    debug!("Config built: {:?}", config);
    let q = json!(["REQ", "gnostr-query", filt]);
    let query_string = to_string(&q)?;
    debug!("Constructed query string: {}", query_string);

    let relays = if let Some(relay_str) = matches.get_one::<String>("relay") {
        debug!("Using specified relay: {}", relay_str);
        vec![Url::parse(relay_str)?]
    } else {
        debug!("Using bootstrap relays.");
        BOOTSTRAP_RELAYS
            .iter()
            .filter_map(|s| Url::parse(s).ok())
            .collect()
    };

    debug!("Sending query to relays: {:?}", relays);
    let vec_result = gnostr_query::send(query_string.clone(), relays, Some(limit_check)).await
        .map_err(|e| {
            error!("Failed to send query: {}", e);
            e
        })?;
    debug!("Received query result.");

    //trace
    debug!("vec_result:\n{:?}", vec_result.clone());
	//for s in vec_result {println!("s={}", s)};
    //println!("vec_result:\n{:?}", vec_result);

    let mut json_result: Vec<String> = vec![];
    for element in vec_result {
        debug!("Processing result element: {}", element);
        json_result.push(element);
    }

    if matches.get_many::<String>("search").is_some() {
        for element in json_result {
            print!("{}", element);
        }
        debug!("Exiting after search results.");
        std::process::exit(0);
    } else {
        for element in json_result {
            print!("{}\n", element);
        }
    }
    debug!("gnostr-query binary finished successfully.");
    Ok(())
}
