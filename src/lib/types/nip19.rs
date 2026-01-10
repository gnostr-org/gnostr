// Copyright 2015-2020 nostr-proto Developers
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according to those terms.

//! NIP-19 bech32-encoded entities

#![allow(missing_docs)]

use std::str::FromStr;

use bech32::{self, Bech32, Bech32m, Hrp};

use crate::types::{Error, Id, EventKind, PublicKey, RelayUrl};

/// Different NIP-19 bech32 encoded entity types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nip19 {
    /// Public key (npub)
    PublicKey(PublicKey),
    /// Private key (nsec)
    PrivateKey(String), // We'll store this as a hex string for now
    /// Event Id (note)
    EventId(Id),
    /// Nostr Profile (nprofile)
    Profile(Nip19Profile),
    /// Nostr Event with metadata (nevent)
    Event(Nip19Event),
    /// Nostr Addressable Event (naddr)
    Address(Nip19Address),
    /// Nostr Relay (nrelay) - Deprecated
    Relay(RelayUrl),
}

/// A NIP-19 bech32 profile (`nprofile`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip19Profile {
    /// The public key
    pub public_key: PublicKey,
    /// Relays where the profile may be found
    pub relays: Vec<RelayUrl>,
}

/// A NIP-19 bech32 event (`nevent`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip19Event {
    /// The event ID
    pub event_id: Id,
    /// Author of the event (optional)
    pub author: Option<PublicKey>,
    /// Kind of the event (optional)
    pub kind: Option<EventKind>,
    /// Relays where the event may be found
    pub relays: Vec<RelayUrl>,
}

/// A NIP-19 bech32 addressable event (`naddr`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip19Address {
    /// The event kind
    pub kind: EventKind,
    /// The public key of the author
    pub public_key: PublicKey,
    /// The 'd' tag identifier
    pub identifier: String,
    /// Relays where the event may be found
    pub relays: Vec<RelayUrl>,
}

const TLV_TYPE_SPECIAL: u8 = 0;
const TLV_TYPE_RELAY: u8 = 1;
const TLV_TYPE_AUTHOR: u8 = 2;
const TLV_TYPE_KIND: u8 = 3;

impl Nip19 {
    /// Decode a bech32 encoded NIP-19 string
    pub fn decode(s: &str) -> Result<Self, Error> {
        let (hrp, data) = bech32::decode(s)?;

        match hrp.as_str() {
            "npub" => Ok(Nip19::PublicKey(PublicKey::from_bytes(&data, true)?)),
            "nsec" => {
                if data.len() != 32 { return Err(Error::InvalidPrivateKey); }
                Ok(Nip19::PrivateKey(hex::encode(data)))
            },
            "note" => Ok(Nip19::EventId(Id::try_from_bytes(&data).map_err(|_| Error::InvalidId)?)),
            "nprofile" => {
                let mut public_key = None;
                let mut relays = Vec::new();
                let mut cursor = 0;
                while cursor < data.len() {
                    let t = data[cursor];
                    cursor += 1;
                    let l = data[cursor] as usize;
                    cursor += 1;
                    let v = &data[cursor..cursor + l];
                    cursor += l;

                    match t {
                        TLV_TYPE_SPECIAL => public_key = Some(PublicKey::from_bytes(v, true)?),
                        TLV_TYPE_RELAY => relays.push(RelayUrl::try_from_str(&String::from_utf8(v.to_vec())?)?),
                        _ => {} // Ignore unknown TLV types
                    }
                }
                let public_key = public_key.ok_or(Error::InvalidNip19Data)?;
                Ok(Nip19::Profile(Nip19Profile { public_key, relays }))
            }
            "nevent" => {
                let mut event_id = None;
                let mut author = None;
                let mut kind = None;
                let mut relays = Vec::new();
                let mut cursor = 0;
                while cursor < data.len() {
                    let t = data[cursor];
                    cursor += 1;
                    let l = data[cursor] as usize;
                    cursor += 1;
                    let v = &data[cursor..cursor + l];
                    cursor += l;

                    match t {
                        TLV_TYPE_SPECIAL => event_id = Some(Id::try_from_bytes(v).map_err(|_| Error::InvalidId)?),
                        TLV_TYPE_RELAY => relays.push(RelayUrl::try_from_str(&String::from_utf8(v.to_vec())?)?),
                        TLV_TYPE_AUTHOR => author = Some(PublicKey::from_bytes(v, true)?),
                        TLV_TYPE_KIND => {
                            if v.len() == 4 {
                                let mut bytes = [0u8; 4];
                                bytes.copy_from_slice(v);
                                kind = Some(EventKind::from(u32::from_be_bytes(bytes)));
                            }
                        }
                        _ => {} // Ignore unknown TLV types
                    }
                }
                let event_id = event_id.ok_or(Error::InvalidNip19Data)?;
                Ok(Nip19::Event(Nip19Event {
                    event_id,
                    author,
                    kind,
                    relays,
                }))
            }
            "naddr" => {
                let mut kind = None;
                let mut public_key = None;
                let mut identifier = None;
                let mut relays = Vec::new();
                let mut cursor = 0;
                while cursor < data.len() {
                    let t = data[cursor];
                    cursor += 1;
                    let l = data[cursor] as usize;
                    cursor += 1;
                    let v = &data[cursor..cursor + l];
                    cursor += l;

                    match t {
                        TLV_TYPE_SPECIAL => identifier = Some(String::from_utf8(v.to_vec())?),
                        TLV_TYPE_RELAY => relays.push(RelayUrl::try_from_str(&String::from_utf8(v.to_vec())?)?),
                        TLV_TYPE_AUTHOR => public_key = Some(PublicKey::from_bytes(v, true)?),
                        TLV_TYPE_KIND => {
                            if v.len() == 4 {
                                let mut bytes = [0u8; 4];
                                bytes.copy_from_slice(v);
                                kind = Some(EventKind::from(u32::from_be_bytes(bytes)));
                            }
                        }
                        _ => {} // Ignore unknown TLV types
                    }
                }
                let kind = kind.ok_or(Error::InvalidNip19Data)?;
                let public_key = public_key.ok_or(Error::InvalidNip19Data)?;
                let identifier = identifier.ok_or(Error::InvalidNip19Data)?;
                Ok(Nip19::Address(Nip19Address {
                    kind,
                    public_key,
                    identifier,
                    relays,
                }))
            }
            "nrelay" => Ok(Nip19::Relay(RelayUrl::try_from_str(&String::from_utf8(data)?)?)),
            _ => Err(Error::InvalidNip19Prefix),
        }
    }

