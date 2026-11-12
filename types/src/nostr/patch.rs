//! Relay URL collection and NIP-34 patch event publishing helpers.

use std::time::Duration;

use url::Url;

use crate::nostr::{
    Client, Error, Event, EventBuilder, EventKind, Id, Keys, PublicKey, Tag,
    client::{Options, local_relay_urls},
};

const ONLINE_RELAYS_GPS_CSV: &[u8] = include_bytes!("online_relays_gps.csv");

/// The deterministic parent commit identity and its published patch event, if known.
#[derive(Clone, Copy, Debug)]
pub struct ParentCommitLink {
    /// Public key derived from the zero-padded parent Git commit hash.
    pub pubkey: PublicKey,
    /// Actual Nostr patch event ID, never a padded Git hash.
    pub event_id: Option<Id>,
}

impl ParentCommitLink {
    /// Event tags identifying this parent commit and its patch event.
    pub fn tags(&self) -> Vec<Tag> {
        let mut tags = Vec::new();
        append_parent_tags(&mut tags, Some(self));
        tags
    }
}

/// Preserve the Git worker/counter nonce without using NIP-13's `nonce` tag.
pub fn commit_nonce_tag(nonce: &str) -> Result<Tag, Error> {
    let (worker, counter) = nonce.split_once('-').ok_or(Error::InvalidOperation)?;
    if worker.len() < 2
        || !worker.bytes().all(|byte| byte.is_ascii_digit())
        || worker.parse::<u32>().is_err()
        || counter.len() != 8
        || !counter.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(Error::InvalidOperation);
    }
    Ok(Tag::new_tag("commit-nonce", nonce))
}

fn append_parent_tags(tags: &mut Vec<Tag>, parent: Option<&ParentCommitLink>) {
    if let Some(parent) = parent {
        tags.push(Tag::new_pubkey(parent.pubkey, None, None));
        if let Some(event_id) = parent.event_id {
            tags.push(Tag::new_event(event_id, None, Some("reply".to_string())));
        }
    }
}

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
    parent: Option<&ParentCommitLink>,
) -> Result<Event, Error> {
    let mut tags = Vec::new();
    append_parent_tags(&mut tags, parent);
    build_patch_event_with_tags(
        keys,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        &tags,
    )
}

/// Build a signed patch with additional tags, such as parent links and commit nonce.
pub fn build_patch_event_with_tags(
    keys: &Keys,
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
    extra_tags: &[Tag],
) -> Result<Event, Error> {
    build_patch_event_with_tags_and_pow(
        keys,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        extra_tags,
        0,
    )
}

/// Build a patch with additional signed tags and NIP-13 event-ID proof of work.
pub fn build_patch_event_with_tags_and_pow(
    keys: &Keys,
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
    extra_tags: &[Tag],
    difficulty: u8,
) -> Result<Event, Error> {
    build_patch_event_with_tags_and_mining(
        keys,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        extra_tags,
        &super::EventMining::Difficulty(difficulty),
    )
}

/// Build a tagged patch with an independent event-ID mining target.
pub fn build_patch_event_with_tags_and_mining(
    keys: &Keys,
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
    extra_tags: &[Tag],
    target: &super::EventMining,
) -> Result<Event, Error> {
    let mut tags = vec![
        Tag::new_identifier(d_tag_value.to_string()),
        Tag::new_tag("commit", commit_id),
        Tag::new_tag("t", "gnostr"),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::new_event(*event_id, None, None));
    }
    tags.extend_from_slice(extra_tags);

    EventBuilder::new(EventKind::Patches, patch_content.to_string(), tags)
        .to_mined_event(&keys.secret_key()?, target)
        .map_err(|err| Error::Custom(err.into()))
}

/// Build a kind-1 announcement with a `nostr:note` content reference and `e`/`t` tags.
pub fn build_patch_announcement(
    keys: &Keys,
    patch_event_id: Id,
    parent: Option<&ParentCommitLink>,
) -> Result<Event, Error> {
    let mut tags = Vec::new();
    append_parent_tags(&mut tags, parent);
    build_patch_announcement_with_tags(keys, patch_event_id, &tags)
}

