use std::fmt;

//use serde::de::Error as DeError;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{
    Deserialize, Serialize,
    ser::{SerializeMap, Serializer},
};
use serde_json::{Map, Value, json};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{FeeV1, RelayFeesV1, RelayRetentionV1};
use crate::types::{EventKind, EventKindOrRange, PublicKeyHex, Url};

/// Relay limitations
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayLimitationV2 {
    /// max message length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_message_length: Option<usize>,

    /// max subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_subscriptions: Option<usize>,

    /// max filters
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_filters: Option<usize>,

    /// max limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_limit: Option<usize>,

    /// max subid length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_subid_length: Option<usize>,

    /// max event tags
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_event_tags: Option<usize>,

    /// max content length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_content_length: Option<usize>,

    /// min pow difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub min_pow_difficulty: Option<usize>,

    /// auth required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub auth_required: Option<bool>,

    /// payment required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub payment_required: Option<bool>,

    /// restricted writes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub restricted_writes: Option<bool>,

    /// created at lower limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub created_at_lower_limit: Option<u64>,

    /// created at upper limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub created_at_upper_limit: Option<u64>,
}

impl fmt::Display for RelayLimitationV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Limitation:")?;
        if let Some(mml) = &self.max_message_length {
            write!(f, " MaxMessageLength=\"{mml}\"")?;
        }
        if let Some(ms) = &self.max_subscriptions {
            write!(f, " MaxSubscriptions=\"{ms}\"")?;
        }
        if let Some(mf) = &self.max_filters {
            write!(f, " MaxFilters=\"{mf}\"")?;
        }
        if let Some(ml) = &self.max_limit {
            write!(f, " MaxLimit=\"{ml}\"")?;
        }
        if let Some(msil) = &self.max_subid_length {
            write!(f, " MaxSubidLength=\"{msil}\"")?;
        }
        if let Some(met) = &self.max_event_tags {
            write!(f, " MaxEventTags=\"{met}\"")?;
        }
        if let Some(mcl) = &self.max_content_length {
            write!(f, " MaxContentLength=\"{mcl}\"")?;
        }
        if let Some(mpd) = &self.min_pow_difficulty {
            write!(f, " MinPowDifficulty=\"{mpd}\"")?;
        }
        if let Some(ar) = &self.auth_required {
            write!(f, " AuthRequired=\"{ar}\"")?;
        }
        if let Some(pr) = &self.payment_required {
            write!(f, " PaymentRequired=\"{pr}\"")?;
        }
        if let Some(rw) = &self.restricted_writes {
            write!(f, " RestrictedWrites=\"{rw}\"")?;
        }
        if let Some(call) = &self.created_at_lower_limit {
            write!(f, " CreatedAtLowerLimit=\"{call}\"")?;
        }
        if let Some(caul) = &self.created_at_upper_limit {
            write!(f, " CreatedAtUpperLimit=\"{caul}\"")?;
        }
        Ok(())
    }
}

/// Relay information document as described in NIP-11, supplied by a relay
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayInformationDocumentV2 {
    /// Name of the relay
    pub name: Option<String>,

    /// Description of the relay in plain text
    pub description: Option<String>,

    /// A banner image for the relay
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub banner: Option<Url>,

    /// An icon for the relay
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub icon: Option<Url>,

    /// Public key of an administrative contact of the relay
    pub pubkey: Option<PublicKeyHex>,

    /// The relay's public key, for signing events
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub self_pubkey: Option<PublicKeyHex>,

    /// An administrative contact for the relay. Should be a URI.
    pub contact: Option<String>,

    /// A list of NIPs supported by the relay
    pub supported_nips: Vec<u32>,

    /// The software running the relay
    pub software: Option<String>,

    /// The software version
    pub version: Option<String>,

    /// limitation
    pub limitation: Option<RelayLimitationV2>,

    /// retention
    pub retention: Vec<RelayRetentionV1>,

    /// content limitation: relay countries
    pub relay_countries: Vec<String>,

    /// community preferences: language tags
    pub language_tags: Vec<String>,

    /// community preferences: tags
    pub tags: Vec<String>,

    /// community preferences: posting policy
    pub posting_policy: Option<Url>,

    /// payments_url
    pub payments_url: Option<Url>,

    /// fees
    pub fees: Option<RelayFeesV1>,

    /// Additional fields not specified in NIP-11
    pub other: Map<String, Value>,
}

