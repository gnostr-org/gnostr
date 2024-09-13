use gnostr_types::{Event, Filter, IdHex};
use http::Uri;

mod internal;
use internal::*;

mod reflog_simple;
use crate::reflog_simple::{
    pwd, ref_hash_list, ref_hash_list_padded, ref_hash_list_w_commit_message,
};

pub mod weeble;
pub use weeble::weeble;

pub mod wobble;
pub use wobble::wobble;

pub mod blockheight;
pub use blockheight::blockheight;

pub mod gitminer;
pub use gitminer::*;
pub mod repo;
pub use repo::*;
pub mod worker;
pub use worker::*;

/// REF: https://api.nostr.watch
/// nostr.watch API Docs
///
/// Uptime absolutely not guaranteed
///
/// Endpoints
///
/// Supported Methods: GET
///
/// Online Relays: https://api.nostr.watch/v1/online
/// Public Relays: https://api.nostr.watch/v1/public
/// Pay to Relays: https://api.nostr.watch/v1/paid
/// Offline Relays: https://api.nostr.watch/v1/offline
/// Relays by supported NIP: https://api.nostr.watch/v1/nip/X Use NIP ids without leading zeros - for example: https://api.nostr.watch/v1/nip/1
pub mod relays;
use futures::executor::block_on;
pub use relays::{
    relays, relays_by_nip, relays_offline, relays_online, relays_paid, relays_public,
};
pub mod watch_list;
pub use watch_list::*;
pub async fn watch_list() -> Result<Vec<String>, url::ParseError> {
    let future = watch_list(); // Nothing is printed
    let list = block_on(future);
    Ok(list.unwrap())
}

pub fn strip_trailing_nl(input: &mut String) {
    let new_len = input
        .char_indices()
        .rev()
        .find(|(_, c)| !matches!(c, '\n' | '\r'))
        .map_or(0, |(i, _)| i + 1);
    if new_len != input.len() {
        input.truncate(new_len);
    }
}

pub fn get_pwd() -> Result<String, &'static str> {
    let mut no_nl = pwd().unwrap().to_string();
    no_nl.retain(|c| c != '\n');
    return Ok(format!("{  }", no_nl));
}

/// get_relays_by_nip
/// pub fn get_relays_by_nip(nip: &str) -> Result<String, &'static str>
pub fn get_relays_by_nip(nip: &str) -> Result<String, &'static str> {
    let _relays_no_nl = relays_by_nip(nip).unwrap().to_string();

    Ok(relays_by_nip(nip).unwrap().to_string())
}
/// get_relays <https://api.nostr.watch>
/// pub fn get_relays() -> Result<String, &'static str>
pub fn get_relays() -> Result<String, &'static str> {
    let _relays_no_nl = relays().unwrap().to_string();

    Ok(format!("{}", relays().unwrap().to_string()))
}
/// get_relays_online <https://api.nostr.watch>
/// pub fn get_relays_online() -> Result<String, &'static str>
pub fn get_relays_online() -> Result<String, &'static str> {
    let _relays_no_nl = relays_public().unwrap().to_string();

    Ok(format!("{}", relays_online().unwrap().to_string()))
}
/// get_relays_public <https://api.nostr.watch>
/// pub fn get_relays_public() -> Result<String, &'static str>
pub fn get_relays_public() -> Result<String, &'static str> {
    let _relays_no_nl = relays_public().unwrap().to_string();

    Ok(format!("{}", relays_public().unwrap().to_string()))
}
/// get_relays_paid <https://api.nostr.watch>
/// pub fn get_relays_paid() -> Result<String, &'static str>
pub fn get_relays_paid() -> Result<String, &'static str> {
    let _relays_no_nl = relays_paid().unwrap().to_string();

    Ok(format!("{}", relays_public().unwrap().to_string()))
}
/// get_relays_offline <https://api.nostr.watch>
/// pub fn get_relays_offline() -> Result<String, &'static str>
pub fn get_relays_offline() -> Result<String, &'static str> {
    let _relays_no_nl = relays_offline().unwrap().to_string();

    Ok(format!("{}", relays_public().unwrap().to_string()))
}

