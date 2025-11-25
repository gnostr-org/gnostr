// Copyright 2015-2020 nostr-proto Developers
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according to those terms.

//! This crate provides types for nostr protocol handling.

#![deny(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    //unused_qualifications,
    unused_results,
    unused_lifetimes,
    unused_labels,
    unused_extern_crates,
    non_ascii_idents,
    keyword_idents,
    deprecated_in_future,
    unstable_features,
    single_use_lifetimes,
    //unsafe_code,
    unreachable_pub,
    missing_docs,
    missing_copy_implementations
)]
#![deny(clippy::string_slice)]

/// internal
pub mod internal;

mod client_message;
pub use client_message::ClientMessage;

mod content;
pub use content::{ContentSegment, ShatteredContent, Span};

mod delegation;
pub use delegation::{DelegationConditions, EventDelegation};

mod error;
pub use error::Error;

mod event;
pub use event::{Event, PreEvent, Rumor, ZapData};

mod event_kind;
pub use event_kind::{EventKind, EventKindIterator, EventKindOrRange};

mod event_reference;
pub use event_reference::EventReference;

mod filter;
pub use filter::Filter;

mod id;
pub use id::{Id, IdHex};

mod identity;
pub use identity::Identity;

mod key_signer;
pub use key_signer::KeySigner;

mod metadata;
pub use metadata::Metadata;

mod naddr;
pub use naddr::NAddr;

mod nevent;
pub use nevent::NEvent;

pub mod nip0;
pub mod nip2;
pub mod nip4;
pub mod nip6;
pub mod nip9;
pub mod nip15;
pub mod nip26;
pub mod nip34;
mod nip05;
pub use nip05::Nip05;

mod nostr_url;
pub use nostr_url::{find_nostr_bech32_pos, find_nostr_url_pos, NostrBech32, NostrUrl};

mod pay_request_data;
pub use pay_request_data::PayRequestData;

mod private_key;
pub use private_key::{ContentEncryptionAlgorithm, EncryptedPrivateKey, KeySecurity, PrivateKey};

mod profile;
pub use profile::Profile;

mod public_key;
pub use public_key::{PublicKey, PublicKeyHex};
pub use secp256k1::XOnlyPublicKey;

mod relay_information_document;
pub use relay_information_document::{
    Fee, RelayFees, RelayInformationDocument, RelayLimitation, RelayRetention,
};

mod relay_list;
pub use relay_list::{RelayList, RelayListUsage};

mod relay_message;
pub use relay_message::RelayMessage;

mod relay_usage;
pub use relay_usage::{RelayUsage, RelayUsageSet};

mod satoshi;
pub use satoshi::MilliSatoshi;

mod signature;
pub use signature::{Signature, SignatureHex};

mod signer;
pub use signer::Signer;

mod simple_relay_list;
pub use simple_relay_list::{SimpleRelayList, SimpleRelayUsage};

mod subscription_id;
pub use subscription_id::SubscriptionId;

mod tag;
pub use tag::Tag;

mod unixtime;
pub use unixtime::Unixtime;

mod url;
pub use self::url::{RelayOrigin, RelayUrl, UncheckedUrl, Url};

/// NIP-44 related types and functionalities for secure direct messages.
pub mod nip44;
pub use nip44::{decrypt, encrypt, get_conversation_key, Error as Nip44Error};

#[cfg(test)]
#[macro_export]
/// A helper macro for testing `serde` serialization and deserialization.
macro_rules! test_serde {
    ($t:ty, $fnname:ident) => {
        #[test]
        fn $fnname() {
            let a = <$t>::mock();
            let x = serde_json::to_string(&a).unwrap();
            println!("{}", x);
            let b = serde_json::from_str(&x).unwrap();
            assert_eq!(a, b);
        }
    };
}

// mod types;


mod versioned;
pub use versioned::{
    ClientMessageV1, ClientMessageV2, ClientMessageV3, EventV1, EventV2, EventV3, FeeV1,
    MetadataV1, Nip05V1, PreEventV1, PreEventV2, PreEventV3, RelayFeesV1,
    RelayInformationDocumentV1, RelayInformationDocumentV2, RelayLimitationV1, RelayLimitationV2,
    RelayMessageV1, RelayMessageV2, RelayMessageV3, RelayMessageV4, RelayMessageV5,
    RelayRetentionV1, RumorV1, RumorV2, RumorV3, TagV1, TagV2, TagV3, Why, ZapDataV1, ZapDataV2,
};

