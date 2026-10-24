//! Relay URL collection and NIP-34 patch event publishing helpers.

use std::time::Duration;

use url::Url;

use crate::nostr::{
    Client, Error, Event, EventBuilder, EventKind, Id, Keys, Tag,
    client::{Options, local_relay_urls},
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

/// Build a signed NIP-34 patch with identifier, commit, and `gnostr` hashtag tags.
/// An optional manifest event is referenced with an `e` tag.
pub fn build_patch_event(
    keys: &Keys,
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
) -> Result<Event, Error> {
    let mut tags = vec![
        Tag::new_identifier(d_tag_value.to_string()),
        Tag::new_tag("commit", commit_id),
        Tag::new_tag("t", "gnostr"),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::new_event(*event_id, None, None));
    }

    EventBuilder::new(EventKind::Patches, patch_content.to_string(), tags)
        .to_event(&keys.secret_key()?)
        .map_err(|err| Error::Custom(err.into()))
}

/// Build a kind-1 announcement with a `nostr:note` content reference and `e`/`t` tags.
pub fn build_patch_announcement(keys: &Keys, patch_event_id: Id) -> Result<Event, Error> {
    EventBuilder::new(
        EventKind::TextNote,
        format!("nostr:{}", patch_event_id.as_bech32_string()),
        vec![
            Tag::new_event(patch_event_id, None, None),
            Tag::new_tag("t", "gnostr"),
        ],
    )
    .to_event(&keys.secret_key()?)
    .map_err(|err| Error::Custom(err.into()))
}

/// Build and publish a tagged NIP-34 patch event.
pub async fn publish_patch_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
) -> Result<Id, Error> {
    install_rustls_crypto_provider();
    let event = build_patch_event(
        keys,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
    )?;

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
    fn patch_announcement_references_patch_and_uses_same_signer() {
        let keys = Keys::generate();
        let patch = build_patch_event(&keys, "repo", "commit", "patch", None).unwrap();
        let note = build_patch_announcement(&keys, patch.id).unwrap();
        note.verify(None).unwrap();
        assert_eq!(note.kind, EventKind::TextNote);
        assert_eq!(note.pubkey, patch.pubkey);
        assert_eq!(
            note.content,
            format!("nostr:{}", patch.id.as_bech32_string())
        );
        assert_eq!(
            note.tags,
            vec![
                Tag::new_event(patch.id, None, None),
                Tag::new_tag("t", "gnostr"),
            ]
        );
    }

    #[test]
    fn patch_has_gnostr_hashtag_and_preserves_manifest_reference() {
        let keys = Keys::generate();
        let manifest = Id::mock();
        let event = build_patch_event(&keys, "repo", "commit", "patch", Some(&manifest)).unwrap();
        event.verify(None).unwrap();
        assert_eq!(event.pubkey, keys.public_key());
        assert_eq!(event.kind, EventKind::Patches);
        assert!(event.tags.contains(&Tag::new_tag("t", "gnostr")));
        assert!(event.tags.contains(&Tag::new_tag("commit", "commit")));
        assert!(
            event
                .tags
                .contains(&Tag::new_identifier("repo".to_string()))
        );
        assert!(event.tags.contains(&Tag::new_event(manifest, None, None)));
        assert_eq!(event.content, "patch");
    }

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