    /// Encode a NIP-19 entity into a bech32 string
    pub fn encode(&self) -> Result<String, Error> {
        match self {
            Nip19::PublicKey(pk) => bech32::encode::<Bech32>(Hrp::parse("npub")?, pk.as_bytes()).map_err(|e| e.into()),
            Nip19::PrivateKey(sk_hex) => {
                let sk_bytes = hex::decode(sk_hex)?;
                bech32::encode::<Bech32>(Hrp::parse("nsec")?, &sk_bytes).map_err(|e| e.into())
            },
            Nip19::EventId(id) => bech32::encode::<Bech32>(Hrp::parse("note")?, id.0.as_slice()).map_err(|e| e.into()),
            Nip19::Profile(profile) => {
                let mut data = Vec::new();
                // Special: Public Key
                data.push(TLV_TYPE_SPECIAL);
                data.push(profile.public_key.as_bytes().len() as u8);
                data.extend_from_slice(profile.public_key.as_bytes());
                // Relays
                for relay in &profile.relays {
                    let relay_bytes = relay.as_str().as_bytes();
                    data.push(TLV_TYPE_RELAY);
                    data.push(relay_bytes.len() as u8);
                    data.extend_from_slice(relay_bytes);
                }
                bech32::encode::<Bech32>(Hrp::parse("nprofile")?, &data).map_err(|e| e.into())
            }
            Nip19::Event(event) => {
                let mut data = Vec::new();
                // Special: Event ID
                data.push(TLV_TYPE_SPECIAL);
                data.push(event.event_id.0.as_slice().len() as u8);
                data.extend_from_slice(event.event_id.0.as_slice());
                // Author
                if let Some(author) = &event.author {
                    data.push(TLV_TYPE_AUTHOR);
                    data.push(author.as_bytes().len() as u8);
                    data.extend_from_slice(author.as_bytes());
                }
                // Kind
                if let Some(kind) = &event.kind {
                    let kind_bytes = u32::from(*kind).to_be_bytes();
                    data.push(TLV_TYPE_KIND);
                    data.push(kind_bytes.len() as u8);
                    data.extend_from_slice(&kind_bytes);
                }
                // Relays
                for relay in &event.relays {
                    let relay_bytes = relay.as_str().as_bytes();
                    data.push(TLV_TYPE_RELAY);
                    data.push(relay_bytes.len() as u8);
                    data.extend_from_slice(relay_bytes);
                }
                bech32::encode::<Bech32>(Hrp::parse("nevent")?, &data).map_err(|e| e.into())
            }
            Nip19::Address(addr) => {
                let mut data = Vec::new();
                // Special: Identifier
                let identifier_bytes = addr.identifier.as_bytes();
                data.push(TLV_TYPE_SPECIAL);
                data.push(identifier_bytes.len() as u8);
                data.extend_from_slice(identifier_bytes);
                // Kind
                let kind_bytes = u32::from(addr.kind).to_be_bytes();
                data.push(TLV_TYPE_KIND);
                data.push(kind_bytes.len() as u8);
                data.extend_from_slice(&kind_bytes);
                // Author
                data.push(TLV_TYPE_AUTHOR);
                data.push(addr.public_key.as_bytes().len() as u8);
                data.extend_from_slice(addr.public_key.as_bytes());
                // Relays
                for relay in &addr.relays {
                    let relay_bytes = relay.as_str().as_bytes();
                    data.push(TLV_TYPE_RELAY);
                    data.push(relay_bytes.len() as u8);
                    data.extend_from_slice(relay_bytes);
                }
                bech32::encode::<Bech32>(Hrp::parse("naddr")?, &data).map_err(|e| e.into())
            }
            Nip19::Relay(relay_url) => {
                bech32::encode::<Bech32>(Hrp::parse("nrelay")?, relay_url.as_str().as_bytes()).map_err(|e| e.into())
            }
        }
    }
}

