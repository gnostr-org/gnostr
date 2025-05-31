use gnostr_types::Filter;
use std::env;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => panic!("Usage: fetch_by_kind_and_author <RelayURL> <FilterJSON>"),
    };
    let filter: Filter = match args.next() {
        Some(filter) => match serde_json::from_str(&filter) {
            Ok(f) => f,
            Err(e) => panic!("{}", e),
        },
        None => panic!("Usage: fetch_by_kind_and_author <RelayURL> <FilterJSON>"),
    };
    for event in gnostr::fetch_by_filter(&relay_url, filter) {
        gnostr::print_event(&event);
    }
}
