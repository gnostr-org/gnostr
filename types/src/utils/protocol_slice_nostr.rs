//! Nostr carrier for the perfect_ip slice data plane.
//!
//! The point of perfect_ip is slice data as JSON, and nostr events are a JSON
//! transport: each slice becomes one signed event that any relay can carry
//! and any receiver can verify against the manifest.
//!
//! Envelope (a signed kind-9261 regular event; see [`PERFECT_IP_NOSTR_KIND`]):
//!
//! - content: `serde_json` of the slice — v1 [`ProtocolSlice`] or v2
//!   [`ProtocolSliceV2`], exactly the bytes the libp2p wire compat matrix
//!   pins in `p2p/tests/fractal_wire_compat.rs`.
//! - `["perfect_ip", "<version>"]` with version `"1"` or `"2"`. Receivers
//!   MUST check this tag before parsing content. Unlike raw serde — which
//!   silently drops v2's `len` when a v2 slice is read as v1 (proven in the
//!   wire matrix) — the nostr envelope is strict in both directions and
//!   refuses the wrong version with [`NostrCarrierError::WrongVersion`].
//! - `["d", "<root_id>", "<slice_id>"]` — the parameterized-replaceable
//!   slot, one per slice, so re-announcing a slice refreshes the same relay
//!   slot instead of accumulating duplicates.
//! - `["t", "perfect_ip"]` — discovery hashtag.
//!
//! Verification stays where it belongs: [`crate::perfect_ip_v2::IntegrityManagerV2::record_slice`]
//! hash-checks the decoded slice against the manifest, so a tampered event
//! content is rejected on record, not trusted on parse.
//!
//! This module is pure `gnostr-types`: it references `crate::nostr` and the
//! sibling slice modules only, adding no crate dependencies in either
//! direction.

use std::fmt;

use super::protocol_slice::ProtocolSlice;
use super::protocol_slice_v2::ProtocolSliceV2;
use crate::nostr::{Event, EventBuilder, EventKind, PrivateKey, Tag};

/// Regular event kind carrying perfect_ip slices.
///
/// This MUST be a regular (non-replaceable) kind: the vendored gnostr-relay
/// keys replaceable and parameterized-replaceable storage by `(pubkey,
/// kind)` only, ignoring the `d` tag — with a 30xxx kind every slice event
/// after the first answers `OK false "replaced: have newer event"` and only
/// one slice would ever be stored. Kind 9261 is unassigned in the NIP
/// registry (9000-9999 regular range, only 9041/9372/9735 are taken).
pub const PERFECT_IP_NOSTR_KIND: u32 = 9261;
/// Tag marking the payload version: `["perfect_ip", "1"|"2"]`.
pub const PERFECT_IP_TAG: &str = "perfect_ip";
/// Discovery hashtag value.
pub const PERFECT_IP_TOPIC: &str = "perfect_ip";
/// d-tag third field marking the manifest event for a root.
pub const PERFECT_IP_MANIFEST_MARK: &str = "manifest";
/// Version marker for v1 payloads.
pub const PERFECT_IP_VERSION_V1: &str = "1";
/// Version marker for v2 payloads.
pub const PERFECT_IP_VERSION_V2: &str = "2";

/// Why an event is not a usable perfect_ip slice for this receiver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NostrCarrierError {
    /// Event kind is not [`PERFECT_IP_NOSTR_KIND`].
    WrongKind { expected: u32, found: u32 },
    /// No `["perfect_ip", ...]` tag present.
    MissingVersionTag,
    /// Version tag disagrees with the parser's expected version.
    WrongVersion {
        expected: &'static str,
        found: String,
    },
    /// No `["d", ...]` tag present.
    MissingDTag,
    /// The event's `d` tag names a different root than expected.
    WrongRoot { expected: String, found: String },
    /// The event is a manifest, not a slice.
    NotASlice,
    /// The event is a slice, not a manifest.
    NotAManifest,
    /// Event content is not JSON for the expected slice shape.
    ContentJson(String),
    /// The event could not be built or signed.
    EventBuild(String),
}