impl Default for RelayInformationDocumentV2 {
    fn default() -> RelayInformationDocumentV2 {
        RelayInformationDocumentV2 {
            name: None,
            description: None,
            banner: None,
            icon: None,
            pubkey: None,
            self_pubkey: None,
            contact: None,
            supported_nips: vec![],
            software: None,
            version: None,
            limitation: None,
            retention: vec![],
            relay_countries: vec![],
            language_tags: vec![],
            tags: vec![],
            posting_policy: None,
            payments_url: None,
            fees: None,
            other: Map::new(),
        }
    }
}

impl RelayInformationDocumentV2 {
    /// If the relay supports the queried `nip`
    pub fn supports_nip(&self, nip: u32) -> bool {
        self.supported_nips.contains(&nip)
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayInformationDocumentV2 {
        let mut m = Map::new();
        let _ = m.insert(
            "early_nips".to_string(),
            Value::Array(vec![
                Value::Number(5.into()),
                Value::Number(6.into()),
                Value::Number(7.into()),
            ]),
        );
        RelayInformationDocumentV2 {
            name: Some("Crazy Horse".to_string()),
            description: Some("A really wild horse".to_string()),
            banner: Some(Url::try_from_str("https://example.com/banner.jpg").unwrap()),
            icon: Some(Url::try_from_str("https://example.com/icon.jpg").unwrap()),
            pubkey: Some(PublicKeyHex::mock()),
            self_pubkey: Some(PublicKeyHex::mock()),
            contact: None,
            supported_nips: vec![11, 12, 13, 14],
            software: None,
            version: None,
            limitation: Some(RelayLimitationV2 {
                max_message_length: Some(16384),
                max_subscriptions: Some(20),
                max_filters: Some(100),
                max_limit: Some(5000),
                max_subid_length: Some(100),
                max_event_tags: Some(100),
                max_content_length: Some(8196),
                min_pow_difficulty: Some(30),
                auth_required: Some(true),
                payment_required: Some(true),
                restricted_writes: Some(true),
                created_at_lower_limit: None,
                created_at_upper_limit: None,
            }),
            retention: vec![
                RelayRetentionV1 {
                    kinds: vec![
                        EventKindOrRange::EventKind(EventKind::Metadata),
                        EventKindOrRange::EventKind(EventKind::TextNote),
                        EventKindOrRange::Range(vec![
                            EventKind::EventDeletion,
                            EventKind::Reaction,
                        ]),
                        EventKindOrRange::Range(vec![EventKind::ChannelCreation]),
                    ],
                    time: Some(3600),
                    count: None,
                },
                RelayRetentionV1 {
                    kinds: vec![EventKindOrRange::Range(vec![
                        EventKind::Other(40000),
                        EventKind::Other(49999),
                    ])],
                    time: Some(100),
                    count: None,
                },
                RelayRetentionV1 {
                    kinds: vec![EventKindOrRange::Range(vec![
                        EventKind::FollowSets,
                        EventKind::Other(39999),
                    ])],
                    time: None,
                    count: Some(1000),
                },
                RelayRetentionV1 {
                    kinds: vec![],
                    time: Some(3600),
                    count: Some(10000),
                },
            ],
            relay_countries: vec!["CA".to_owned(), "US".to_owned()],
            language_tags: vec!["en".to_owned()],
            tags: vec!["sfw-only".to_owned(), "bitcoin-only".to_owned()],
            posting_policy: Some(
                Url::try_from_str("https://example.com/posting-policy.html").unwrap(),
            ),
            payments_url: Some(Url::try_from_str("https://example.com/payments").unwrap()),
            fees: Some(RelayFeesV1 {
                admission: vec![FeeV1 {
                    amount: 1000000,
                    unit: "msats".to_owned(),
                    kinds: vec![],
                    period: None,
                }],
                subscription: vec![FeeV1 {
                    amount: 5000000,
                    unit: "msats".to_owned(),
                    kinds: vec![],
                    period: Some(2592000),
                }],
                publication: vec![FeeV1 {
                    amount: 100,
                    unit: "msats".to_owned(),
                    kinds: vec![EventKindOrRange::EventKind(EventKind::EventDeletion)],
                    period: None,
                }],
            }),
            other: m,
        }
    }
}

impl fmt::Display for RelayInformationDocumentV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Information:")?;
        if let Some(name) = &self.name {
            write!(f, " Name=\"{name}\"")?;
        }
        if let Some(desc) = &self.description {
            write!(f, " Description=\"{desc}\"")?;
        }
        if let Some(banner) = &self.banner {
            write!(f, " Banner=\"{banner}\"")?;
        }
        if let Some(icon) = &self.icon {
            write!(f, " Icon=\"{icon}\"")?;
        }
        if let Some(pubkey) = &self.pubkey {
            write!(f, " Pubkey=\"{pubkey}\"")?;
        }
        if let Some(self_pubkey) = &self.self_pubkey {
            write!(f, " SelfPubkey=\"{self_pubkey}\"")?;
        }
        if let Some(contact) = &self.contact {
            write!(f, " Contact=\"{contact}\"")?;
        }
        if !self.supported_nips.is_empty() {
            write!(f, " NIPS={:?}", self.supported_nips)?;
        }
        if let Some(software) = &self.software {
            write!(f, " Software=\"{software}\"")?;
        }
        if let Some(version) = &self.version {
            write!(f, " Version=\"{version}\"")?;
        }
        if let Some(limitation) = &self.limitation {
            write!(f, " Limitation=\"{limitation}\"")?;
        }
        for retention in &self.retention {
            write!(f, " Retention=\"{retention}\"")?;
        }
        if !self.relay_countries.is_empty() {
            write!(f, " Countries=[")?;
            for country in &self.relay_countries {
                write!(f, "{country},")?;
            }
            write!(f, "]")?;
        }
        if !self.language_tags.is_empty() {
            write!(f, " Languages=[")?;
            for language in &self.language_tags {
                write!(f, "{language},")?;
            }
            write!(f, "]")?;
        }
        if !self.tags.is_empty() {
            write!(f, " Tags=[")?;
            for tag in &self.tags {
                write!(f, "{tag},")?;
            }
            write!(f, "]")?;
        }
        if let Some(policy_url) = &self.posting_policy {
            write!(f, " PostingPolicy={policy_url}")?;
        }
        if let Some(url) = &self.payments_url {
            write!(f, " PaymentsUrl={url}")?;
        }
        if let Some(fees) = &self.fees {
            write!(f, " Fees={fees}")?;
        }
        for (k, v) in self.other.iter() {
            write!(f, " {k}=\"{v}\"")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayInformationDocumentV2, test_relay_information_document_serde}

