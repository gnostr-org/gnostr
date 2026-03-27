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
    // unreachable_pub,
    missing_docs,
    missing_copy_implementations
)]
#![deny(clippy::string_slice)]

/// Module for handling errors.
pub mod error;
pub use error::Error;

/// NIP-34 related functionality.
pub mod nip34;


#[cfg(test)]
/// Module for test utilities.
pub mod test_utils;

/// NIP-44 related functionality.
pub mod nip44;

/// Client for interacting with Nostr relays.
pub mod client;
pub use client::*;

/// Module for client messages.
pub mod client_message;
pub use client_message::*;

/// Module for content.
pub mod content;
pub use content::*;

/// Module for delegation.
pub mod delegation;
pub use delegation::*;

/// Module for events.
pub mod event;
pub use event::*;

/// Module for event builders.
pub mod event_builder;
pub use event_builder::*;

/// Module for event kinds.
pub mod event_kind;
pub use event_kind::*;

/// Module for event references.
pub mod event_reference;
pub use event_reference::*;

/// Module for filters.
pub mod filter;
pub use filter::*;

/// Module for IDs.
pub mod id;
pub use id::*;

/// Module for image dimensions.
pub mod image_dimensions;
pub use image_dimensions::*;

/// Module for identities.
pub mod identity;
pub use identity::*;

/// Module for keys.
pub mod keys;
pub use keys::*;

/// Module for key signers.
pub mod key_signer;
pub use key_signer::*;

/// Module for handling Nostr event metadata.
pub mod metadata;
pub use metadata::*;

/// Module for NAddr.
pub mod naddr;
pub use naddr::*;

/// Module for NEvent.
pub mod nevent;
pub use nevent::*;

/// NIP-19 related functionality for Nostr entity bech32 encoding.
pub mod nip19;
pub use nip19::*;

/// Module for Nostr URLs.
pub mod nostr_url;
pub use nostr_url::*;

/// Module for pay request data.
pub mod pay_request_data;
pub use pay_request_data::*;

/// NIP-28 related functionality for public chat channels.
pub mod nip28;
pub use nip28::*;

/// Module for private keys.
pub mod private_key;
pub use private_key::*;

/// Module for profiles.
pub mod profile;
pub use profile::*;

/// Module for public keys.
pub mod public_key;
pub use public_key::*;

/// Module for relay information documents.
pub mod relay_information_document;
pub use relay_information_document::*;

/// Module for relay lists.
pub mod relay_list;
pub use relay_list::*;

/// Relay information document as described in NIP-11, supplied by a relay.
pub mod relay_info;
pub use relay_info::*;

/// Module for relay messages.
pub mod relay_message;
pub use relay_message::*;

/// Module for relay usage.
pub mod relay_usage;
pub use relay_usage::*;

/// Module for satoshi.
pub mod satoshi;
pub use satoshi::*;

/// Module for signatures.
pub mod signature;
pub use signature::*;

/// Module for signers.
pub mod signer;
pub use signer::*;

/// Module for simple relay lists.
pub mod simple_relay_list;
pub use simple_relay_list::*;

/// Module for subscription IDs.
pub mod subscription_id;
pub use subscription_id::*;

/// Module for tags.
pub mod tag;
pub use tag::*;

/// Module for Unix timestamps.
pub mod unixtime;
pub use unixtime::*;

/// Module for URLs.
pub mod url;
pub use url::*;

/// Module for weeble calculations.
pub mod weeble;
pub use weeble::*;

/// Module for blockheight calculations.
pub mod blockheight;
pub use blockheight::*;

/// Module for blockhash calculations.
pub mod blockhash;
pub use blockhash::*;

/// Module for wobble calculations.
pub mod wobble;
pub use wobble::*;

pub use internal::fetch;
pub use internal::event_to_wire;
pub use internal::post;
pub use internal::filters_to_wire;
/// Internal utility functions for event conversion and relay communication.
pub mod internal {
    #![allow(clippy::print_with_newline)]
    use base64::Engine;
    use http::Uri;
    use tokio_tungstenite::{tungstenite, tungstenite::Message};

    use crate::client_message::ClientMessage;
    use crate::event::Event;

    use crate::filter::Filter;
    use crate::relay_message::RelayMessage;
    use crate::versioned::relay_message5::RelayMessageV5;
    use crate::subscription_id::SubscriptionId;

    /// Convert a list of `Filter`s to a WebSocket wire message string.
    pub fn filters_to_wire(filters: Vec<Filter>) -> String {
        let message = ClientMessage::Req(
            SubscriptionId(format!(
                "{:?}/{:?}/{:?}",
                crate::weeble::weeble_sync(),
                crate::blockheight::blockheight_sync(),
                crate::weeble::weeble_sync(),
            )),
            filters,
        );
        serde_json::to_string(&message).expect("Could not serialize message")
    }

    /// Convert an `Event` to a WebSocket wire message string.
    pub fn event_to_wire(event: Event) -> String {
        let message = ClientMessage::Event(Box::new(event));
        serde_json::to_string(&message).expect("Could not serialize message")
    }

    // pub(crate) fn event_to_wire_v2(event: EventV2) -> String {
    //    let message = ClientMessage::Event_V2(Box::new(event));
    //    serde_json::to_string(&message).expect("Could not serialize message")
    //}