#[inline]
pub(crate) fn get_leading_zero_bits(bytes: &[u8]) -> u8 {
    let mut res = 0_u8;
    for b in bytes {
        if *b == 0 {
            res += 8;
        } else {
            res += b.leading_zeros() as u8;
            return res;
        }
    }
    res
}

/// Trait for converting Option<T> into Vec<T>
pub trait IntoVec<T> {
    /// Convert into a Vec<T>
    fn into_vec(self) -> Vec<T>;
}

impl<T> IntoVec<T> for Option<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            None => vec![],
            Some(t) => vec![t],
        }
    }
}

use bech32::Hrp;
lazy_static::lazy_static! {
    static ref HRP_LNURL: Hrp = Hrp::parse("lnurl").expect("HRP error on lnurl");
    static ref HRP_NADDR: Hrp = Hrp::parse("naddr").expect("HRP error on naddr");
    static ref HRP_NCRYPTSEC: Hrp = Hrp::parse("ncryptsec").expect("HRP error on ncryptsec");
    static ref HRP_NEVENT: Hrp = Hrp::parse("nevent").expect("HRP error on nevent");
    static ref HRP_NOTE: Hrp = Hrp::parse("note").expect("HRP error on note");
    static ref HRP_NPROFILE: Hrp = Hrp::parse("nprofile").expect("HRP error on nprofile");
    static ref HRP_NPUB: Hrp = Hrp::parse("npub").expect("HRP error on npub");
    static ref HRP_NRELAY: Hrp = Hrp::parse("nrelay").expect("HRP error on nrelay");
    static ref HRP_NSEC: Hrp = Hrp::parse("nsec").expect("HRP error on nsec");
}

/// Add a 'p' pubkey tag to a set of tags if it doesn't already exist
pub fn add_pubkey_to_tags(
    existing_tags: &mut Vec<Tag>,
    new_pubkey: PublicKey,
    new_hint: Option<UncheckedUrl>,
) -> usize {
    match existing_tags.iter().position(|existing_tag| {
        if let Ok((pubkey, _, __)) = existing_tag.parse_pubkey() {
            pubkey == new_pubkey
        } else {
            false
        }
    }) {
        Some(idx) => idx,
        None => {
            existing_tags.push(Tag::new_pubkey(new_pubkey, new_hint, None));
            existing_tags.len() - 1
        }
    }
}

/// Add an 'e' id tag to a set of tags if it doesn't already exist
pub fn add_event_to_tags(
    existing_tags: &mut Vec<Tag>,
    new_id: Id,
    new_hint: Option<UncheckedUrl>,
    new_marker: &str,
    use_quote: bool,
) -> usize {
    if new_marker == "mention" && use_quote {
        // NIP-18: "Quote reposts are kind 1 events with an embedded q tag..."
        let newtag = Tag::new_quote(new_id, new_hint);

        match existing_tags.iter().position(|existing_tag| {
            if let Ok((id, _rurl)) = existing_tag.parse_quote() {
                id == new_id
            } else {
                false
            }
        }) {
            None => {
                existing_tags.push(newtag);
                existing_tags.len() - 1
            }
            Some(idx) => idx,
        }
    } else {
        let newtag = Tag::new_event(new_id, new_hint, Some(new_marker.to_string()));

        match existing_tags.iter().position(|existing_tag| {
            if let Ok((id, _rurl, _optmarker)) = existing_tag.parse_event() {
                id == new_id
            } else {
                false
            }
        }) {
            None => {
                existing_tags.push(newtag);
                existing_tags.len() - 1
            }
            Some(idx) => idx,
        }
    }
}

/// Add an 'a' addr tag to a set of tags if it doesn't already exist
pub fn add_addr_to_tags(
    existing_tags: &mut Vec<Tag>,
    new_addr: &NAddr,
    new_marker: Option<String>,
) -> usize {
    match existing_tags.iter().position(|existing_tag| {
        if let Ok((ea, _optmarker)) = existing_tag.parse_address() {
            ea.kind == new_addr.kind && ea.author == new_addr.author && ea.d == new_addr.d
        } else {
            false
        }
    }) {
        Some(idx) => idx,
        None => {
            existing_tags.push(Tag::new_address(new_addr, new_marker));
            existing_tags.len() - 1
        }
    }
}

