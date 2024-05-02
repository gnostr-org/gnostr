use gnostr_types::{EventKind, Filter, PublicKey, PublicKeyHex};
use std::env;

fn main() {
    let mut args = env::args();
    let _ = args.next(); // program name
    let relay_url = match args.next() {
        Some(u) => u,
        None => panic!("Usage: fetch_by_kind_and_author <RelayURL> <KindNumber> <PubKey>"),
    };
    let kind_number = match args.next() {
        Some(num) => num.parse::<u32>().unwrap(),
        None => panic!("Usage: fetch_by_kind_and_author <RelayURL> <KindNumber> <PubKey>"),
    };
    let kind: EventKind = kind_number.into();
    let author_key = match args.next() {
        Some(key) => match PublicKey::try_from_bech32_string(&key, true) {
            Ok(key) => key,
            Err(_) => match PublicKey::try_from_hex_string(&key, true) {
                Ok(key) => key,
                Err(_) => panic!("Could not parse public key"),
            },
        },
        None => panic!("Usage: fetch_by_kind_and_author <RelayURL> <KindNumber> <PubKey>"),
    };
    let key: PublicKeyHex = author_key.into();
    let filter = Filter {
        kinds: vec![kind],
        authors: vec![key],
        ..Default::default()
    };

    for event in gnostr_bins::fetch_by_filter(&relay_url, filter) {
        gnostr_bins::print_event(&event);
    }
}
