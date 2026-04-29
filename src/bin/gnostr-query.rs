use gnostr::crawler::processor::BOOTSTRAP_RELAYS;
use gnostr::query::cli::cli;
use gnostr::query::ConfigBuilder;
use gnostr::sub_commands::query::search_relays_for_nip50;
use serde_json::{json, to_string};
use url::Url;

/// Usage
/// nip-0034 kinds
/// gnostr-query -k 1630,1632,1621,30618,1633,1631,1617,30617
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    //debug!("debug! main!");
    //println!("println! main!");
    let matches = cli().await?;

    let mut filt = serde_json::Map::new();
    let _ = serde_json::Map::new();

    if let Some(authors) = matches.get_one::<String>("authors") {
        filt.insert(
            "authors".to_string(),
            json!(authors.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(ids) = matches.get_one::<String>("ids") {
        filt.insert(
            "ids".to_string(),
            json!(ids.split(',').collect::<Vec<&str>>()),
        );
    }

    let mut limit_check: i32 = 0;
    if let Some(limit) = matches.get_one::<i32>("limit") {
        // ["EOSE","gnostr-query"] counts as a message!      + 1
        filt.insert("limit".to_string(), json!(limit.clone() /*+ 1*/));
        limit_check = *limit;
    }

    if let Some(generic) = matches.get_many::<String>("generic") {
        let generic_vec: Vec<&String> = generic.collect();
        if generic_vec.len() == 2 {
            let tag = format!("#{}", generic_vec[0]);
            let val = generic_vec[1].split(',').collect::<String>();
            filt.insert(tag, json!(val));
        }
    }

    if let Some(hashtag) = matches.get_one::<String>("hashtag") {
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(mentions) = matches.get_one::<String>("mentions") {
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(references) = matches.get_one::<String>("references") {
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
    }

    if let Some(kinds) = matches.get_one::<String>("kinds") {
        if let Ok(kind_ints) = kinds
            .split(',')
            .map(|s| s.parse::<i64>())
            .collect::<Result<Vec<i64>, _>>()
        {
            filt.insert("kinds".to_string(), json!(kind_ints));
        } else {
            eprintln!("Error parsing kinds. Ensure they are integers.");
            std::process::exit(1);
        }
    }
    let search_term = matches
        .get_many::<String>("search")
        .and_then(|search| search.into_iter().next().cloned())
        .filter(|search| !search.is_empty());

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
        .build()?;

    //debug!("debug config=\n{config:?}");
    //println!("println config=\n{config:?}");
    let q = json!(["REQ", "gnostr-query", filt]);
    let query_string = to_string(&q)?;
    //debug!("debug:query_string:\n{:?}", query_string);
    //println!("println :query_string:\n{:?}", query_string);

    let relay_args: Vec<String> = matches
        .get_many::<String>("relay")
        .map(|values| values.cloned().collect())
        .unwrap_or_default();
    let relays = if search_term.is_some() {
        search_relays_for_nip50()?
    } else if relay_args.is_empty() {
        BOOTSTRAP_RELAYS
            .iter()
            .filter_map(|s| Url::parse(s).ok())
            .collect()
    } else {
        relay_args
            .iter()
            .flat_map(|relay_arg| relay_arg.split(','))
            .map(str::trim)
            .filter(|relay| !relay.is_empty())
            .map(Url::parse)
            .collect::<Result<Vec<_>, _>>()?
    };

    let vec_result = gnostr::query::send(query_string.clone(), relays, Some(limit_check)).await?;

    //trace
    //debug!("vec_result:\n{:?}", vec_result.clone());
    //for s in vec_result {println!("s={}", s)};
    //println!("vec_result:\n{:?}", vec_result);

    let mut json_result: Vec<String> = vec![];
    for element in vec_result {
        //log::debug!("element=\n{}", element);
        //println!("element=\n{}", element);
        json_result.push(element);
    }

    for element in json_result {
        print!("{}", element);
    }
    Ok(())
}