/// Add an 'subject' tag to a set of tags if it doesn't already exist
pub fn add_subject_to_tags_if_missing(existing_tags: &mut Vec<Tag>, subject: String) {
    if !existing_tags.iter().any(|t| t.tagname() == "subject") {
        existing_tags.push(Tag::new_subject(subject));
    }
}


#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_real_messages() {
        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"75468b04a0e03633a40f1c8d7e1a0cad1363ecc514ecbcde22093874e04e8166","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011201,"kind":1,"tags":[["e","247baa8ed5db8097b16d9594a3a27fd2b64c030fa9e68ce7d6106df4a499700d","","reply"],["p","6b0d4c8d9dc59e110d380b0429a02891f1341a0fa2ba1b1cf83a3db4d47e3964","","reply"]],"content":"you're not allowed to pronounce these words, traitor","sig":"588577ccd5ad6be8f61d93e4738799dede9b169ad150ee3ee6a1c4bb80adfbee27bb4e302e0ea173637c189d6664f1dc82ad3590b5524240bf492fa0b754432c"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.content);
            }
            _ => panic!("Wrong message type"),
        }

        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"267660849149c7226a4a4f7c75f359f3995965c05d25451f13c907bf0b158178","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011264,"kind":1,"tags":[["e","8a128cd11c6a56554b8201635a19c97258504060464cec4f3e5f0500814339cf","","reply"],["p","000000000652e452ee68a01187fb08c899496cb46cb51d1aa0803d063acedba7","","reply"]],"content":"this is quite nice, specially the part where you say it was written in Rust.","sig":"1c49b4f4d2b86077ae4c1f7f8dc212d6c040dfdff7864eac2154fe7df1baceb162cf658d78634b803b964f920aeb861014ed30df113ed0857aaf1854e3c572a3"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.as_ref().content);
            }
            _ => panic!("Wrong message type"),
        }

        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"fe0cfc6d2be988f46f849535518c3e43a509ea8a016ccd8b83a3ffd79575fd33","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011340,"kind":1,"tags":[["e","b1a2a2e55f1b6f1f6756e6e4c1c4ecbce0123ede048423413228134143fd84ac","","root"],["e","c758d9d467bf925923f57bb6b47db870fad50ba9629bc086f573f3d4ff278c84","","reply"],["p","9ec7a778167afb1d30c4833de9322da0c08ba71a69e1911d5578d3144bb56437","","root"],["p","32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245","","reply"]],"content":"they are definitely annoying in Go, but we already have them anyway because of the `[\"EVENT\", {}]` message so this doesn't make any difference in my case at least.","sig":"23b1eed3087a72f2e940c1c95541b22b3434390926780ed055abf5dd77a3aa16e1c5c3965382ec7343c0da3ece31e05945f910d684f3196e81e05765a5b1e631"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.content);
            }
            _ => panic!("Wrong message type"),
        }

        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"adf038ca047260a20f70b7863c3a8ef7afdac455cd9fcb785950b86ebb104911","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011516,"kind":1,"tags":[["e","c0138298e2ac89078e206aea1e16f1d9a37257c8400f48aba781dd890bc9f35b","","root"],["e","24b757dfc938d9d29d7be40ac91424bfecd8c0016929ac911447a2f785519d97","","reply"],["p","3235036bd0957dfb27ccda02d452d7c763be40c91a1ac082ba6983b25238388c","","root"],["p","46fcbe3065eaf1ae7811465924e48923363ff3f526bd6f73d7c184b16bd8ce4d","","reply"]],"content":"when I started writing branle a million years ago I thought it would be so much simpler too, I guess that explains why twitter has 800 developers on its payroll","sig":"0f7d1cfbcc38bb861f51538cb8e4a5268e2bdca13969eaba8d0993e19fa8469d9ebcc60081523d075ca63c7ab55270e2a3de2373db605cde081b82357907af1f"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.content);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