    /// Fetches events from a relay by sending a wire message.
    pub fn fetch(host: String, uri: Uri, wire: String) -> Vec<Event> {
        let mut events: Vec<Event> = Vec::new();

        let key: [u8; 16] = rand::random();
        let request = http::request::Request::builder()
            .method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                base64::engine::general_purpose::STANDARD.encode(key),
            )
            .uri(uri)
            .body(())
            .expect("Could not build request");

        let (mut websocket, _response) =
            tungstenite::connect(request).expect("Could not connect to relay");

        websocket
            .send(Message::Text(wire.into()))
            .expect("Could not send message to relay");

        loop {
            let message = match websocket.read() {
                Ok(m) => m,
                Err(e) => {
                    //handle differently
                    println!("Problem reading from websocket: {}", e);
                    return events;
                }
            };

            match message {
                Message::Text(s) => {
                    let relay_message: RelayMessageV5 = serde_json::from_str(&s).expect(&s);
                    match relay_message {
                        RelayMessageV5::Closed(_, _) => todo!(),
                        RelayMessageV5::Event(_, e) => events.push(*e),
                        RelayMessageV5::Notice(s) => println!("NOTICE: {}", s),
                        RelayMessageV5::Eose(_) => {
                            let message = ClientMessage::Close(SubscriptionId(format!(
                                "{:?}/{:?}/{:?}",
                                crate::weeble::weeble_sync(),
                                crate::blockheight::blockheight_sync(),
                                crate::weeble::weeble_sync(),
                            )));
                            let wire = match serde_json::to_string(&message) {
                                Ok(w) => w,
                                Err(e) => {
                                    println!("Could not serialize message: {}", e);
                                    return events;
                                }
                            };
                            if let Err(e) = websocket.send(Message::Text(wire.into())) {
                                println!("Could not write close subscription message: {}", e);
                                return events;
                            }
                            if let Err(e) = websocket.send(Message::Close(None)) {
                                println!("Could not write websocket close message: {}", e);
                                return events;
                            }
                        }
                        RelayMessageV5::Ok(_id, ok, reason) => {
                            println!("OK: ok={} reason={}", ok, reason)
                        }
                        RelayMessageV5::Auth(challenge) => {
                            // NIP-0042 ["AUTH", "<challenge-string>"]
                            print!(r#"["AUTH", "{}"]"#, challenge)
                        }
                        RelayMessageV5::Notify(_) => todo!(),
                    }
                }
                Message::Binary(_) => {
                    println!("IGNORING BINARY MESSAGE")
                }
                Message::Ping(vec) => {
                    if let Err(e) = websocket.send(Message::Pong(vec)) {
                        println!("Unable to pong: {}", e);
                    }
                }
                Message::Pong(_) => {
                    println!("IGNORING PONG")
                }
                Message::Close(_) => {
                    //println!("Closing");
                    break;
                }
                Message::Frame(_) => {
                    println!("UNEXPECTED RAW WEBSOCKET FRAME")
                }
            }
        }

        events
    }

    /// Posts a wire message to a relay.
    pub fn post(host: String, uri: Uri, wire: String) {
        //gnostr key here
        let key: [u8; 16] = rand::random();
        let request = http::request::Request::builder()
            .method("GET")
            .header("Host", host.clone())
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                base64::engine::general_purpose::STANDARD.encode(key),
            )
            .uri(uri)
            .body(())
            .expect("Could not build request");

        let (mut websocket, _response) =
            tungstenite::connect(request).expect("Could not connect to relay");

        print!("{}
", wire);
        websocket
            .send(Message::Text(wire.into()))
            .expect("Could not send message to relay");

        // Get and print one response message

        let message = match websocket.read() {
            Ok(m) => m,
            Err(e) => {
                //handle differently
                println!("Problem reading from websocket: {}", e);
                return;
            }
        };

        match message {
            Message::Text(s) => {
                let relay_message: RelayMessage = serde_json::from_str(&s).expect(&s);
                match relay_message {
                    RelayMessage::Event(_, e) => {
                        println!(r#"["EVENT", {}]"#, serde_json::to_string(&e).unwrap())
                    }
                    RelayMessage::Notice(s) => println!("NOTICE: {}", s),
                    RelayMessage::Eose(_) => println!("EOSE"),
                    //nostr uses json extensively
                    //yet relays dont return json formatted messages?
                    RelayMessage::Ok(_id, ok, reason) => println!(
                        r#"["{}",{{"ok":"{}","reason":"{}"}}]"#,
                        host, ok, reason
                    ),
                    RelayMessage::Auth(challenge) => print!(r#"["AUTH":"{}"]"#, challenge),
                    RelayMessage::Notify(_) => todo!(),
                    RelayMessage::Closed(_, _) => todo!(),
                }
            }
            Message::Binary(_) => {
                println!("IGNORING BINARY MESSAGE")
            }
            Message::Ping(vec) => {
                if let Err(e) = websocket.send(Message::Pong(vec)) {
                    println!("Unable to pong: {}", e);
                }
            }
            Message::Pong(_) => {
                println!("IGNORING PONG")
            }
            Message::Close(_) => {
                println!("Closing");
            }
            Message::Frame(_) => {
                println!("UNEXPECTED RAW WEBSOCKET FRAME")
            }
        }
    }
}


/// Module containing versioned Nostr event and message structures.
pub mod versioned;
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

trait IntoVec<T> {
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
