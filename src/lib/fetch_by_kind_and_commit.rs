use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::{EventKind, Filter, PublicKey, PublicKeyHex};
use log::debug;
fn fetch_by_kind_and_commit(author_pubkey: &str, kind: &str, relay: &str) {
    let author_key = match author_pubkey {
        Some(key) => match PublicKey::try_from_bech32_string(&key, true) {
            Ok(key) => key,
            Err(_) => match PublicKey::try_from_hex_string(&key, true) {
                Ok(key) => key,
                Err(_) => {
                    debug!("gnostr_fetch_by_kind_and_commit failed! invalid pubkey");
                }
            },
        },
        None => {
            debug!("gnostr_fetch_by_kind_and_commit failed! invalid pubkey");
        }
    };

    let kind_number = match kind {
        Some(num) => num.parse::<u32>().unwrap(),
        None => {
            debug!("gnostr_fetch_by_kind_and_commit failed! invalid kind");
        }
    };

    let relay_url = match relay {
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
