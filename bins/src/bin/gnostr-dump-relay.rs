// TEMPORARILY
#![allow(clippy::uninlined_format_args)]

use std::env;

use gnostr_types::Filter;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => panic!("Usage: dump_relay <RelayURL>"),
    };

    let filter = Filter::new();
    let events = gnostr_bins::fetch_by_filter(&relay_url, filter);
    for event in events {
        gnostr_bins::print_event(&event);
    }
}
