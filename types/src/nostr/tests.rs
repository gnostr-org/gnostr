//! Real serde and contract coverage for the asyncgit Nostr type surface.

use super::*;

crate::test_serde! {DelegationConditions, test_delegation_conditions_serde}
crate::test_serde! {EventKind, test_event_kind_serde}
crate::test_serde! {Filter, test_filter_serde}
crate::test_serde! {Id, test_id_serde}
crate::test_serde! {IdHex, test_id_hex_serde}
crate::test_serde! {MilliSatoshi, test_milli_satoshi_serde}
crate::test_serde! {MetadataV1, test_metadata_v1_serde}
crate::test_serde! {NAddr, test_naddr_serde}
crate::test_serde! {NEvent, test_nevent_serde}
crate::test_serde! {Nip05V1, test_nip05_v1_serde}
crate::test_serde! {PayRequestData, test_pay_request_data_serde}
crate::test_serde! {PublicKey, test_public_key_serde}
crate::test_serde! {PublicKeyHex, test_public_key_hex_serde}
crate::test_serde! {RelayInformationDocumentV1, test_relay_information_document_v1_serde}
crate::test_serde! {RelayInformationDocumentV2, test_relay_information_document_v2_serde}
crate::test_serde! {RelayMessageV2, test_relay_message_v2_serde}
crate::test_serde! {RelayMessageV3, test_relay_message_v3_serde}
crate::test_serde! {RelayMessageV4, test_relay_message_v4_serde}
crate::test_serde! {RelayMessageV5, test_relay_message_v5_serde}
crate::test_serde! {Signature, test_signature_serde}
crate::test_serde! {SignatureHex, test_signature_hex_serde}
crate::test_serde! {SimpleRelayList, test_simple_relay_list_serde}
crate::test_serde! {SubscriptionId, test_subscription_id_serde}
crate::test_serde! {TagV1, test_tag_v1_serde}
crate::test_serde! {TagV2, test_tag_v2_serde}
crate::test_serde! {TagV3, test_tag_v3_serde}
crate::test_serde! {Unixtime, test_unixtime_serde}
crate::test_serde! {UncheckedUrl, test_unchecked_url_serde}
crate::test_serde! {Url, test_url_serde}
crate::test_serde! {ClientMessageV2, test_client_message_v2_serde}
crate::test_serde! {ClientMessageV3, test_client_message_v3_serde}
crate::test_serde! {EventV2, test_event_v2_serde}
crate::test_serde! {EventV3, test_event_v3_serde}

#[test]
fn event_kind_mapping_covers_known_nip34_kinds() {
    assert_eq!(u32::from(EventKind::RepositoryAnnouncement), 30617);
    assert_eq!(u32::from(EventKind::GitRepoAnnouncement), 30618);
    assert_eq!(u32::from(EventKind::GitIssue), 1621);
    assert_eq!(u32::from(EventKind::GitReply), 1622);
    assert_eq!(u32::from(EventKind::GitStatusOpen), 1630);
    assert_eq!(u32::from(EventKind::GitStatusApplied), 1631);
    assert_eq!(u32::from(EventKind::GitStatusClosed), 1632);
    assert_eq!(u32::from(EventKind::GitStatusDraft), 1633);
    assert_eq!(EventKind::from(1618), EventKind::Other(1618));
    assert_eq!(EventKind::from(1619), EventKind::Other(1619));
    assert_eq!(EventKind::from(10317), EventKind::Replaceable(10317));
}

#[test]
fn naddr_round_trips_through_bech32() {
    let naddr = NAddr::mock();
    let encoded = naddr.as_bech32_string();
    let decoded = NAddr::try_from_bech32_string(&encoded).expect("naddr bech32");
    assert_eq!(decoded.d, naddr.d);
    assert_eq!(decoded.kind, naddr.kind);
    assert_eq!(decoded.author, naddr.author);
}

#[test]
fn nevent_round_trips_through_bech32() {
    let mut nevent = NEvent::mock();
    nevent.kind = Some(EventKind::Patches);
    nevent.author = Some(PublicKey::mock_deterministic());
    let encoded = nevent.as_bech32_string();
    let decoded = NEvent::try_from_bech32_string(&encoded).expect("nevent bech32");
    assert_eq!(decoded.id, nevent.id);
    assert_eq!(decoded.relays, nevent.relays);
    assert_eq!(decoded.kind, nevent.kind);
    assert_eq!(decoded.author, nevent.author);
}

#[test]
fn filter_matches_event_shape_after_roundtrip() {
    let filter = Filter::mock();
    let json = serde_json::to_string(&filter).expect("filter json");
    let decoded: Filter = serde_json::from_str(&json).expect("filter roundtrip");
    assert_eq!(decoded.ids, filter.ids);
    assert_eq!(decoded.kinds, filter.kinds);
    assert_eq!(decoded.tags, filter.tags);
    assert_eq!(decoded.since, filter.since);
}

#[test]
fn content_shattering_finds_nostr_references() {
    let content = "note #[0] nostr:nevent1qqsqqqq9wh98g4u6e480vyp6p4w3ux2cd0mxn2rssq0w5cscsgzp2ksprpmhxue69uhkzapwdehhxarjwahhy6mn9e3k7mf0qyt8wumn8ghj7etyv4hzumn0wd68ytnvv9hxgtcpremhxue69uhkummnw3ez6ur4vgh8wetvd3hhyer9wghxuet59uq3kamnwvaz7tmwdaehgu3wd45kketyd9kxwetj9e3k7mf0qy2hwumn8ghj7mn0wd68ytn00p68ytnyv4mz7qgnwaehxw309ahkvenrdpskjm3wwp6kytcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsz9mhwden5te0wfjkccte9ehx7um5wghxyctwvshszxthwden5te0wfjkccte9eekummjwsh8xmmrd9skctcnmzajy https://example.com";
    let shattered = ShatteredContent::new(content.to_string());
    assert!(!shattered.segments.is_empty());
    assert!(shattered
        .segments
        .iter()
        .any(|segment| matches!(segment, ContentSegment::TagReference(0))));
}