impl fmt::Display for NostrCarrierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongKind { expected, found } => {
                write!(f, "wrong event kind: expected {expected}, found {found}")
            }
            Self::MissingVersionTag => write!(f, "missing [\"perfect_ip\", ...] version tag"),
            Self::WrongVersion { expected, found } => {
                write!(
                    f,
                    "wrong perfect_ip version: expected {expected}, found {found}"
                )
            }
            Self::MissingDTag => write!(f, "missing [\"d\", ...] tag"),
            Self::WrongRoot { expected, found } => {
                write!(f, "wrong root: expected {expected}, found {found}")
            }
            Self::NotASlice => write!(f, "event is a manifest, not a slice"),
            Self::NotAManifest => write!(f, "event is a slice, not a manifest"),
            Self::ContentJson(message) => write!(f, "content json error: {message}"),
            Self::EventBuild(message) => write!(f, "event build error: {message}"),
        }
    }
}

impl std::error::Error for NostrCarrierError {}

fn envelope_tags(root_id: &str, slice_id: &str, version: &str) -> Vec<Tag> {
    vec![
        Tag::new(&["d", root_id, slice_id]),
        Tag::new(&[PERFECT_IP_TAG, version]),
        Tag::new(&["t", PERFECT_IP_TOPIC]),
    ]
}

fn find_tag<'a>(event: &'a Event, name: &str) -> Option<&'a Tag> {
    event.tags.iter().find(|tag| tag.get_index(0) == name)
}

fn check_envelope(
    event: &Event,
    root_id: &str,
    expected_version: &'static str,
) -> Result<(), NostrCarrierError> {
    if event.kind != EventKind::from(PERFECT_IP_NOSTR_KIND) {
        return Err(NostrCarrierError::WrongKind {
            expected: PERFECT_IP_NOSTR_KIND,
            found: event.kind.into(),
        });
    }
    let version = find_tag(event, PERFECT_IP_TAG).ok_or(NostrCarrierError::MissingVersionTag)?;
    let version = version.get_index(1);
    if version != expected_version {
        return Err(NostrCarrierError::WrongVersion {
            expected: expected_version,
            found: version.to_string(),
        });
    }
    let d_tag = find_tag(event, "d").ok_or(NostrCarrierError::MissingDTag)?;
    if d_tag.get_index(1) != root_id {
        return Err(NostrCarrierError::WrongRoot {
            expected: root_id.to_string(),
            found: d_tag.get_index(1).to_string(),
        });
    }
    Ok(())
}

/// Wrap a v2 slice in a signed kind-9261 event.
pub fn slice_event_v2(
    keys: &PrivateKey,
    root_id: &str,
    slice: &ProtocolSliceV2,
) -> Result<Event, NostrCarrierError> {
    let content = serde_json::to_string(slice)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))?;
    EventBuilder::new(
        EventKind::from(PERFECT_IP_NOSTR_KIND),
        content,
        envelope_tags(root_id, &slice.id, PERFECT_IP_VERSION_V2),
    )
    .to_event(keys)
    .map_err(|error| NostrCarrierError::EventBuild(error.to_string()))
}

/// Wrap a v1 slice in a signed kind-9261 event.
pub fn slice_event_v1(
    keys: &PrivateKey,
    root_id: &str,
    slice: &ProtocolSlice,
) -> Result<Event, NostrCarrierError> {
    let content = serde_json::to_string(slice)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))?;
    EventBuilder::new(
        EventKind::from(PERFECT_IP_NOSTR_KIND),
        content,
        envelope_tags(root_id, &slice.id, PERFECT_IP_VERSION_V1),
    )
    .to_event(keys)
    .map_err(|error| NostrCarrierError::EventBuild(error.to_string()))
}

/// The d-tag third field of an event, discriminating slice from manifest.
fn d_tag_kind(event: &Event) -> &str {
    find_tag(event, "d")
        .map(|tag| tag.get_index(2))
        .unwrap_or("")
}

fn manifest_tags(root_id: &str, version: &str) -> Vec<Tag> {
    vec![
        Tag::new(&["d", root_id, PERFECT_IP_MANIFEST_MARK]),
        Tag::new(&[PERFECT_IP_TAG, version]),
        Tag::new(&["t", PERFECT_IP_TOPIC]),
    ]
}

