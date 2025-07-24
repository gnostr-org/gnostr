use gnostr_types::Event;
use std::env;
use std::io::Read;

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

    gnostr::post_event(&relay_url, event);
}
