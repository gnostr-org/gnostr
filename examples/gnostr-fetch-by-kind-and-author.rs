use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::{EventKind, Filter, PublicKey, PublicKeyHex};
use std::env;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let author_key = match args.next() {
        Some(key) => match PublicKey::try_from_bech32_string(&key, true) {
            Ok(key) => key,
            Err(_) => match PublicKey::try_from_hex_string(&key, true) {
                Ok(key) => key,
                Err(_) => {
                    print!("Usage: fetch-by-kind-and-author <PubKey> <KindNumber> <RelayURL>");
                    std::process::exit(0);
                }
            },
        },
        None => {
            print!("Usage: fetch-by-kind-and-author <PubKey>");
            std::process::exit(0);
        }
    };

    let kind_number = match args.next() {
        Some(num) => num.parse::<u32>().unwrap(),
        None => {
            print!("Usage: fetch-by-kind-and-author <PubKey> <KindNumber> <RelayURL>");
            std::process::exit(0);
        }
    };

    let relay_url = match args.next() {
        Some(u) => u,
        None => BOOTSTRAP_RELAYS[2].clone(),
    };

    let kind: EventKind = kind_number.into();

    let key: PublicKeyHex = author_key.into();
    let filter = Filter {
        kinds: vec![kind],
        authors: vec![key],
        ..Default::default()
    };

    for event in gnostr::fetch_by_filter(&relay_url, filter) {
        gnostr::print_event(&event);
    }
}
