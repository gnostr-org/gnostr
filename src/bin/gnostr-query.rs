use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_query::cli::cli;
use gnostr_query::ConfigBuilder;
use log::debug;
use serde_json::{json, to_string};
use url::Url;

use serde_json::{Map, Value};

fn process_json_map_by_value(json_map: Map<String, Value>) {
    for (key, value) in json_map {
        println!("{{\"{}\": {}}}", key, value.to_string());
    }
}

/// Usage
/// nip-0034 kinds
/// gnostr-query -k 1630,1632,1621,30618,1633,1631,1617,30617
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        //process_json_map_by_value(filt.clone());
    }

    if let Some(generic) = matches.get_many::<String>("generic") {
        let generic_vec: Vec<&String> = generic.collect();
        if generic_vec.len() == 2 {
            let tag = format!("#{}", generic_vec[0]);
            let val = generic_vec[1].split(',').collect::<String>();
            filt.insert(tag, json!(val));
            //process_json_map_by_value(filt.clone());
        }
    }

    if let Some(hashtag) = matches.get_one::<String>("hashtag") {
        filt.insert(
            "#t".to_string(),
            json!(hashtag.split(',').collect::<Vec<&str>>()),
        );
        //process_json_map_by_value(filt.clone());
    }

    if let Some(mentions) = matches.get_one::<String>("mentions") {
        filt.insert(
            "#p".to_string(),
            json!(mentions.split(',').collect::<Vec<&str>>()),
        );
        //process_json_map_by_value(filt.clone());
    }

    if let Some(references) = matches.get_one::<String>("references") {
        filt.insert(
            "#e".to_string(),
            json!(references.split(',').collect::<Vec<&str>>()),
        );
        //process_json_map_by_value(filt.clone());
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
        //process_json_map_by_value(filt.clone());
    }
    //["REQ", "", { "search": "orange" }, { "kinds": [1, 2], "search": "purple" }]
    if let Some(search) = matches.get_many::<String>("search") {
        let search_vec: Vec<&String> = search.collect();
        //if search_vec.len() == 2 {
        let search_string = "search".to_string();
        let val = search_vec[0].split(',').collect::<String>();
        filt.insert(search_string, json!(val));
        //process_json_map_by_value(filt.clone());
        //}
    }
    process_json_map_by_value(filt.clone());
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
        .build()?;

    log::debug!("config=\n{config:?}");
    let q = json!(["REQ", "gnostr-query", filt]);
    //println!("{}", q.clone());
    let query_string = to_string(&q)?;
    println!("{{ \"query_string\" : {} }}", query_string.to_string());

    let relay_url_str = matches.get_one::<String>("relay").unwrap();
    let relay_url = Url::parse(relay_url_str)?;

    if matches.get_many::<String>("search").is_some() {
        //println!("search is some {:?}", matches.get_many::<String>("search"));
        let vec_result = gnostr_query::send(
            query_string.clone(),
            //vec![Url::parse(&BOOTSTRAP_RELAYS.to_vec()[0]).expect("")],
            vec![Url::parse(&relay_url_str).expect("")],
            Some(limit_check),
        )
        .await;

        //trace
        //println!("144:{{ \"vec_result\" : \" {:?} \"}}", vec_result);

        let mut json_result: Vec<String> = vec![];
        for element in vec_result.unwrap() {
            println!("{{\"element\":{}}}", element);
            json_result.push(element);
        }

        for element in json_result {
            //println!("{{\"element\": {} }}", element);
        }
        std::process::exit(0);
    } else {
        let vec_result = gnostr_query::send(
            query_string.clone(),
            //vec![Url::parse(&BOOTSTRAP_RELAYS.to_vec()[0]).expect("")],
            vec![Url::parse(&relay_url_str).expect("")],
            Some(limit_check),
        )
        .await;

        let mut json_result: Vec<String> = vec![];
        for element in vec_result.unwrap() {
            //println!("{{ \"element\": \"{}\"}}", element);
            json_result.push(element);
        }

        for element in json_result {
            print!("{}", element);
        }
    }
    Ok(())
}