/// pub fn get_weeble() -> Result<String, &'static str>
pub fn get_weeble() -> Result<String, &'static str> {
    let _weeble_no_nl = weeble().unwrap().to_string();

    Ok(format!("{}", weeble().unwrap().to_string()))
}
/// pub fn get_wobble() -> Result<String, &'static str>
pub fn get_wobble() -> Result<String, &'static str> {
    let _wobble_no_nl = wobble().unwrap().to_string();

    Ok(format!("{}", wobble().unwrap().to_string()))
}
/// pub fn get_blockheight() -> Result<String, &'static str>
pub fn get_blockheight() -> Result<String, &'static str> {
    let _blockheight_no_nl = blockheight().unwrap().to_string();

    Ok(format!("{}", blockheight().unwrap().to_string()))
}

pub fn hash_list_w_commit_message() {
    let _ = ref_hash_list_w_commit_message();
}

pub fn hash_list() {
    let _ = ref_hash_list();
}

pub fn hash_list_padded() {
    let _ = ref_hash_list_padded();
}

pub fn url_to_host_and_uri(url: &str) -> (String, Uri) {
    let uri: http::Uri = url.parse::<http::Uri>().expect("Could not parse url");
    let authority = uri.authority().expect("Has no hostname").as_str();
    let host = authority
        .find('@')
        .map(|idx| authority.split_at(idx + 1).1)
        .unwrap_or_else(|| authority);
    if host.is_empty() {
        panic!("URL has empty hostname");
    }
    (host.to_owned(), uri)
}

pub fn fetch_by_filter(url: &str, filter: Filter) -> Vec<Event> {
    let (host, uri) = url_to_host_and_uri(url);
    let wire = filters_to_wire(vec![filter]);
    fetch(host, uri, wire)
}

pub fn fetch_by_id(url: &str, id: IdHex) -> Option<Event> {
    let mut filter = Filter::new();
    filter.add_id(&id);
    let events = fetch_by_filter(url, filter);
    if events.is_empty() {
        None
    } else {
        Some(events[0].clone())
    }
}

pub fn post_event(url: &str, event: Event) {
    let (host, uri) = url_to_host_and_uri(url);
    let wire = event_to_wire(event);
    post(host, uri, wire)
}
use gnostr_types::EventV2;
pub fn post_event_v2(url: &str, event: EventV2) {
    let (host, uri) = url_to_host_and_uri(url);
    let wire = event_to_wire(event);
    post(host, uri, wire)
}

pub fn print_event(event: &Event) {
    println!(
        "{}",
        serde_json::to_string(event).expect("Cannot serialize event to JSON")
    );
}
use std::error::Error;
use std::process;

use sha256::digest;

pub struct Config {
    pub query: String,
}

pub fn print_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() == 1 {
            println!("{}", digest("".to_string()));
            process::exit(0);
        }

        let query = args[1].clone();
        Ok(Config { query })
    }
}
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    println!("{}", digest(config.query));
    Ok(())
}
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<&'a str> {
    let mut results = Vec::new();
    for line in contents.lines() {
        // do something with line
        println!("{}", line);
        if line.contains(query) {
            // do something with line
            let val = digest(query);
            println!("{}", val);
            results.push(line);
        }
    }
    results
}

/// cargo +nightly t -- --nocapture
///
/// cargo    test -- --nocapture
///
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_relays_paid() {
        get_relays_paid();
    }
    #[test]
    fn test_get_relays_public() {
        get_relays_public();
    }
    #[test]
    fn test_get_relays_online() {
        get_relays_online();
    }
    #[test]
    fn test_get_relays_by_nip() {
        get_relays_by_nip(&"0");
    }
    #[test]
    fn test_get_relays_offline() {
        get_relays_offline();
    }
    #[test]
    fn test_get_relays() {
        get_relays();
    }

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
    #[test]
    fn one_query() {
        let query = digest("");
        let contents = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        assert_eq!(
            vec!["e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"],
            search(&query, contents)
        );
    }
    #[test]
    #[should_panic]
    fn panic_query() {
        let query = digest("");
        let contents = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ";

        assert_eq!(
            vec!["e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"],
            search(&query, contents)
        );
    }
}

//#[cfg(debug_assertions)]
//let cloned_query = query.clone();
//#[cfg(debug_assertions)]
//println!("{:?}", print_type_of(&query));
//#[cfg(debug_assertions)]
//println!("{:?}", print_type_of(&cloned_query));
//println!("{:?}", query);
//#[cfg(debug_assertions)]
//let s = &"hello world".to_string();
//#[cfg(debug_assertions)]
//let cloned_s = s.clone();
//#[cfg(debug_assertions)]
//println!("{:?}", print_type_of(&s));
//#[cfg(debug_assertions)]
//println!("{:?}", print_type_of(&cloned_s));
