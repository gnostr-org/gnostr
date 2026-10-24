//! Relay URL collection and NIP-34 patch event publishing helpers.

use std::time::Duration;

use url::Url;

use crate::nostr::{
    client::{local_relay_urls, Options},
    Client, Error, EventBuilder, EventKind, Id, Keys, Tag,
};

const ONLINE_RELAYS_GPS_CSV: &[u8] = include_bytes!("online_relays_gps.csv");

/// Install the `ring` rustls crypto provider.
///
/// This is a global one-time operation. It is safe to call multiple times;
/// subsequent calls are ignored.
pub fn install_rustls_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Return a deduplicated list of relay URLs to use for publishing.
///
/// The list starts with locally configured relays (from
/// [`local_relay_urls`]) and appends relays from the embedded
/// `online_relays_gps.csv` file, keeping only `wss://` URLs.
pub fn get_relay_urls() -> Vec<String> {
    let mut relays = local_relay_urls();

    let content = String::from_utf8_lossy(ONLINE_RELAYS_GPS_CSV);
    for relay in content.lines().skip(1).filter_map(|line| {
        let url_str = line.split(',').next()?.trim();
        if url_str.is_empty() {
            return None;
        }
        let full_url_str = if url_str.contains("://") {
            url_str.to_string()
        } else {
            format!("wss://{}", url_str)
        };
        match Url::parse(&full_url_str) {
            Ok(url) if url.scheme() == "wss" => Some(url.to_string()),
            _ => {
                eprintln!(
                    "Warning: Invalid or unsupported relay URL scheme: {}",
                    full_url_str
                );
                None
            }
        }
    }) {
        if !relays.contains(&relay) {
            relays.push(relay);
        }
    }

    relays
}

/// Build and publish a NIP-34 patch event.
///
/// The event is of kind [`EventKind::Patches`] and carries an `identifier`
/// (`d` tag) and a `commit` tag. If `build_manifest_event_id` is provided, an
/// `e` tag referencing it is added.
pub async fn publish_patch_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
) -> Result<Id, Error> {
    install_rustls_crypto_provider();

    let mut tags = vec![
        Tag::new_identifier(d_tag_value.to_string()),
        Tag::new_tag("commit", commit_id),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::new_event(*event_id, None, None));
    }

    let event = EventBuilder::new(EventKind::Patches, patch_content.to_string(), tags)
        .to_event(&keys.secret_key()?)?;

    log::info!(
        "Signed NIP-34 patch event: id={}, pubkey={}, kind={:?}, commit={}, identifier={}, relays={}",
        event.id,
        event.pubkey,
        event.kind,
        commit_id,
        d_tag_value,
        relay_urls.len()
    );
    if log::log_enabled!(log::Level::Debug) {
        let event_json = serde_json::to_string(&event).map_err(|err| Error::Custom(err.into()))?;
        log::debug!("NIP-34 patch event JSON: {}", event_json);
    }

    let mut client = Client::new(
        keys,
        Options::new().send_timeout(Some(Duration::from_secs(1))),
    );
    client.add_relays(relay_urls.to_vec()).await?;
    client.connect().await;

    client.send_event(event).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_relay_urls_includes_local_and_csv() {
        let urls = get_relay_urls();
        // local_relay_urls may be empty in CI, but the CSV is embedded.
        assert!(!urls.is_empty(), "expected at least one relay from CSV");
        for url in &urls {
            assert!(
                url.starts_with("wss://"),
                "only wss URLs should be kept: {url}"
            );
        }
    }

    #[test]
    fn test_get_relay_urls_deduplicates() {
        // This is a smoke test; real deduplication depends on local state.
        let urls = get_relay_urls();
        let deduped: std::collections::HashSet<_> = urls.iter().collect();
        assert_eq!(urls.len(), deduped.len());
    }
}