/// Wrap the v2 manifest in a signed kind-9261 event.
///
/// The manifest is what makes the nostr transport self-sufficient: a
/// receiver that only knows the root id can fetch it, then hash-verify and
/// reassemble every slice event it collects.
pub fn manifest_event_v2(
    keys: &PrivateKey,
    root_id: &str,
    manifest: &[crate::perfect_ip_v2::ManifestEntry],
) -> Result<Event, NostrCarrierError> {
    let content = serde_json::to_string(manifest)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))?;
    EventBuilder::new(
        EventKind::from(PERFECT_IP_NOSTR_KIND),
        content,
        manifest_tags(root_id, PERFECT_IP_VERSION_V2),
    )
    .to_event(keys)
    .map_err(|error| NostrCarrierError::EventBuild(error.to_string()))
}

/// Wrap the v1 manifest (a bare id list) in a signed kind-9261 event.
pub fn manifest_event_v1(
    keys: &PrivateKey,
    root_id: &str,
    manifest_ids: &[String],
) -> Result<Event, NostrCarrierError> {
    let content = serde_json::to_string(manifest_ids)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))?;
    EventBuilder::new(
        EventKind::from(PERFECT_IP_NOSTR_KIND),
        content,
        manifest_tags(root_id, PERFECT_IP_VERSION_V1),
    )
    .to_event(keys)
    .map_err(|error| NostrCarrierError::EventBuild(error.to_string()))
}

/// Decode and strictly validate a v2 manifest event.
pub fn event_to_manifest_v2(
    event: &Event,
    root_id: &str,
) -> Result<Vec<crate::perfect_ip_v2::ManifestEntry>, NostrCarrierError> {
    check_envelope(event, root_id, PERFECT_IP_VERSION_V2)?;
    if d_tag_kind(event) != PERFECT_IP_MANIFEST_MARK {
        return Err(NostrCarrierError::NotAManifest);
    }
    serde_json::from_str(&event.content)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))
}

/// Decode and strictly validate a v1 manifest event (a bare id list).
pub fn event_to_manifest_v1(
    event: &Event,
    root_id: &str,
) -> Result<Vec<String>, NostrCarrierError> {
    check_envelope(event, root_id, PERFECT_IP_VERSION_V1)?;
    if d_tag_kind(event) != PERFECT_IP_MANIFEST_MARK {
        return Err(NostrCarrierError::NotAManifest);
    }
    serde_json::from_str(&event.content)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))
}

/// Decode and strictly validate a v2 slice event.
pub fn event_to_slice_v2(
    event: &Event,
    root_id: &str,
) -> Result<ProtocolSliceV2, NostrCarrierError> {
    check_envelope(event, root_id, PERFECT_IP_VERSION_V2)?;
    if d_tag_kind(event) == PERFECT_IP_MANIFEST_MARK {
        return Err(NostrCarrierError::NotASlice);
    }
    serde_json::from_str(&event.content)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))
}

