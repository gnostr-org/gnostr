use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::{EventKind, Filter, PublicKeyHex};
use std::env;
fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let pubkeyhex = match args.next() {
        Some(id) => id,
        None => panic!("Usage: fetch_metadata <PubKeyHex> <RelayURL>"),
    };
    let relay_url = match args.next() {
        Some(u) => u,
        None => BOOTSTRAP_RELAYS[0].clone(),
    };

    let pkh = PublicKeyHex::try_from_str(&pubkeyhex).unwrap();

    let mut filter = Filter::new();
    filter.add_author(&pkh);
    filter.add_event_kind(EventKind::Metadata);
    let events = gnostr::fetch_by_filter(&relay_url, filter);
    if !events.is_empty() {
        gnostr::print_event(&events[0]);
    } else {
        println!("Not found");
    }
}
