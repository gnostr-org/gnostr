use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::Filter;
use std::env;

//fetch_by_filter "{\"REQ\":[\"kind\",\"1\"]}" wss://relay.damus.io | gnostr-xq
fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let filter: Filter = match args.next() {
        Some(filter) => match serde_json::from_str(&filter) {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        },
        None => panic!("Usage: fetch_by_kind_and_author <FilterJSON> <RelayURL>\n       fetch_by_filter \'{{\"REQ\":[\"kind\",\"1\"]}}\' wss://relay.damus.io | gnostr-xq"),
    };
    let relay_url = match args.next() {
        Some(u) => u,
        None => BOOTSTRAP_RELAYS[2].clone(),//panic!("Usage: fetch_by_kind_and_author <FilterJSON> <RelayURL>"),
    };
    for event in gnostr::fetch_by_filter(&relay_url, filter) {
        gnostr::print_event(&event);
    }
}
