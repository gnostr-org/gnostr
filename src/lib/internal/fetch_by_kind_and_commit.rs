use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use gnostr_types::{EventKind, Filter, PublicKey, PublicKeyHex};
use log::debug;

pub fn build_kind_and_commit_filter(author_pubkey: &str, kind: &str) -> Result<Filter, String> {
    let author_key = match PublicKey::try_from_bech32_string(author_pubkey, true) {
        Ok(key) => key,
        Err(_) => match PublicKey::try_from_hex_string(author_pubkey, true) {
            Ok(key) => key,
            Err(_) => return Err(format!("Invalid author public key: {}", author_pubkey)),
        },
    };

    let kind_number = match kind.parse::<u32>() {
        Ok(num) => num,
        Err(_) => return Err(format!("Invalid event kind: {}", kind)),
    };

    let event_kind: EventKind = kind_number.into();

    let key_hex: PublicKeyHex = author_key.into();
    let filter = Filter {
        kinds: vec![event_kind],
        authors: vec![key_hex],
        ..Default::default()
    };
    Ok(filter)
}

// Original function for context, might be removed or adapted later
fn fetch_by_kind_and_commit(author_pubkey: &str, kind: &str, relay: &str) {
    let filter = match build_kind_and_commit_filter(author_pubkey, kind) {
        Ok(f) => f,
        Err(e) => {
            log::debug!("Error building filter: {}", e);
            return;
        }
    };

    let relay_url = if relay.is_empty() {
        BOOTSTRAP_RELAYS[2].clone()
    } else {
        relay.to_owned()
    };

    for event in crate::fetch_by_filter(&relay_url, filter) {
        crate::print_event(&event);
    }
}