/// Build a signed patch announcement with additional commit-proof tags.
pub fn build_patch_announcement_with_tags(
    keys: &Keys,
    patch_event_id: Id,
    extra_tags: &[Tag],
) -> Result<Event, Error> {
    build_patch_announcement_with_tags_and_pow(keys, patch_event_id, extra_tags, 0)
}

/// Build a patch announcement with additional tags and NIP-13 event-ID proof.
pub fn build_patch_announcement_with_tags_and_pow(
    keys: &Keys,
    patch_event_id: Id,
    extra_tags: &[Tag],
    difficulty: u8,
) -> Result<Event, Error> {
    build_patch_announcement_with_tags_and_mining(
        keys,
        patch_event_id,
        extra_tags,
        &super::EventMining::Difficulty(difficulty),
    )
}

/// Build an announcement referencing the final mined patch ID.
pub fn build_patch_announcement_with_tags_and_mining(
    keys: &Keys,
    patch_event_id: Id,
    extra_tags: &[Tag],
    target: &super::EventMining,
) -> Result<Event, Error> {
    let mut tags = vec![
        Tag::new_event(patch_event_id, None, Some("mention".to_string())),
        Tag::new_tag("t", "gnostr"),
    ];
    tags.extend_from_slice(extra_tags);
    EventBuilder::new(
        EventKind::TextNote,
        format!("nostr:{}", patch_event_id.as_bech32_string()),
        tags,
    )
    .to_mined_event(&keys.secret_key()?, target)
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
    parent: Option<&ParentCommitLink>,
) -> Result<Id, Error> {
    install_rustls_crypto_provider();
    let event = build_patch_event(
        keys,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        parent,
    )?;
    send_patch_event(keys, relay_urls, event, commit_id, d_tag_value).await
}

/// Publish a patch with extra signed tags without altering existing callers.
pub async fn publish_patch_event_with_tags(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
    extra_tags: &[Tag],
) -> Result<Id, Error> {
    publish_patch_event_with_tags_and_pow(
        keys,
        relay_urls,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        extra_tags,
        0,
    )
    .await
}

/// Publish a tagged patch after mining its event ID to the requested bit target.
#[allow(clippy::too_many_arguments)]
pub async fn publish_patch_event_with_tags_and_pow(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
    extra_tags: &[Tag],
    difficulty: u8,
) -> Result<Id, Error> {
    publish_patch_event_with_tags_and_mining(
        keys,
        relay_urls,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        extra_tags,
        &super::EventMining::Difficulty(difficulty),
    )
    .await
}

/// Mine a tagged patch to a bit or prefix target before publication.
#[allow(clippy::too_many_arguments)]
pub async fn publish_patch_event_with_tags_and_mining(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
    extra_tags: &[Tag],
    target: &super::EventMining,
) -> Result<Id, Error> {
    install_rustls_crypto_provider();
    let event = build_patch_event_with_tags_and_mining(
        keys,
        d_tag_value,
        commit_id,
        patch_content,
        build_manifest_event_id,
        extra_tags,
        target,
    )?;
    send_patch_event(keys, relay_urls, event, commit_id, d_tag_value).await
}