/// Decode and strictly validate a v1 slice event.
pub fn event_to_slice_v1(event: &Event, root_id: &str) -> Result<ProtocolSlice, NostrCarrierError> {
    check_envelope(event, root_id, PERFECT_IP_VERSION_V1)?;
    if d_tag_kind(event) == PERFECT_IP_MANIFEST_MARK {
        return Err(NostrCarrierError::NotASlice);
    }
    serde_json::from_str(&event.content)
        .map_err(|error| NostrCarrierError::ContentJson(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::perfect_ip_v2::{IntegrityManagerV2, packetize_v2};

    /// Well-known weak test key (same one the nostr event_builder tests use).
    fn test_key() -> PrivateKey {
        let keys = crate::nostr::Keys::parse(
            "473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813".to_string(),
        )
        .expect("parse test key");
        keys.secret_key().expect("secret key")
    }

    fn canonical_v2() -> ProtocolSliceV2 {
        ProtocolSliceV2 {
            id: "R".to_string(),
            header: crate::perfect_ip_v2::HeaderV2 {
                seq_num: 1,
                total_packets: 3,
                len: 2,
            },
            data: vec![0xDE, 0xAD],
            is_parity: false,
        }
    }

    fn canonical_v1() -> ProtocolSlice {
        ProtocolSlice {
            id: "R".to_string(),
            header: crate::perfect_ip::Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD],
            is_parity: false,
        }
    }

    #[test]
    fn v2_slice_round_trips_through_event() {
        let keys = test_key();
        let slice = canonical_v2();
        let event = slice_event_v2(&keys, "R", &slice).expect("build event");

        assert_eq!(event.kind, EventKind::from(PERFECT_IP_NOSTR_KIND));
        // Content is exactly the pinned v2 JSON wire bytes.
        assert_eq!(
            event.content,
            "{\"id\":\"R\",\"header\":{\"seq_num\":1,\"total_packets\":3,\"len\":2},\
             \"data\":[222,173],\"is_parity\":false}"
                .replace(' ', "")
        );
        // Envelope tags, exactly.
        let tags: Vec<Vec<String>> = event
            .tags
            .iter()
            .map(|tag| tag.clone().into_inner())
            .collect();
        assert_eq!(
            tags,
            vec![
                vec!["d", "R", "R"],
                vec![PERFECT_IP_TAG, "2"],
                vec!["t", PERFECT_IP_TOPIC],
            ]
        );

        let decoded = event_to_slice_v2(&event, "R").expect("decode");
        assert_eq!(decoded, slice);
    }

    #[test]
    fn v1_slice_round_trips_through_event() {
        let keys = test_key();
        let slice = canonical_v1();
        let event = slice_event_v1(&keys, "R", &slice).expect("build event");

        assert_eq!(
            event.content,
            "{\"id\":\"R\",\"header\":{\"seq_num\":1,\"total_packets\":3},\
             \"data\":[222,173],\"is_parity\":false}"
                .replace(' ', "")
        );
        let decoded = event_to_slice_v1(&event, "R").expect("decode");
        assert_eq!(decoded, slice);
    }

    #[test]
    fn full_packet_tree_round_trips_and_reconstructs() {
        let keys = test_key();
        let payload: Vec<u8> = (0..3000).map(|i| (i % 251) as u8).collect();
        let result = packetize_v2("ROOT".to_string(), payload.clone());

        // Every slice becomes an event; every event decodes and verifies.
        let mut manager = IntegrityManagerV2::new(result.manifest.clone());
        for slice in &result.batch.packets {
            let event = slice_event_v2(&keys, "ROOT", slice).expect("build event");
            let decoded = event_to_slice_v2(&event, "ROOT").expect("decode event");
            manager.record_slice(decoded);
        }
        assert!(manager.rejected.is_empty());
        assert!(manager.verify_integrity());
        assert_eq!(manager.reconstruct_payload().expect("payload"), payload);
    }

    #[test]
    fn cross_version_events_are_refused_both_directions() {
        // Unlike raw serde (which silently drops v2's len when read as v1),
        // the envelope is strict: the version tag is checked before content
        // is parsed, so neither direction degrades silently.
        let keys = test_key();
        let v1_event = slice_event_v1(&keys, "R", &canonical_v1()).expect("v1 event");
        let v2_event = slice_event_v2(&keys, "R", &canonical_v2()).expect("v2 event");

        assert_eq!(
            event_to_slice_v2(&v1_event, "R").expect_err("v2 must refuse v1"),
            NostrCarrierError::WrongVersion {
                expected: "2",
                found: "1".to_string(),
            }
        );
        assert_eq!(
            event_to_slice_v1(&v2_event, "R").expect_err("v1 must refuse v2"),
            NostrCarrierError::WrongVersion {
                expected: "1",
                found: "2".to_string(),
            }
        );
    }

    #[test]
    fn tampered_content_is_rejected_on_record_not_on_parse() {
        let keys = test_key();
        let slice = canonical_v2();
        let mut event = slice_event_v2(&keys, "R", &slice).expect("build event");

        // Tamper inside the data array: still valid JSON, wrong bytes.
        let byte = event.content.find("222").expect("data byte present");
        event.content.replace_range(byte..byte + 3, "223");

        // Parsing succeeds; the manifest check is what fails closed.
        let decoded = event_to_slice_v2(&event, "R").expect("tampered event parses");
        assert_ne!(decoded, slice);
        let manifest = crate::perfect_ip_v2::generate_manifest_v2("R".to_string(), &slice.data);
        let mut manager = IntegrityManagerV2::new(manifest);
        manager.record_slice(decoded);
        assert_eq!(manager.rejected.len(), 1);
        assert!(manager.reconstruct_payload().is_err());
    }

    #[test]
    fn wrong_kind_wrong_root_and_missing_tags_are_refused() {
        let keys = test_key();
        let slice = canonical_v2();

        let wrong_kind = EventBuilder::new(
            EventKind::from(1u32),
            serde_json::to_string(&slice).unwrap(),
            envelope_tags("R", "R", PERFECT_IP_VERSION_V2),
        )
        .to_event(&keys)
        .unwrap();
        assert!(matches!(
            event_to_slice_v2(&wrong_kind, "R").expect_err("wrong kind"),
            NostrCarrierError::WrongKind { .. }
        ));

        let wrong_root = slice_event_v2(&keys, "OTHER", &slice).expect("build event");
        assert!(matches!(
            event_to_slice_v2(&wrong_root, "R").expect_err("wrong root"),
            NostrCarrierError::WrongRoot { .. }
        ));

        let no_version = EventBuilder::new(
            EventKind::from(PERFECT_IP_NOSTR_KIND),
            serde_json::to_string(&slice).unwrap(),
            vec![Tag::new(&["d", "R", "R"])],
        )
        .to_event(&keys)
        .unwrap();
        assert!(matches!(
            event_to_slice_v2(&no_version, "R").expect_err("missing version"),
            NostrCarrierError::MissingVersionTag
        ));

        let no_d = EventBuilder::new(
            EventKind::from(PERFECT_IP_NOSTR_KIND),
            serde_json::to_string(&slice).unwrap(),
            vec![Tag::new(&[PERFECT_IP_TAG, "2"])],
        )
        .to_event(&keys)
        .unwrap();
        assert!(matches!(
            event_to_slice_v2(&no_d, "R").expect_err("missing d"),
            NostrCarrierError::MissingDTag
        ));
    }

    #[test]
    fn manifest_event_round_trips_and_discriminates() {
        let keys = test_key();
        let payload: Vec<u8> = (0..3000).map(|i| (i % 251) as u8).collect();
        let result = packetize_v2("ROOT".to_string(), payload);

        let event = manifest_event_v2(&keys, "ROOT", &result.manifest).expect("build manifest");
        let d_tag = find_tag(&event, "d").expect("d tag");
        assert_eq!(d_tag.get_index(1), "ROOT");
        assert_eq!(d_tag.get_index(2), PERFECT_IP_MANIFEST_MARK);

        let decoded = event_to_manifest_v2(&event, "ROOT").expect("decode manifest");
        assert_eq!(decoded, result.manifest);

        // A manifest event is not a slice, and a slice event is not a
        // manifest — the decoders discriminate explicitly.
        assert!(matches!(
            event_to_slice_v2(&event, "ROOT").expect_err("manifest is not a slice"),
            NostrCarrierError::NotASlice
        ));
        let slice_event = slice_event_v2(&keys, "ROOT", &result.batch.packets[0]).unwrap();
        assert!(matches!(
            event_to_manifest_v2(&slice_event, "ROOT").expect_err("slice is not a manifest"),
            NostrCarrierError::NotAManifest
        ));

        // v1 manifest (bare id list) round trips too.
        let ids: Vec<String> = result
            .manifest
            .iter()
            .map(|entry| entry.id.clone())
            .collect();
        let v1_event = manifest_event_v1(&keys, "ROOT", &ids).expect("build v1 manifest");
        assert_eq!(
            event_to_manifest_v1(&v1_event, "ROOT").expect("decode v1"),
            ids
        );
    }

    /// End-to-end: publish the manifest + every slice as kind-9261 events to
    /// a live loopback gnostr-relay, fetch them back with a tag filter, and
    /// reconstruct the payload using ONLY the root id as prior knowledge.
    #[tokio::test]
    #[serial_test::serial]
    async fn slice_events_round_trip_through_live_relay() {
        use actix_test::start;
        use gnostr_relay::App as GnostrRelayApp;
        use tempfile::tempdir;

        struct EnvVarGuard {
            key: &'static str,
            value: Option<std::ffi::OsString>,
        }
        impl EnvVarGuard {
            fn set(key: &'static str, value: impl AsRef<std::ffi::OsStr>) -> Self {
                let previous = std::env::var_os(key);
                unsafe {
                    std::env::set_var(key, value);
                }
                Self {
                    key,
                    value: previous,
                }
            }
        }
        impl Drop for EnvVarGuard {
            fn drop(&mut self) {
                match &self.value {
                    Some(value) => unsafe {
                        std::env::set_var(self.key, value);
                    },
                    None => unsafe {
                        std::env::remove_var(self.key);
                    },
                }
            }
        }

        let home_dir = tempdir().expect("home dir");
        let _home_guard = EnvVarGuard::set("HOME", home_dir.path());
        let _xdg_guard = EnvVarGuard::set("XDG_CONFIG_HOME", home_dir.path().join("config"));

        let config_dir = tempdir().expect("relay config dir");
        let config_path = config_dir.path().join("relay.toml");
        std::fs::write(
            &config_path,
            r#"
[server]
port = 0
host = "127.0.0.1"

[database]
path = ":memory:"
"#,
        )
        .expect("write relay config");

        let data_dir = tempdir().expect("relay data dir");
        let data_path = data_dir.path().to_path_buf();
        let relay_srv = start(move || {
            let app_data = GnostrRelayApp::create(
                Some(config_path.as_path()),
                true,
                Some("NOSTR".to_owned()),
                Some(data_path.as_path()),
            )
            .expect("failed to create relay app");
            app_data.web_app()
        });
        let relay_url = relay_srv.url("/").replace("http", "ws");

        let keys = test_key();
        let payload: Vec<u8> = (0..3000).map(|i| (i % 251) as u8).collect();
        let result = packetize_v2("ROOT".to_string(), payload.clone());
        let expected_events = result.batch.packets.len() + 1; // slices + manifest

        let mut client = crate::nostr::Client::new(
            &crate::nostr::Keys::new(keys.clone()),
            crate::nostr::Options::new()
                .wait_for_send(true)
                // Each confirmed send opens a fresh WebSocket and waits for
                // the relay's OK; the option default of 1s is too tight
                // under sequential sends.
                .send_timeout(Some(std::time::Duration::from_secs(10))),
        );
        client
            .add_relays(vec![relay_url.clone()])
            .await
            .expect("add relay");
        client.connect().await;

        // Publish: manifest first, then every slice. Confirmed sends wait
        // for the relay's `OK true`, surfacing any rejection instead of
        // fire-and-forgetting.
        client
            .send_event_confirmed(
                manifest_event_v2(&keys, "ROOT", &result.manifest).expect("manifest"),
            )
            .await
            .expect("publish manifest");
        for slice in &result.batch.packets {
            client
                .send_event_confirmed(slice_event_v2(&keys, "ROOT", slice).expect("slice event"))
                .await
                .expect("publish slice");
        }

        // Fetch everything tagged for this root. The LMDB write worker is
        // async, so retry the query briefly until the set is complete.
        let mut events = Vec::new();
        for _ in 0..10 {
            let mut filter = crate::nostr::Filter::new();
            filter
                .add_event_kind(EventKind::from(PERFECT_IP_NOSTR_KIND))
                .add_tag_value('t', PERFECT_IP_TOPIC.to_string());
            events = client
                .get_events_of(vec![filter], Some(std::time::Duration::from_secs(5)))
                .await
                .expect("query relay");
            if events.len() >= expected_events {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
        assert_eq!(
            events.len(),
            expected_events,
            "expected {expected_events} events from relay, got {}",
            events.len()
        );

        // Receiver side: prior knowledge is ONLY the root id. The manifest
        // event supplies the verification material; slice events are
        // hash-checked against it on record.
        let manifest_event = events
            .iter()
            .find(|event| d_tag_kind(event) == PERFECT_IP_MANIFEST_MARK)
            .expect("manifest event in relay set");
        let manifest = event_to_manifest_v2(manifest_event, "ROOT").expect("decode manifest");

        let mut manager = IntegrityManagerV2::new(manifest);
        let mut slice_count = 0usize;
        for event in &events {
            if d_tag_kind(event) == PERFECT_IP_MANIFEST_MARK {
                continue;
            }
            let slice = event_to_slice_v2(event, "ROOT").expect("decode slice event");
            manager.record_slice(slice);
            slice_count += 1;
        }
        assert_eq!(slice_count, result.batch.packets.len());
        assert!(
            manager.rejected.is_empty(),
            "relay-delivered slices must hash-verify: {:?}",
            manager.rejected
        );
        assert!(manager.verify_integrity());
        assert_eq!(
            manager.reconstruct_payload().expect("payload"),
            payload,
            "payload reconstructed from relay-delivered events must be byte-exact"
        );
    }
}