    #[test]
    fn test_to_json_only() {
        // This is so you can see the JSON limitation.
        // Run with "cargo test toest_to_json_only -- --nocapture"
        let mock = RelayInformationDocumentV2::mock();
        let s = serde_json::to_string(&mock).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_relay_information_document_json() {
        let json = r##"{
            "name": "A Relay",
            "description": null,
            "myfield": [1,2],
            "supported_nips": [11,12],
            "retention": [
                { "kinds": [0, 1, [5, 7], [40, 49]], "time": 3600 },
                { "kinds": [[40000, 49999]], "time": 100 },
                { "kinds": [[30000, 39999]], "count": 1000 },
                { "time": 3600, "count": 10000 }
            ],
            "relay_countries": ["CA", "US"],
            "language_tags": ["en", "br"],
            "tags":[],
            "other": { "misc_data": "value" }
        }"##;

        let rid: RelayInformationDocumentV2 = serde_json::from_str(json).unwrap();
        let json2 = serde_json::to_value(&rid).unwrap();

        let expected_json2: serde_json::Value = serde_json::from_str(
            r##"{
            "name": "A Relay",
            "description": null,
            "pubkey": null,
            "contact": null,
            "supported_nips": [11, 12],
            "software": null,
            "version": null,
            "limitation": null,
            "retention": [
                { "kinds": [0, 1, [5, 7], [40, 49]], "time": 3600 },
                { "kinds": [[40000, 49999]], "time": 100 },
                { "kinds": [[30000, 39999]], "count": 1000 },
                { "time": 3600, "count": 10000 }
            ],
            "relay_countries": ["CA", "US"],
            "language_tags": ["en", "br"],
            "tags": [],
            "posting_policy": null,
            "payments_url": null,
            "fees": null,
            "other": { "misc_data": "value" }
        }"##,
        )
        .unwrap();

        assert_eq!(json2, expected_json2);
    }
}