async fn send_patch_event(
    keys: &Keys,
    relay_urls: &[String],
    event: Event,
    commit_id: &str,
    d_tag_value: &str,
) -> Result<Id, Error> {
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
    use crate::nostr::nip13::NIP13Event;

    #[test]
    fn commit_nonce_is_signed_without_claiming_nip13_proof_of_work() {
        let keys = Keys::generate();
        let parent = ParentCommitLink {
            pubkey: Keys::generate().public_key(),
            event_id: Some(Id::mock()),
        };
        let nonce = commit_nonce_tag("01-00000122").unwrap();
        let mut tags = parent.tags();
        tags.push(nonce.clone());
        let patch =
            build_patch_event_with_tags(&keys, "repo", "commit", "patch", None, &tags).unwrap();
        let note = build_patch_announcement_with_tags(&keys, patch.id, &tags).unwrap();
        let profile = Client::new(&keys, Options::new())
            .build_metadata_event(
                &crate::nostr::Metadata::default(),
                vec![Tag::new_tag("t", "gnostr"), nonce.clone()],
            )
            .unwrap();
        for event in [&patch, &note, &profile] {
            event.verify(None).unwrap();
            assert!(event.tags.contains(&nonce));
            assert!(event.tags.contains(&Tag::new_tag("t", "gnostr")));
            assert!(event.nonce_data().is_none());
        }
        assert!(patch.tags.contains(&Tag::new_event(
            parent.event_id.unwrap(),
            None,
            Some("reply".to_string()),
        )));
        assert_eq!(
            commit_nonce_tag("100-00000122").unwrap().value(),
            "100-00000122"
        );
        for invalid in [
            "1-00000122",
            "01-0000012g",
            "01-000001222",
            "01-122",
            "4294967296-00000122",
        ] {
            assert!(commit_nonce_tag(invalid).is_err());
        }
    }

    #[test]
    fn patch_announcement_references_patch_and_uses_same_signer() {
        let keys = Keys::generate();
        let patch = build_patch_event(&keys, "repo", "commit", "patch", None, None).unwrap();
        let note = build_patch_announcement(&keys, patch.id, None).unwrap();
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
                Tag::new_event(patch.id, None, Some("mention".to_string())),
                Tag::new_tag("t", "gnostr"),
            ]
        );
    }

    #[test]
    fn patch_has_gnostr_hashtag_and_preserves_manifest_reference() {
        let keys = Keys::generate();
        let manifest = Id::mock();
        let event =
            build_patch_event(&keys, "repo", "commit", "patch", Some(&manifest), None).unwrap();
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
    fn patch_and_announcement_link_to_parent_event_and_identity() {
        let keys = Keys::generate();
        let parent = ParentCommitLink {
            pubkey: Keys::generate().public_key(),
            event_id: Some(Id::mock()),
        };
        let manifest = Id::mock();
        let patch = build_patch_event(
            &keys,
            "repo",
            "commit",
            "patch",
            Some(&manifest),
            Some(&parent),
        )
        .unwrap();
        assert!(patch.tags.contains(&Tag::new_event(manifest, None, None)));
        let note = build_patch_announcement(&keys, patch.id, Some(&parent)).unwrap();
        for event in [&patch, &note] {
            event.verify(None).unwrap();
            assert!(
                event
                    .tags
                    .contains(&Tag::new_pubkey(parent.pubkey, None, None))
            );
            assert!(event.tags.contains(&Tag::new_event(
                parent.event_id.unwrap(),
                None,
                Some("reply".to_string())
            )));
            assert!(event.tags.contains(&Tag::new_tag("t", "gnostr")));
        }
        assert!(
            note.tags
                .contains(&Tag::new_event(patch.id, None, Some("mention".to_string())))
        );
    }

    #[test]
    fn undiscovered_parent_keeps_identity_without_a_fabricated_event_reference() {
        let keys = Keys::generate();
        let parent = ParentCommitLink {
            pubkey: Keys::generate().public_key(),
            event_id: None,
        };
        let patch =
            build_patch_event(&keys, "repo", "commit", "patch", None, Some(&parent)).unwrap();
        patch.verify(None).unwrap();
        assert!(
            patch
                .tags
                .contains(&Tag::new_pubkey(parent.pubkey, None, None))
        );
        assert!(!patch.tags.iter().any(|tag| tag.tagname() == "e"));
        let note = build_patch_announcement(&keys, patch.id, Some(&parent)).unwrap();
        note.verify(None).unwrap();
        assert!(
            note.tags
                .contains(&Tag::new_pubkey(parent.pubkey, None, None))
        );
        assert!(
            note.tags
                .contains(&Tag::new_event(patch.id, None, Some("mention".to_string()),))
        );
        assert!(!note.tags.iter().any(|tag| tag.marker() == "reply"));
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
