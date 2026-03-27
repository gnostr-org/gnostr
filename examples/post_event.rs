use std::{env, io::Read};

use gnostr_types::Event;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => panic!("Usage: fetch_by_id <RelayURL> < EventJSON"),
    };

    let mut s: String = String::new();
    std::io::stdin().read_to_string(&mut s).unwrap();

    let event: Event = serde_json::from_str(&s).unwrap();

    crate::post_event(&relay_url, event);
}
