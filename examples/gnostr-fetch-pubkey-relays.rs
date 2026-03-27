use std::env;

use gnostr_types::{EventKind, Filter, PublicKeyHex};
use gnostr::{get_relays_public, fetch_by_filter, print_event};

fn main() {
    println!("{:#}", crate::get_relays_public().unwrap());
    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => panic!("Usage: fetch_relay_list <RelayURL> <PubKeyHex>"),
    };
    let pubkeyhex = match args.next() {
        Some(id) => id,
        None => panic!("Usage: fetch_metadata <RelayURL> <PubKeyHex>"),
    };

    let pkh = PublicKeyHex::try_from_str(&pubkeyhex).unwrap();

    let mut filter = Filter::new();
    filter.add_author(&pkh);
    filter.add_event_kind(EventKind::RelayList);
    let events = crate::fetch_by_filter(&relay_url, filter);
    if !events.is_empty() {
        crate::print_event(&events[0]);
    } else {
        println!("Not found");
    }
}
