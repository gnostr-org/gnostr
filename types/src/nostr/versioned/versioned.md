# versioned Code Documentation

**Generated on:** 2026-01-21 13:49:15
**Directory:** /Users/Shared/gnostr-org/.github/gnostr/src/lib/types/versioned
**Files included:** 21

---

## Directory Structure

```
./client_message1.rs
./client_message2.rs
./client_message3.rs
./event1.rs
./event2.rs
./event3.rs
./metadata.rs
./mod.rs
./nip05.rs
./relay_information_document1.rs
./relay_information_document2.patch
./relay_information_document2.rs
./relay_list.rs
./relay_message1.rs
./relay_message2.rs
./relay_message3.rs
./relay_message4.rs
./relay_message5.rs
./tag1.rs
./tag2.rs
./tag3.rs
./versioned.md
./zap_data.rs
```

---

## File Contents

### client_message1.rs

**Size:** 4312 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV1;
use crate::types::{Filter, SubscriptionId};

/// A message from a client to a relay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum ClientMessageV1 {
    /// An event
    Event(Box<EventV1>),

    /// A subscription request
    Req(SubscriptionId, Vec<Filter>),

    /// A request to close a subscription
    Close(SubscriptionId),

    /// Used to send authentication events
    Auth(Box<EventV1>),
}

impl Serialize for ClientMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessageV1::Event(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            ClientMessageV1::Req(id, filters) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(&id)?;
                for filter in filters {
                    seq.serialize_element(&filter)?;
                }
                seq.end()
            }
            ClientMessageV1::Close(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            ClientMessageV1::Auth(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ClientMessageV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessageV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<ClientMessageV1, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<ClientMessageV1> = None;
        if word == "EVENT" {
            let event: EventV1 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV1::Event(Box::new(event)))
        } else if word == "REQ" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let mut filters: Vec<Filter> = vec![];
            loop {
                let f: Option<Filter> = seq.next_element()?;
                match f {
                    None => break,
                    Some(fil) => filters.push(fil),
                }
            }
            output = Some(ClientMessageV1::Req(id, filters))
        } else if word == "CLOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(ClientMessageV1::Close(id))
        } else if word == "AUTH" {
            let event: EventV1 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV1::Auth(Box::new(event)))
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(cm) => Ok(cm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}
```

---

### client_message2.rs

**Size:** 4608 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV2;
use crate::types::{Filter, SubscriptionId};

/// A message from a client to a relay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum ClientMessageV2 {
    /// An event
    Event(Box<EventV2>),

    /// A subscription request
    Req(SubscriptionId, Vec<Filter>),

    /// A request to close a subscription
    Close(SubscriptionId),

    /// Used to send authentication events
    Auth(Box<EventV2>),
}

impl ClientMessageV2 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> ClientMessageV2 {
        ClientMessageV2::Event(Box::new(EventV2::mock()))
    }
}

impl Serialize for ClientMessageV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessageV2::Event(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            ClientMessageV2::Req(id, filters) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(&id)?;
                for filter in filters {
                    seq.serialize_element(&filter)?;
                }
                seq.end()
            }
            ClientMessageV2::Close(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            ClientMessageV2::Auth(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ClientMessageV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessageV2;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<ClientMessageV2, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<ClientMessageV2> = None;
        if word == "EVENT" {
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV2::Event(Box::new(event)))
        } else if word == "REQ" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let mut filters: Vec<Filter> = vec![];
            loop {
                let f: Option<Filter> = seq.next_element()?;
                match f {
                    None => break,
                    Some(fil) => filters.push(fil),
                }
            }
            output = Some(ClientMessageV2::Req(id, filters))
        } else if word == "CLOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(ClientMessageV2::Close(id))
        } else if word == "AUTH" {
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV2::Auth(Box::new(event)))
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(cm) => Ok(cm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {ClientMessageV2, test_client_message_serde}
}
```

---

### client_message3.rs

**Size:** 4608 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV3;
use crate::types::{Filter, SubscriptionId};

/// A message from a client to a relay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum ClientMessageV3 {
    /// An event
    Event(Box<EventV3>),

    /// A subscription request
    Req(SubscriptionId, Vec<Filter>),

    /// A request to close a subscription
    Close(SubscriptionId),

    /// Used to send authentication events
    Auth(Box<EventV3>),
}

impl ClientMessageV3 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> ClientMessageV3 {
        ClientMessageV3::Event(Box::new(EventV3::mock()))
    }
}

impl Serialize for ClientMessageV3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessageV3::Event(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            ClientMessageV3::Req(id, filters) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(&id)?;
                for filter in filters {
                    seq.serialize_element(&filter)?;
                }
                seq.end()
            }
            ClientMessageV3::Close(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            ClientMessageV3::Auth(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ClientMessageV3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessageV3;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<ClientMessageV3, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<ClientMessageV3> = None;
        if word == "EVENT" {
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV3::Event(Box::new(event)))
        } else if word == "REQ" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let mut filters: Vec<Filter> = vec![];
            loop {
                let f: Option<Filter> = seq.next_element()?;
                match f {
                    None => break,
                    Some(fil) => filters.push(fil),
                }
            }
            output = Some(ClientMessageV3::Req(id, filters))
        } else if word == "CLOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(ClientMessageV3::Close(id))
        } else if word == "AUTH" {
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV3::Auth(Box::new(event)))
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(cm) => Ok(cm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {ClientMessageV3, test_client_message_serde}
}
```

---

### event1.rs

**Size:** 38768 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::str::FromStr;

use lightning_invoice::Bolt11Invoice;
#[cfg(feature = "speedy")]
use regex::Regex;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    id::{self, Id},
    Error, EventDelegation, EventKind, EventReference, IntoVec, MilliSatoshi, NAddr, NostrBech32,
    NostrUrl, PublicKey, PublicKeyHex, RelayUrl, Signature, TagV1, Unixtime, ZapDataV1,
};

/// The main event type
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EventV1 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The signature of the event, which cryptographically verifies that the
    /// holder of the PrivateKey matching the event's PublicKey generated
    /// (or authorized) this event. The signature is taken over the id field
    /// only, but the id field is taken over the rest of the event data.
    pub sig: Signature,

    /// DEPRECATED (please set to Null): An optional verified time for the event
    /// (using OpenTimestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ots: Option<String>,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV1>,
}

macro_rules! serialize_inner_event {
    ($pubkey:expr, $created_at:expr, $kind:expr, $tags:expr,
     $content:expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?
        )
    }};
}

/// Data used to construct an event
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PreEventV1 {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<TagV1>,
    /// The content of the event
    pub content: String,
}

impl PreEventV1 {
    /// Generate an ID from this PreEvent for use in an Event or a Rumor
    pub fn hash(&self) -> Result<Id, Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Hash
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        Ok(Id(id))
    }
}

/// A Rumor is an Event without a signature
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RumorV1 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV1>,
}

impl RumorV1 {
    /// Create a new rumor
    pub fn new(input: PreEventV1) -> Result<RumorV1, Error> {
        // Generate Id
        let id = input.hash()?;

        Ok(RumorV1 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
        })
    }

    /// Turn into an Event (the signature will be all zeroes)
    pub fn into_event_with_bad_signature(self) -> EventV1 {
        EventV1 {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            sig: Signature::zeroes(),
            ots: None,
            content: self.content,
            tags: self.tags,
        }
    }
}

impl EventV1 {
    /// Check the validity of an event. This is useful if you deserialize an
    /// event from the network. If you create an event using new() it should
    /// already be trustworthy.
    pub fn verify(&self, maxtime: Option<Unixtime>) -> Result<(), Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Verify the signature
        self.pubkey.verify(serialized.as_bytes(), &self.sig)?;

        // Also verify the ID is the SHA256
        // (the above verify function also does it internally,
        //  so there is room for improvement here)
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();

        // Optional verify that the message was in the past
        if let Some(mt) = maxtime {
            if self.created_at > mt {
                return Err(Error::EventInFuture);
            }
        }

        if id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    /// Get the k-tag kind, if any
    pub fn k_tag_kind(&self) -> Option<EventKind> {
        for tag in self.tags.iter() {
            if let TagV1::Kind { kind, .. } = tag {
                return Some(*kind);
            }
        }
        None
    }

    /// If the event refers to people by tag, get all the PublicKeys it refers
    /// to along with recommended relay URL and petname for each
    pub fn people(&self) -> Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> {
        let mut output: Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> = Vec::new();
        // All 'p' tags
        for tag in self.tags.iter() {
            if let TagV1::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                ..
            } = tag
            {
                output.push((
                    pubkey.to_owned(),
                    recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                    petname.to_owned(),
                ));
            }
        }

        output
    }

    /// If the pubkey is tagged in the event
    pub fn is_tagged(&self, pubkey: &PublicKey) -> bool {
        let pkh: PublicKeyHex = pubkey.into();

        for tag in self.tags.iter() {
            if let TagV1::Pubkey { pubkey, .. } = tag {
                if *pubkey == pkh {
                    return true;
                }
            }
        }

        false
    }

    /// If the event refers to people within the contents, get all the
    /// PublicKeys it refers to within the contents.
    pub fn people_referenced_in_content(&self) -> Vec<PublicKey> {
        let mut output = Vec::new();
        for nurl in NostrUrl::find_all_in_string(&self.content).drain(..) {
            if let NostrBech32::Pubkey(pk) = nurl.0 {
                output.push(pk);
            }
            if let NostrBech32::Profile(prof) = nurl.0 {
                output.push(prof.pubkey);
            }
        }
        output
    }

    /// All events IDs that this event refers to, whether root, reply, mention,
    /// or otherwise along with optional recommended relay URLs
    pub fn referred_events(&self) -> Vec<EventReference> {
        let mut output: Vec<EventReference> = Vec::new();

        // Collect every 'e' tag and 'a' tag
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                output.push(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.clone(),
                });
            } else if let TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    output.push(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        output
    }

    /// Get a reference to another event that this event replies to.
    /// An event can only reply to one other event via 'e' or 'a' tag from a
    /// feed-displayable event that is not a Repost.
    pub fn replies_to(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // Repost 'e' and 'a' tags are always considered mentions, not replies.
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            return None;
        }

        // If there are no 'e' tags nor 'a' tags, then none
        let num_event_ref_tags = self
            .tags
            .iter()
            .filter(|e| matches!(e, TagV1::Event { .. }) || matches!(e, TagV1::Address { .. }))
            .count();
        if num_event_ref_tags == 0 {
            return None;
        }

        // look for an 'e' tag with marker 'reply'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // Use the last unmarked 'e' tag or any 'a' tag
        if let Some(tag) = self.tags.iter().rev().find(|t| {
            matches!(t, TagV1::Event { marker: None, .. }) || matches!(t, TagV1::Address { .. })
        }) {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.to_owned(),
                });
            } else if let TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event replies to a thread, get that threads root event Id if
    /// available, along with an optional recommended_relay_url
    pub fn replies_to_root(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                }
            }
        }

        // otherwise use the first unmarked 'e' tag or first 'a' tag
        // (even if there is only 1 'e' or 'a' tag which means it is both root and
        // reply)
        if let Some(tag) = self.tags.iter().find(|t| {
            matches!(t, TagV1::Event { marker: None, .. }) || matches!(t, TagV1::Address { .. })
        }) {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.to_owned(),
                });
            } else if let TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event mentions others, get those other event Ids
    /// and optional recommended relay Urls
    pub fn mentions(&self) -> Vec<EventReference> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        // For kind=6 and kind=16, all 'e' and 'a' tags are mentions
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            for tag in self.tags.iter() {
                if let TagV1::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                } else if let TagV1::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    trailing: _,
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }

            return output;
        }

        // Look for nostr links within the content

        // Collect every 'e' tag marked as 'mention'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "mention" {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // Collect every unmarked 'e' or 'a' tag that is not the first (root) or the
        // last (reply)
        let e_tags: Vec<&TagV1> = self
            .tags
            .iter()
            .filter(|e| {
                matches!(e, TagV1::Event { marker: None, .. }) || matches!(e, TagV1::Address { .. })
            })
            .collect();
        if e_tags.len() > 2 {
            // mentions are everything other than first and last
            for tag in &e_tags[1..e_tags.len() - 1] {
                if let TagV1::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                } else if let TagV1::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    ..
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }
        }

        output
    }

    /// If this event reacts to another, get that other event's Id,
    /// the reaction content, and an optional Recommended relay Url
    pub fn reacts_to(&self) -> Option<(Id, String, Option<RelayUrl>)> {
        if self.kind != EventKind::Reaction {
            return None;
        }

        // The last 'e' tag is it
        if let Some(TagV1::Event {
            id,
            recommended_relay_url,
            ..
        }) = self
            .tags
            .iter()
            .rev()
            .find(|t| matches!(t, TagV1::Event { .. }))
        {
            return Some((
                *id,
                self.content.clone(),
                recommended_relay_url
                    .as_ref()
                    .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
            ));
        }

        None
    }

    /// If this event deletes others, get all the Ids of the events that it
    /// deletes along with the reason for the deletion
    pub fn deletes(&self) -> Option<(Vec<Id>, String)> {
        if self.kind != EventKind::EventDeletion {
            return None;
        }

        let mut ids: Vec<Id> = Vec::new();

        // All 'e' tags are deleted
        for tag in self.tags.iter() {
            if let TagV1::Event { id, .. } = tag {
                ids.push(*id);
            }
        }

        if ids.is_empty() {
            None
        } else {
            Some((ids, self.content.clone()))
        }
    }

    /// If this event zaps another event, get data about that.
    ///
    /// That includes the Id, the amount, and the public key of the provider,
    /// all of which should be verified by the caller.
    ///
    /// Errors returned from this are not fatal, but may be useful for
    /// explaining to a user why a zap receipt is invalid.
    pub fn zaps(&self) -> Result<Option<ZapDataV1>, Error> {
        if self.kind != EventKind::Zap {
            return Ok(None);
        }

        let mut zapped_id: Option<Id> = None;
        let mut zapped_amount: Option<MilliSatoshi> = None;
        let mut zapped_pubkey: Option<PublicKey> = None;

        for tag in self.tags.iter() {
            if let TagV1::Other { tag, data } = tag {
                // Find the bolt11 tag
                if tag != "bolt11" {
                    continue;
                }
                if data.is_empty() {
                    return Err(Error::ZapReceipt("missing bolt11 tag value".to_string()));
                }

                // Extract as an Invoice
                let result = Bolt11Invoice::from_str(&data[0]);
                if let Err(e) = result {
                    return Err(Error::ZapReceipt(format!("bolt11 failed to parse: {}", e)));
                }
                let invoice = result.unwrap();

                // Verify the signature
                if let Err(e) = invoice.check_signature() {
                    return Err(Error::ZapReceipt(format!(
                        "bolt11 signature check failed: {}",
                        e
                    )));
                }

                // Get the public key
                let secpk = match invoice.payee_pub_key() {
                    Some(pubkey) => pubkey.to_owned(),
                    None => invoice.recover_payee_pub_key(),
                };
                let (xonlypk, _) = secpk.x_only_public_key();
                let pubkeybytes = xonlypk.serialize();
                let pubkey = match PublicKey::from_bytes(&pubkeybytes, false) {
                    Ok(pubkey) => pubkey,
                    Err(e) => {
                        return Err(Error::ZapReceipt(format!("payee public key error: {}", e)));
                    }
                };
                zapped_pubkey = Some(pubkey);

                if let Some(u) = invoice.amount_milli_satoshis() {
                    zapped_amount = Some(MilliSatoshi(u));
                } else {
                    return Err(Error::ZapReceipt(
                        "Amount missing from zap receipt".to_string(),
                    ));
                }
            }
            if let TagV1::Event { id, .. } = tag {
                zapped_id = Some(*id);
            }
        }

        if zapped_id.is_none() {
            // This probably means a person was zapped, not a note. So not an error.
            return Ok(None);
        }
        if zapped_amount.is_none() {
            return Err(Error::ZapReceipt("Missing amount".to_string()));
        }
        if zapped_pubkey.is_none() {
            return Err(Error::ZapReceipt("Missing payee public key".to_string()));
        }

        Ok(Some(ZapDataV1 {
            id: zapped_id.unwrap(),
            amount: zapped_amount.unwrap(),
            pubkey: zapped_pubkey.unwrap(),
            provider_pubkey: self.pubkey,
        }))
    }

    /// If this event specifies the client that created it, return that client
    /// string
    pub fn client(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Other { tag, data } = tag {
                if tag == "client" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a subject, return that subject string
    pub fn subject(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Subject { subject, .. } = tag {
                return Some(subject.clone());
            }
        }

        None
    }

    /// If this event specifies a title, return that title string
    pub fn title(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Title { title, .. } = tag {
                return Some(title.clone());
            }
        }

        None
    }

    /// If this event specifies a summary, return that summary string
    pub fn summary(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Other { tag, data } = tag {
                if tag == "summary" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a content warning, return that subject string
    pub fn content_warning(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::ContentWarning { warning, .. } = tag {
                return Some(warning.clone());
            }
        }

        None
    }

    /// If this is a parameterized event, get the parameter
    pub fn parameter(&self) -> Option<String> {
        if self.kind.is_parameterized_replaceable() {
            for tag in self.tags.iter() {
                if let TagV1::Identifier { d, .. } = tag {
                    return Some(d.to_owned());
                }
            }
            Some("".to_owned()) // implicit
        } else {
            None
        }
    }

    /// Return all the hashtags this event refers to
    pub fn hashtags(&self) -> Vec<String> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<String> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV1::Hashtag { hashtag, .. } = tag {
                output.push(hashtag.clone());
            }
        }

        output
    }

    /// Return all the URLs this event refers to
    pub fn urls(&self) -> Vec<RelayUrl> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<RelayUrl> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV1::Reference { url, .. } = tag {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(url) {
                    output.push(relay_url);
                }
            }
        }

        output
    }

    /// Get the proof-of-work count of leading bits
    pub fn pow(&self) -> u8 {
        // Count leading bits in the Id field
        let zeroes: u8 = crate::types::get_leading_zero_bits(&self.id.0);

        // Check that they meant it
        let mut target_zeroes: u8 = 0;
        for tag in self.tags.iter() {
            if let TagV1::Nonce { target, .. } = tag {
                if let Some(t) = target {
                    target_zeroes = t.parse::<u8>().unwrap_or(0);
                }
                break;
            }
        }

        zeroes.min(target_zeroes)
    }

    /// Was this event delegated, was that valid, and if so what is the pubkey
    /// of the delegator?
    pub fn delegation(&self) -> EventDelegation {
        for tag in self.tags.iter() {
            if let TagV1::Delegation {
                pubkey,
                conditions,
                sig,
                ..
            } = tag
            {
                // Convert hex strings into functional types
                let signature = match Signature::try_from_hex_string(sig) {
                    Ok(sig) => sig,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };
                let delegator_pubkey = match PublicKey::try_from_hex_string(pubkey, true) {
                    Ok(pk) => pk,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };

                // Verify the delegation tag
                match conditions.verify_signature(&delegator_pubkey, &self.pubkey, &signature) {
                    Ok(_) => {
                        // Check conditions
                        if let Some(kind) = conditions.kind {
                            if self.kind != kind {
                                return EventDelegation::InvalidDelegation(
                                    "Event Kind not delegated".to_owned(),
                                );
                            }
                        }
                        if let Some(created_after) = conditions.created_after {
                            if self.created_at < created_after {
                                return EventDelegation::InvalidDelegation(
                                    "Event created before delegation started".to_owned(),
                                );
                            }
                        }
                        if let Some(created_before) = conditions.created_before {
                            if self.created_at > created_before {
                                return EventDelegation::InvalidDelegation(
                                    "Event created after delegation ended".to_owned(),
                                );
                            }
                        }
                        return EventDelegation::DelegatedBy(delegator_pubkey);
                    }
                    Err(e) => {
                        return EventDelegation::InvalidDelegation(format!("{e}"));
                    }
                }
            }
        }

        EventDelegation::NotDelegated
    }

    /// If the event came through a proxy, get the (Protocol, Id)
    pub fn proxy(&self) -> Option<(&str, &str)> {
        for t in self.tags.iter() {
            if let TagV1::Other { tag, data } = t {
                if tag == "proxy" && data.len() >= 2 {
                    return Some((&data[1], &data[0]));
                }
            }
        }
        None
    }
}

// Direct access into speedy-serialized bytes, to avoid alloc-deserialize just
// to peek at one of these fields
#[cfg(feature = "speedy")]
impl EventV1 {
    /// Read the ID of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_id_from_speedy_bytes(bytes: &[u8]) -> Option<Id> {
        if bytes.len() < 32 {
            None
        } else if let Ok(arr) = <[u8; 32]>::try_from(&bytes[0..32]) {
            use crate::types::id;

            Some(unsafe { std::mem::transmute::<[u8; 32], id::Id>(arr) })
        } else {
            None
        }
    }

    /// Read the pubkey of the event from a speedy encoding without decoding
    /// (close to zero allocation, VerifyingKey does stuff I didn't check)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_pubkey_from_speedy_bytes(bytes: &[u8]) -> Option<PublicKey> {
        if bytes.len() < 64 {
            None
        } else {
            PublicKey::from_bytes(&bytes[32..64], false).ok()
        }
    }

    /// Read the created_at of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_created_at_from_speedy_bytes(bytes: &[u8]) -> Option<Unixtime> {
        if bytes.len() < 72 {
            None
        } else if let Ok(i) = i64::read_from_buffer(&bytes[64..72]) {
            Some(Unixtime(i))
        } else {
            None
        }
    }

    /// Read the kind of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_kind_from_speedy_bytes(bytes: &[u8]) -> Option<EventKind> {
        if bytes.len() < 76 {
            None
        } else if let Ok(u) = u32::read_from_buffer(&bytes[72..76]) {
            Some(u.into())
        } else {
            None
        }
    }

    // Read the sig of the event from a speedy encoding without decoding
    // (offset would be 76..140

    /// Read the ots of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_ots_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        #[allow(clippy::if_same_then_else)]
        if bytes.len() < 140 {
            None
        } else if bytes[140] == 0 {
            None
        } else if bytes.len() < 145 {
            None
        } else {
            let len = u32::from_ne_bytes(bytes[141..145].try_into().unwrap());
            unsafe {
                Some(std::str::from_utf8_unchecked(
                    &bytes[146..146 + len as usize],
                ))
            }
        }
    }

    /// Read the content of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_content_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        let start = if bytes.len() < 145 {
            return None;
        } else if bytes[140] == 0 {
            141
        } else {
            // get OTS length and move past it
            let len = u32::from_ne_bytes(bytes[141..145].try_into().unwrap());
            141 + 4 + len as usize
        };

        let len = u32::from_ne_bytes(bytes[start..start + 4].try_into().unwrap());

        unsafe {
            Some(std::str::from_utf8_unchecked(
                &bytes[start + 4..start + 4 + len as usize],
            ))
        }
    }

    /// Check if any human-readable tag matches the Regex in the speedy encoding
    /// without decoding the whole thing (because our Tag representation is so
    /// complicated, we do deserialize the tags for now)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn tag_search_in_speedy_bytes(bytes: &[u8], re: &Regex) -> Result<bool, Error> {
        if bytes.len() < 145 {
            return Ok(false);
        }

        // skip OTS
        let mut offset = if bytes[140] == 0 {
            141
        } else {
            // get OTS length and move past it
            let len = u32::from_ne_bytes(bytes[141..145].try_into().unwrap());
            141 + 4 + len as usize
        };

        // skip content
        let len = u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4 + len as usize;

        // Deserialize the tags
        let tags: Vec<TagV1> = Vec::<TagV1>::read_from_buffer(&bytes[offset..])?;

        // Search through them
        for tag in &tags {
            match tag {
                TagV1::ContentWarning { warning, .. } => {
                    if re.is_match(warning.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Hashtag { hashtag, .. } => {
                    if re.is_match(hashtag.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Subject { subject, .. } => {
                    if re.is_match(subject.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Title { title, .. } => {
                    if re.is_match(title.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Other { tag, data } => {
                    if tag == "summary" && !data.is_empty() && re.is_match(data[0].as_ref()) {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}

impl From<EventV1> for RumorV1 {
    fn from(e: EventV1) -> RumorV1 {
        RumorV1 {
            id: e.id,
            pubkey: e.pubkey,
            created_at: e.created_at,
            kind: e.kind,
            content: e.content,
            tags: e.tags,
        }
    }
}

impl From<RumorV1> for PreEventV1 {
    fn from(r: RumorV1) -> PreEventV1 {
        PreEventV1 {
            pubkey: r.pubkey,
            created_at: r.created_at,
            kind: r.kind,
            content: r.content,
            tags: r.tags,
        }
    }
}

impl TryFrom<PreEventV1> for RumorV1 {
    type Error = Error;
    fn try_from(e: PreEventV1) -> Result<RumorV1, Error> {
        RumorV1::new(e)
    }
}
```

---

### event2.rs

**Size:** 49277 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::str::FromStr;

use lightning_invoice::Bolt11Invoice;
#[cfg(feature = "speedy")]
use regex::Regex;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    id::{self, Id},
    Error, EventDelegation, EventKind, EventReference, IntoVec, KeySigner, MilliSatoshi, NAddr,
    NostrBech32, NostrUrl, PrivateKey, PublicKey, PublicKeyHex, RelayUrl, Signature, Signer, TagV2,
    Unixtime, ZapDataV1,
};

/// The main event type
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EventV2 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The signature of the event, which cryptographically verifies that the
    /// holder of the PrivateKey matching the event's PublicKey generated
    /// (or authorized) this event. The signature is taken over the id field
    /// only, but the id field is taken over the rest of the event data.
    pub sig: Signature,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV2>,
}

macro_rules! serialize_inner_event {
    ($pubkey:expr, $created_at:expr, $kind:expr, $tags:expr,
     $content:expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?
        )
    }};
}

/// Data used to construct an event
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PreEventV2 {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<TagV2>,
    /// The content of the event
    pub content: String,
}

impl PreEventV2 {
    /// Generate an ID from this PreEvent for use in an Event or a Rumor
    pub fn hash(&self) -> Result<Id, Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Hash
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        Ok(Id(id))
    }
}

/// A Rumor is an Event without a signature
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RumorV2 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV2>,
}

impl RumorV2 {
    /// Create a new rumor
    pub fn new(input: PreEventV2) -> Result<RumorV2, Error> {
        // Generate Id
        let id = input.hash()?;

        Ok(RumorV2 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
        })
    }

    /// Turn into an Event (the signature will be all zeroes)
    pub fn into_event_with_bad_signature(self) -> EventV2 {
        EventV2 {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            sig: Signature::zeroes(),
            content: self.content,
            tags: self.tags,
        }
    }
}

impl EventV2 {
    /// Check the validity of an event. This is useful if you deserialize an
    /// event from the network. If you create an event using new() it should
    /// already be trustworthy.
    pub fn verify(&self, maxtime: Option<Unixtime>) -> Result<(), Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Verify the signature
        self.pubkey.verify(serialized.as_bytes(), &self.sig)?;

        // Also verify the ID is the SHA256
        // (the above verify function also does it internally,
        //  so there is room for improvement here)
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();

        // Optional verify that the message was in the past
        if let Some(mt) = maxtime {
            if self.created_at > mt {
                return Err(Error::EventInFuture);
            }
        }

        if id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    /// Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> EventV2 {
        let signer = {
            let private_key = PrivateKey::mock();
            KeySigner::from_private_key(private_key, "", 1).unwrap()
        };
        let public_key = signer.public_key();
        let pre = PreEventV2 {
            pubkey: public_key,
            created_at: Unixtime::mock(),
            kind: EventKind::mock(),
            tags: vec![TagV2::mock(), TagV2::mock()],
            content: "This is a test".to_string(),
        };
        signer.sign_event2(pre).unwrap()
    }

    /// Get the k-tag kind, if any
    pub fn k_tag_kind(&self) -> Option<EventKind> {
        for tag in self.tags.iter() {
            if let TagV2::Kind { kind, .. } = tag {
                return Some(*kind);
            }
        }
        None
    }

    /// If the event refers to people by tag, get all the PublicKeys it refers
    /// to along with recommended relay URL and petname for each
    pub fn people(&self) -> Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> {
        let mut output: Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> = Vec::new();
        // All 'p' tags
        for tag in self.tags.iter() {
            if let TagV2::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                ..
            } = tag
            {
                output.push((
                    pubkey.to_owned(),
                    recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                    petname.to_owned(),
                ));
            }
        }

        output
    }

    /// If the pubkey is tagged in the event
    pub fn is_tagged(&self, pubkey: &PublicKey) -> bool {
        let pkh: PublicKeyHex = pubkey.into();

        for tag in self.tags.iter() {
            if let TagV2::Pubkey { pubkey, .. } = tag {
                if *pubkey == pkh {
                    return true;
                }
            }
        }

        false
    }

    /// If the event refers to people within the contents, get all the
    /// PublicKeys it refers to within the contents.
    pub fn people_referenced_in_content(&self) -> Vec<PublicKey> {
        let mut output = Vec::new();
        for nurl in NostrUrl::find_all_in_string(&self.content).drain(..) {
            if let NostrBech32::Pubkey(pk) = nurl.0 {
                output.push(pk);
            }
            if let NostrBech32::Profile(prof) = nurl.0 {
                output.push(prof.pubkey);
            }
        }
        output
    }

    /// All events IDs that this event refers to, whether root, reply, mention,
    /// or otherwise along with optional recommended relay URLs
    pub fn referred_events(&self) -> Vec<EventReference> {
        let mut output: Vec<EventReference> = Vec::new();

        // Collect every 'e' tag and 'a' tag
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                output.push(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.clone(),
                });
            } else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    output.push(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        output
    }

    /// Get a reference to another event that this event replies to.
    /// An event can only reply to one other event via 'e' or 'a' tag from a
    /// feed-displayable event that is not a Repost.
    pub fn replies_to(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // Repost 'e' and 'a' tags are always considered mentions, not replies.
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            return None;
        }

        // If there are no 'e' tags nor 'a' tags, then none
        let num_event_ref_tags = self
            .tags
            .iter()
            .filter(|e| matches!(e, TagV2::Event { .. }) || matches!(e, TagV2::Address { .. }))
            .count();
        if num_event_ref_tags == 0 {
            return None;
        }

        // look for an 'e' tag with marker 'reply'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                }
            }
        }

        // Use the last unmarked 'e' tag or any 'a' tag
        if let Some(tag) = self.tags.iter().rev().find(|t| {
            matches!(t, TagV2::Event { marker: None, .. })
                || matches!(t, TagV2::Address { marker: None, .. })
        }) {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.clone(),
                });
            } else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event replies to a thread, get that threads root event Id if
    /// available, along with an optional recommended_relay_url
    pub fn replies_to_root(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                }
            }
        }

        // otherwise use the first unmarked 'e' tag or first 'a' tag
        // (even if there is only 1 'e' or 'a' tag which means it is both root and
        // reply)
        if let Some(tag) = self.tags.iter().find(|t| {
            matches!(t, TagV2::Event { marker: None, .. })
                || matches!(t, TagV2::Address { marker: None, .. })
        }) {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.to_owned(),
                });
            } else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event mentions others, get those other event Ids
    /// and optional recommended relay Urls
    pub fn mentions(&self) -> Vec<EventReference> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        // For kind=6 and kind=16, all 'e' and 'a' tags are mentions
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            for tag in self.tags.iter() {
                if let TagV2::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                } else if let TagV2::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    marker: _,
                    trailing: _,
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }

            return output;
        }

        // Look for nostr links within the content

        // Collect every 'e' tag marked as 'mention'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "mention" {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // Collect every unmarked 'e' or 'a' tag that is not the first (root) or the
        // last (reply)
        let e_tags: Vec<&TagV2> = self
            .tags
            .iter()
            .filter(|e| {
                matches!(e, TagV2::Event { marker: None, .. })
                    || matches!(e, TagV2::Address { marker: None, .. })
            })
            .collect();
        if e_tags.len() > 2 {
            // mentions are everything other than first and last
            for tag in &e_tags[1..e_tags.len() - 1] {
                if let TagV2::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                } else if let TagV2::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    ..
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }
        }

        output
    }

    /// If this event reacts to another, get that other event's Id,
    /// the reaction content, and an optional Recommended relay Url
    pub fn reacts_to(&self) -> Option<(Id, String, Option<RelayUrl>)> {
        if self.kind != EventKind::Reaction {
            return None;
        }

        // The last 'e' tag is it
        if let Some(TagV2::Event {
            id,
            recommended_relay_url,
            ..
        }) = self
            .tags
            .iter()
            .rev()
            .find(|t| matches!(t, TagV2::Event { .. }))
        {
            return Some((
                *id,
                self.content.clone(),
                recommended_relay_url
                    .as_ref()
                    .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
            ));
        }

        None
    }

    /// If this event deletes others, get all the EventReferences of the events
    /// that it deletes along with the reason for the deletion
    pub fn deletes(&self) -> Option<(Vec<EventReference>, String)> {
        if self.kind != EventKind::EventDeletion {
            return None;
        }

        let mut erefs: Vec<EventReference> = Vec::new();

        for tag in self.tags.iter() {
            // All 'e' tags are deleted
            if let TagV2::Event { id, .. } = tag {
                erefs.push(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: vec![],
                    marker: None,
                });
            }
            // All 'a' tag groups are deleted
            else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url,
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey, false) {
                    let ea = NAddr {
                        d: d.clone(),
                        relays: match relay_url {
                            Some(url) => vec![url.clone()],
                            None => vec![],
                        },
                        kind: *kind,
                        author: pk,
                    };
                    erefs.push(EventReference::Addr(ea));
                }
            }
        }

        if erefs.is_empty() {
            None
        } else {
            Some((erefs, self.content.clone()))
        }
    }

    /// Can this event be deleted by the given public key?
    pub fn delete_author_allowed(&self, by: PublicKey) -> bool {
        // Author can always delete
        if self.pubkey == by {
            return true;
        }

        if self.kind == EventKind::GiftWrap {
            for tag in self.tags.iter() {
                if let TagV2::Pubkey { pubkey, .. } = tag {
                    let pkh: PublicKeyHex = by.into();
                    return pkh == *pubkey;
                }
            }
        }

        false
    }

    /// If this event zaps another event, get data about that.
    ///
    /// That includes the Id, the amount, and the public key of the provider,
    /// all of which should be verified by the caller.
    ///
    /// Errors returned from this are not fatal, but may be useful for
    /// explaining to a user why a zap receipt is invalid.
    pub fn zaps(&self) -> Result<Option<ZapDataV1>, Error> {
        if self.kind != EventKind::Zap {
            return Ok(None);
        }

        let mut zapped_id: Option<Id> = None;
        let mut zapped_amount: Option<MilliSatoshi> = None;
        let mut zapped_pubkey: Option<PublicKey> = None;

        for tag in self.tags.iter() {
            if let TagV2::Other { tag, data } = tag {
                // Find the bolt11 tag
                if tag != "bolt11" {
                    continue;
                }
                if data.is_empty() {
                    return Err(Error::ZapReceipt("missing bolt11 tag value".to_string()));
                }

                // Extract as an Invoice
                let result = Bolt11Invoice::from_str(&data[0]);
                if let Err(e) = result {
                    return Err(Error::ZapReceipt(format!("bolt11 failed to parse: {}", e)));
                }
                let invoice = result.unwrap();

                // Verify the signature
                if let Err(e) = invoice.check_signature() {
                    return Err(Error::ZapReceipt(format!(
                        "bolt11 signature check failed: {}",
                        e
                    )));
                }

                // Get the public key
                let secpk = match invoice.payee_pub_key() {
                    Some(pubkey) => pubkey.to_owned(),
                    None => invoice.recover_payee_pub_key(),
                };
                let (xonlypk, _) = secpk.x_only_public_key();
                let pubkeybytes = xonlypk.serialize();
                let pubkey = match PublicKey::from_bytes(&pubkeybytes, false) {
                    Ok(pubkey) => pubkey,
                    Err(e) => {
                        return Err(Error::ZapReceipt(format!("payee public key error: {}", e)));
                    }
                };
                zapped_pubkey = Some(pubkey);

                if let Some(u) = invoice.amount_milli_satoshis() {
                    zapped_amount = Some(MilliSatoshi(u));
                } else {
                    return Err(Error::ZapReceipt(
                        "Amount missing from zap receipt".to_string(),
                    ));
                }
            }
            if let TagV2::Event { id, .. } = tag {
                zapped_id = Some(*id);
            }
        }

        if zapped_id.is_none() {
            // This probably means a person was zapped, not a note. So not an error.
            return Ok(None);
        }
        if zapped_amount.is_none() {
            return Err(Error::ZapReceipt("Missing amount".to_string()));
        }
        if zapped_pubkey.is_none() {
            return Err(Error::ZapReceipt("Missing payee public key".to_string()));
        }

        Ok(Some(ZapDataV1 {
            id: zapped_id.unwrap(),
            amount: zapped_amount.unwrap(),
            pubkey: zapped_pubkey.unwrap(),
            provider_pubkey: self.pubkey,
        }))
    }

    /// If this event specifies the client that created it, return that client
    /// string
    pub fn client(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Other { tag, data } = tag {
                if tag == "client" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a subject, return that subject string
    pub fn subject(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Subject { subject, .. } = tag {
                return Some(subject.clone());
            }
        }

        None
    }

    /// If this event specifies a title, return that title string
    pub fn title(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Title { title, .. } = tag {
                return Some(title.clone());
            }
        }

        None
    }

    /// If this event specifies a summary, return that summary string
    pub fn summary(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Other { tag, data } = tag {
                if tag == "summary" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a content warning, return that subject string
    pub fn content_warning(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::ContentWarning { warning, .. } = tag {
                return Some(warning.clone());
            }
        }

        None
    }

    /// If this is a parameterized event, get the parameter
    pub fn parameter(&self) -> Option<String> {
        if self.kind.is_parameterized_replaceable() {
            for tag in self.tags.iter() {
                if let TagV2::Identifier { d, .. } = tag {
                    return Some(d.to_owned());
                }
            }
            Some("".to_owned()) // implicit
        } else {
            None
        }
    }

    /// Return all the hashtags this event refers to
    pub fn hashtags(&self) -> Vec<String> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<String> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV2::Hashtag { hashtag, .. } = tag {
                output.push(hashtag.clone());
            }
        }

        output
    }

    /// Return all the URLs this event refers to
    pub fn urls(&self) -> Vec<RelayUrl> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<RelayUrl> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV2::Reference { url, .. } = tag {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(url) {
                    output.push(relay_url);
                }
            }
        }

        output
    }

    /// Get the proof-of-work count of leading bits
    pub fn pow(&self) -> u8 {
        // Count leading bits in the Id field
        let zeroes: u8 = crate::types::get_leading_zero_bits(&self.id.0);

        // Check that they meant it
        let mut target_zeroes: u8 = 0;
        for tag in self.tags.iter() {
            if let TagV2::Nonce { target, .. } = tag {
                if let Some(t) = target {
                    target_zeroes = t.parse::<u8>().unwrap_or(0);
                }
                break;
            }
        }

        zeroes.min(target_zeroes)
    }

    /// Was this event delegated, was that valid, and if so what is the pubkey
    /// of the delegator?
    pub fn delegation(&self) -> EventDelegation {
        for tag in self.tags.iter() {
            if let TagV2::Delegation {
                pubkey,
                conditions,
                sig,
                ..
            } = tag
            {
                // Convert hex strings into functional types
                let signature = match Signature::try_from_hex_string(sig) {
                    Ok(sig) => sig,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };
                let delegator_pubkey = match PublicKey::try_from_hex_string(pubkey, true) {
                    Ok(pk) => pk,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };

                // Verify the delegation tag
                match conditions.verify_signature(&delegator_pubkey, &self.pubkey, &signature) {
                    Ok(_) => {
                        // Check conditions
                        if let Some(kind) = conditions.kind {
                            if self.kind != kind {
                                return EventDelegation::InvalidDelegation(
                                    "Event Kind not delegated".to_owned(),
                                );
                            }
                        }
                        if let Some(created_after) = conditions.created_after {
                            if self.created_at < created_after {
                                return EventDelegation::InvalidDelegation(
                                    "Event created before delegation started".to_owned(),
                                );
                            }
                        }
                        if let Some(created_before) = conditions.created_before {
                            if self.created_at > created_before {
                                return EventDelegation::InvalidDelegation(
                                    "Event created after delegation ended".to_owned(),
                                );
                            }
                        }
                        return EventDelegation::DelegatedBy(delegator_pubkey);
                    }
                    Err(e) => {
                        return EventDelegation::InvalidDelegation(format!("{e}"));
                    }
                }
            }
        }

        EventDelegation::NotDelegated
    }

    /// If the event came through a proxy, get the (Protocol, Id)
    pub fn proxy(&self) -> Option<(&str, &str)> {
        for t in self.tags.iter() {
            if let TagV2::Other { tag, data } = t {
                if tag == "proxy" && data.len() >= 2 {
                    return Some((&data[1], &data[0]));
                }
            }
        }
        None
    }
}

// Direct access into speedy-serialized bytes, to avoid alloc-deserialize just
// to peek at one of these fields
#[cfg(feature = "speedy")]
impl EventV2 {
    /// Read the ID of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_id_from_speedy_bytes(bytes: &[u8]) -> Option<Id> {
        if bytes.len() < 32 {
            None
        } else if let Ok(arr) = <[u8; 32]>::try_from(&bytes[0..32]) {
            Some(unsafe { std::mem::transmute::<[u8; 32], id::Id>(arr) })
        } else {
            None
        }
    }

    /// Read the pubkey of the event from a speedy encoding without decoding
    /// (close to zero allocation, VerifyingKey does stuff I didn't check)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_pubkey_from_speedy_bytes(bytes: &[u8]) -> Option<PublicKey> {
        if bytes.len() < 64 {
            None
        } else {
            PublicKey::from_bytes(&bytes[32..64], false).ok()
        }
    }

    /// Read the created_at of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_created_at_from_speedy_bytes(bytes: &[u8]) -> Option<Unixtime> {
        if bytes.len() < 72 {
            None
        } else if let Ok(i) = i64::read_from_buffer(&bytes[64..72]) {
            Some(Unixtime(i))
        } else {
            None
        }
    }

    /// Read the kind of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_kind_from_speedy_bytes(bytes: &[u8]) -> Option<EventKind> {
        if bytes.len() < 76 {
            None
        } else if let Ok(u) = u32::read_from_buffer(&bytes[72..76]) {
            Some(u.into())
        } else {
            None
        }
    }

    // Read the sig of the event from a speedy encoding without decoding
    // (offset would be 76..140

    /// Read the content of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_content_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());

        unsafe {
            Some(std::str::from_utf8_unchecked(
                &bytes[140 + 4..140 + 4 + len as usize],
            ))
        }
    }

    /// Check if any human-readable tag matches the Regex in the speedy encoding
    /// without decoding the whole thing (because our TagV2 representation is so
    /// complicated, we do deserialize the tags for now)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn tag_search_in_speedy_bytes(bytes: &[u8], re: &Regex) -> Result<bool, Error> {
        if bytes.len() < 140 {
            return Ok(false);
        }

        // skip content
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());
        let offset = 140 + 4 + len as usize;

        // Deserialize the tags
        let tags: Vec<TagV2> = Vec::<TagV2>::read_from_buffer(&bytes[offset..])?;

        // Search through them
        for tag in &tags {
            match tag {
                TagV2::ContentWarning { warning, .. } => {
                    if re.is_match(warning.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Hashtag { hashtag, .. } => {
                    if re.is_match(hashtag.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Subject { subject, .. } => {
                    if re.is_match(subject.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Title { title, .. } => {
                    if re.is_match(title.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Other { tag, data } => {
                    if tag == "summary" && !data.is_empty() && re.is_match(data[0].as_ref()) {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}

impl From<EventV2> for RumorV2 {
    fn from(e: EventV2) -> RumorV2 {
        RumorV2 {
            id: e.id,
            pubkey: e.pubkey,
            created_at: e.created_at,
            kind: e.kind,
            content: e.content,
            tags: e.tags,
        }
    }
}

impl From<RumorV2> for PreEventV2 {
    fn from(r: RumorV2) -> PreEventV2 {
        PreEventV2 {
            pubkey: r.pubkey,
            created_at: r.created_at,
            kind: r.kind,
            content: r.content,
            tags: r.tags,
        }
    }
}

impl TryFrom<PreEventV2> for RumorV2 {
    type Error = Error;
    fn try_from(e: PreEventV2) -> Result<RumorV2, Error> {
        RumorV2::new(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{DelegationConditions, Signer, UncheckedUrl};

    test_serde! {EventV2, test_event_serde}

    #[test]
    fn test_event_new_and_verify() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };
        let pubkey = signer.public_key();
        let preevent = PreEventV2 {
            pubkey,
            created_at: Unixtime::mock(),
            kind: EventKind::TextNote,
            tags: vec![TagV2::Event {
                id: Id::mock(),
                recommended_relay_url: Some(UncheckedUrl::mock()),
                marker: None,
                trailing: Vec::new(),
            }],
            content: "Hello World!".to_string(),
        };
        let mut event = signer.sign_event2(preevent).unwrap();
        assert!(event.verify(None).is_ok());

        // Now make sure it fails when the message has been modified
        event.content = "I'm changing this message".to_string();
        let result = event.verify(None);
        assert!(result.is_err());

        // Change it back
        event.content = "Hello World!".to_string();
        let result = event.verify(None);
        assert!(result.is_ok());

        // Tweak the id only
        event.id = Id([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let result = event.verify(None);
        assert!(result.is_err());
    }

    // helper
    fn create_event_with_delegation<S>(created_at: Unixtime, real_signer: &S) -> EventV2
    where
        S: Signer,
    {
        let delegated_signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let conditions = DelegationConditions::try_from_str(
            "kind=1&created_at>1680000000&created_at<1680050000",
        )
        .unwrap();

        let sig = real_signer
            .generate_delegation_signature(delegated_signer.public_key(), &conditions)
            .unwrap();

        let preevent = PreEventV2 {
            pubkey: delegated_signer.public_key(),
            created_at,
            kind: EventKind::TextNote,
            tags: vec![
                TagV2::Event {
                    id: Id::mock(),
                    recommended_relay_url: Some(UncheckedUrl::mock()),
                    marker: None,
                    trailing: Vec::new(),
                },
                TagV2::Delegation {
                    pubkey: PublicKeyHex::try_from_string(real_signer.public_key().as_hex_string())
                        .unwrap(),
                    conditions,
                    sig: sig.into(),
                    trailing: Vec::new(),
                },
            ],
            content: "Hello World!".to_string(),
        };
        delegated_signer.sign_event2(preevent).unwrap()
    }

    #[test]
    fn test_event_with_delegation_ok() {
        let delegator_signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };
        let delegator_pubkey = delegator_signer.public_key();

        let event = create_event_with_delegation(Unixtime(1680000012), &delegator_signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::DelegatedBy(pk) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(pk, delegator_pubkey);
        } else {
            panic!("Expected DelegatedBy result, got {:?}", event.delegation());
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_after() {
        let delegator_privkey = PrivateKey::mock();
        let signer = KeySigner::from_private_key(delegator_privkey, "", 1).unwrap();

        let event = create_event_with_delegation(Unixtime(1690000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created after delegation ended");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_before() {
        let signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };

        let event = create_event_with_delegation(Unixtime(1610000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created before delegation started");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_realworld_event_with_naddr_tag() {
        let raw = r##"{"id":"7760408f6459b9546c3a4e70e3e56756421fba34526b7d460db3fcfd2f8817db","pubkey":"460c25e682fda7832b52d1f22d3d22b3176d972f60dcdc3212ed8c92ef85065c","created_at":1687616920,"kind":1,"tags":[["p","1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411","","mention"],["a","30311:1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411:1687612774","","mention"]],"content":"Watching Karnage's stream to see if I learn something about design. \n\nnostr:naddr1qq9rzd3cxumrzv3hxu6qygqmcu9qzj9n7vtd5vl78jyly037wxkyl7vcqflvwy4eqhxjfa4yzypsgqqqwens0qfplk","sig":"dbc5d05a24bfe990a1faaedfcb81a98940d86a105711dbdad9145d05b0ad0f46e3e24eaa3fc283818f27e057fe836a029fd9a68e7f1de06ff477493199d64064"}"##;
        let _: EventV2 = serde_json::from_str(&raw).unwrap();
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_encoded_direct_field_access() {
        use speedy::Writable;

        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let preevent = PreEventV2 {
            pubkey: signer.public_key(),
            created_at: Unixtime(1680000012),
            kind: EventKind::TextNote,
            tags: vec![
                TagV2::Event {
                    id: Id::mock(),
                    recommended_relay_url: Some(UncheckedUrl::mock()),
                    marker: None,
                    trailing: Vec::new(),
                },
                TagV2::Hashtag {
                    hashtag: "foodstr".to_string(),
                    trailing: Vec::new(),
                },
            ],
            content: "Hello World!".to_string(),
        };
        let event = signer.sign_event2(preevent).unwrap();
        let bytes = event.write_to_vec().unwrap();

        let id = EventV2::get_id_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(id, event.id);

        let pubkey = EventV2::get_pubkey_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(pubkey, event.pubkey);

        let created_at = EventV2::get_created_at_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(created_at, Unixtime(1680000012));

        let kind = EventV2::get_kind_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(kind, event.kind);

        let content = EventV2::get_content_from_speedy_bytes(&bytes);
        assert_eq!(content, Some(&*event.content));

        let re = regex::Regex::new("foodstr").unwrap();
        let found_foodstr = EventV2::tag_search_in_speedy_bytes(&bytes, &re).unwrap();
        assert!(found_foodstr);

        // Print to work out encoding
        //   test like this to see printed data:
        //   cargo test --features=speedy test_speedy_encoded_direct_field_access --
        // --nocapture
        println!("EVENT BYTES: {:?}", bytes);
        println!("ID: {:?}", event.id.0);
        println!("PUBKEY: {:?}", event.pubkey.as_slice());
        println!("CREATED AT: {:?}", event.created_at.0.to_ne_bytes());
        let kind32: u32 = event.kind.into();
        println!("KIND: {:?}", kind32.to_ne_bytes());
        println!("SIG: {:?}", event.sig.0.as_ref());
        println!(
            "CONTENT: [len={:?}] {:?}",
            (event.content.as_bytes().len() as u32).to_ne_bytes(),
            event.content.as_bytes()
        );
        println!("TAGS: [len={:?}]", (event.tags.len() as u32).to_ne_bytes());
    }

    #[test]
    fn test_event_gift_wrap() {
        let signer1 = {
            let sec1 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap();
            KeySigner::from_private_key(sec1, "", 1).unwrap()
        };

        let signer2 = {
            let sec2 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap();
            KeySigner::from_private_key(sec2, "", 1).unwrap()
        };

        let pre = PreEventV2 {
            pubkey: signer1.public_key(),
            created_at: Unixtime(1692_000_000),
            kind: EventKind::TextNote,
            content: "Hey man, this rocks! Please reply for a test.".to_string(),
            tags: vec![],
        };

        let gift_wrap = signer1
            .giftwrap2(pre.clone(), signer2.public_key())
            .unwrap();
        let rumor = signer2.unwrap_giftwrap2(&gift_wrap).unwrap();
        let output_pre: PreEventV2 = rumor.into();

        assert_eq!(pre, output_pre);
    }
}
```

---

### event3.rs

**Size:** 53771 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{cmp::Ordering, fmt, str::FromStr};

use lightning_invoice::Bolt11Invoice;
#[cfg(feature = "speedy")]
use regex::Regex;
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    id::{self, Id},
    Error, EventDelegation, EventKind, EventReference, IntoVec, KeySecurity, KeySigner,
    MilliSatoshi, NostrBech32, NostrUrl, PrivateKey, PublicKey, RelayUrl, Signature, Signer, TagV3,
    Unixtime, ZapData,
};

/// The main event type
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EventV3 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The signature of the event, which cryptographically verifies that the
    /// holder of the PrivateKey matching the event's PublicKey generated
    /// (or authorized) this event. The signature is taken over the id field
    /// only, but the id field is taken over the rest of the event data.
    pub sig: Signature,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV3>,
}

macro_rules! serialize_inner_event {
    ($pubkey:expr, $created_at:expr, $kind:expr, $tags:expr,
     $content:expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?
        )
    }};
}

/// Data used to construct an event
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PreEventV3 {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<TagV3>,
    /// The content of the event
    pub content: String,
}

impl PreEventV3 {
    /// Generate an ID from this PreEvent for use in an Event or a Rumor
    pub fn hash(&self) -> Result<Id, Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Hash
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        Ok(Id(id))
    }
}

/// A Rumor is an Event without a signature
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RumorV3 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV3>,
}

impl RumorV3 {
    /// Create a new rumor
    pub fn new(input: PreEventV3) -> Result<RumorV3, Error> {
        // Generate Id
        let id = input.hash()?;

        Ok(RumorV3 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
        })
    }

    /// Turn into an Event (the signature will be all zeroes)
    pub fn into_event_with_bad_signature(self) -> EventV3 {
        EventV3 {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            sig: Signature::zeroes(),
            content: self.content,
            tags: self.tags,
        }
    }
}

impl fmt::Display for EventV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Event {{ id: {}, pubkey: {}, kind: {}, created_at: {}, content: {}... }}",
            self.id.as_hex_string(),
            self.pubkey.as_hex_string(),
            u32::from(self.kind),
            self.created_at.0,
            &self.content[..self.content.len().min(50)] // Truncate content for display
        )
    }
}

impl EventV3 {
    /// Create a dummy event for testing or placeholder purposes.
    #[allow(dead_code)]
    pub fn new_dummy() -> Self {
        Self {
            id: Id::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            pubkey: PublicKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000000",
                false,
            )
            .unwrap(), // pubkey of all zeroes
            created_at: Unixtime(0),
            kind: EventKind::TextNote,
            sig: Signature::zeroes(),
            content: "Dummy event content".to_string(),
            tags: Vec::new(),
        }
    }

    /// Sign a `PreEventV3` with the provided `PrivateKey` and return an
    /// `EventV3`.
    pub fn sign_with_private_key(
        preevent: PreEventV3,
        private_key: &PrivateKey,
    ) -> Result<Self, Error> {
        let id = preevent.hash()?;
        let signer = KeySigner::from_private_key(private_key.clone(), "", 1)?;
        let sig = signer.sign_id(id)?;

        Ok(EventV3 {
            id,
            pubkey: preevent.pubkey,
            created_at: preevent.created_at,
            kind: preevent.kind,
            tags: preevent.tags,
            content: preevent.content,
            sig,
        })
    }

    /// Check the validity of an event. This is useful if you deserialize an
    /// event from the network. If you create an event using new() it should
    /// already be trustworthy.
    pub fn verify(&self, maxtime: Option<Unixtime>) -> Result<(), Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Verify the signature
        self.pubkey.verify(serialized.as_bytes(), &self.sig)?;
        // Also verify the ID is the SHA256
        // (the above verify function also does it internally,
        //  so there is room for improvement here)
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();

        // Optional verify that the message was in the past
        if let Some(mt) = maxtime {
            if self.created_at > mt {
                return Err(Error::EventInFuture);
            }
        }

        if id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    /// Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> EventV3 {
        let signer = {
            let private_key = PrivateKey::mock();
            KeySigner::from_private_key(private_key, "", 1).unwrap()
        };
        let public_key = signer.public_key();
        let pre = PreEventV3 {
            pubkey: public_key,
            created_at: Unixtime::mock(),
            kind: EventKind::mock(),
            tags: vec![TagV3::mock(), TagV3::mock()],
            content: "This is a test".to_string(),
        };
        let id = pre.hash().unwrap();
        let sig = signer.sign_id(id).unwrap();
        EventV3 {
            id,
            pubkey: pre.pubkey,
            created_at: pre.created_at,
            kind: pre.kind,
            tags: pre.tags,
            content: pre.content,
            sig,
        }
    }

    /// Get the k-tag kind, if any
    pub fn k_tag_kind(&self) -> Option<EventKind> {
        for tag in self.tags.iter() {
            if let Ok(kind) = tag.parse_kind() {
                return Some(kind);
            }
        }
        None
    }

    /// If the event refers to people by tag, get all the PublicKeys it refers
    /// to along with recommended relay URL and petname for each
    pub fn people(&self) -> Vec<(PublicKey, Option<RelayUrl>, Option<String>)> {
        let mut output: Vec<(PublicKey, Option<RelayUrl>, Option<String>)> = Vec::new();
        // All 'p' tags
        for tag in self.tags.iter() {
            if let Ok((pubkey, recommended_relay_url, petname)) = tag.parse_pubkey() {
                output.push((
                    pubkey.to_owned(),
                    recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                    petname.to_owned(),
                ));
            }
        }

        output
    }

    /// If the pubkey is tagged in the event
    pub fn is_tagged(&self, pk: &PublicKey) -> bool {
        for tag in self.tags.iter() {
            if let Ok((pubkey, _recommended_relay_url, _petname)) = tag.parse_pubkey() {
                if pubkey == *pk {
                    return true;
                }
            }
        }

        false
    }

    /// If the event refers to people within the contents, get all the
    /// PublicKeys it refers to within the contents.
    pub fn people_referenced_in_content(&self) -> Vec<PublicKey> {
        let mut output = Vec::new();
        for nurl in NostrUrl::find_all_in_string(&self.content).drain(..) {
            if let NostrBech32::Pubkey(pk) = nurl.0 {
                output.push(pk);
            }
            if let NostrBech32::Profile(prof) = nurl.0 {
                output.push(prof.pubkey);
            }
        }
        output
    }

    /// All events IDs that this event refers to, whether root, reply, mention,
    /// or otherwise along with optional recommended relay URLs
    pub fn referred_events(&self) -> Vec<EventReference> {
        let mut output: Vec<EventReference> = Vec::new();

        // Collect every 'e' tag and 'a' tag
        for tag in self.tags.iter() {
            if let Ok((id, rurl, marker)) = tag.parse_event() {
                output.push(EventReference::Id {
                    id,
                    author: None,
                    relays: rurl
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker,
                });
            } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                output.push(EventReference::Addr(ea))
            }
        }

        output
    }

    /// Get a reference to another event that this event replies to.
    /// An event can only reply to one other event via 'e' or 'a' tag from a
    /// feed-displayable event that is not a Repost.
    pub fn replies_to(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // Repost 'e' and 'a' tags are always considered mentions, not replies.
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            return None;
        }

        // If there are no 'e' tags nor 'a' tags, then none
        let num_event_ref_tags = self
            .tags
            .iter()
            .filter(|t| t.tagname() == "e" || t.tagname() == "a")
            .count();
        if num_event_ref_tags == 0 {
            return None;
        }

        // look for an 'e' tag with marker 'reply'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, marker)) = tag.parse_event() {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker,
                    });
                }
            }
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, marker)) = tag.parse_event() {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker,
                    });
                }
            }
        }

        // look for an 'a' tag marked 'reply'
        for tag in self.tags.iter() {
            if let Ok((ea, marker)) = tag.parse_address() {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Addr(ea));
                };
            }
        }

        // look for an 'a' tag marked 'root'
        for tag in self.tags.iter() {
            if let Ok((ea, marker)) = tag.parse_address() {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Addr(ea));
                };
            }
        }

        // Use the last unmarked 'e' or 'a' tag (whichever is last)
        for tag in self.tags.iter().rev() {
            if tag.tagname() == "e" {
                if let Ok((id, rurl, marker)) = tag.parse_event() {
                    if marker.is_none() {
                        return Some(EventReference::Id {
                            id,
                            author: None,
                            relays: rurl
                                .as_ref()
                                .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                                .into_vec(),
                            marker: None,
                        });
                    }
                }
            } else if tag.tagname() == "a" {
                if let Ok((ea, marker)) = tag.parse_address() {
                    if marker.is_some() && marker.as_deref().unwrap() == "root" {
                        return Some(EventReference::Addr(ea));
                    };
                }
            }
        }

        None
    }

    /// If this event replies to a thread, get that threads root event Id if
    /// available, along with an optional recommended_relay_url
    pub fn replies_to_root(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                if optmarker.is_some() && optmarker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                }
            }
        }

        for tag in self.tags.iter() {
            if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                if optmarker.is_none() {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: None,
                    });
                }
            } else if let Ok((ea, optmarker)) = tag.parse_address() {
                if optmarker.is_none() {
                    return Some(EventReference::Addr(ea));
                }
            }
        }

        None
    }

    /// If this event quotes others, get those other events
    pub fn quotes(&self) -> Vec<EventReference> {
        if self.kind != EventKind::TextNote {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok((id, rurl)) = tag.parse_quote() {
                output.push(EventReference::Id {
                    id,
                    author: None,
                    relays: rurl
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: None,
                });
            }
        }

        output
    }

    /// If this event mentions others, get those other event Ids
    /// and optional recommended relay Urls
    pub fn mentions(&self) -> Vec<EventReference> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        // For kind=6 and kind=16, all 'e' and 'a' tags are mentions
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            for tag in self.tags.iter() {
                if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                    output.push(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                    output.push(EventReference::Addr(ea));
                }
            }

            return output;
        }

        // Look for nostr links within the content

        // Collect every 'e' tag marked as 'mention'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                if optmarker.is_some() && optmarker.as_deref().unwrap() == "mention" {
                    output.push(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                }
            }
        }

        // Collect every unmarked 'e' or 'a' tag that is not the first (root) or the
        // last (reply)
        let e_tags: Vec<&TagV3> = self
            .tags
            .iter()
            .filter(|t| (t.tagname() == "e" || t.tagname() == "a") && t.marker() == "")
            .collect();
        if e_tags.len() > 2 {
            // mentions are everything other than first and last
            for tag in &e_tags[1..e_tags.len() - 1] {
                if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                    output.push(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                    output.push(EventReference::Addr(ea));
                }
            }
        }

        output
    }

    /// If this event reacts to another, get that other event's Id,
    /// the reaction content, and an optional Recommended relay Url
    pub fn reacts_to(&self) -> Option<(Id, String, Option<RelayUrl>)> {
        if self.kind != EventKind::Reaction {
            return None;
        }

        // The last 'e' tag is it
        for tag in self.tags.iter().rev() {
            if let Ok((id, rurl, _optmarker)) = tag.parse_event() {
                return Some((
                    id,
                    self.content.clone(),
                    rurl.as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                ));
            }
        }

        None
    }

    /// If this event deletes others, get all the EventReferences of the events
    /// that it deletes along with the reason for the deletion
    pub fn deletes(&self) -> Option<(Vec<EventReference>, String)> {
        if self.kind != EventKind::EventDeletion {
            return None;
        }

        let mut erefs: Vec<EventReference> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok((id, _rurl, _optmarker)) = tag.parse_event() {
                // All 'e' tags are deleted
                erefs.push(EventReference::Id {
                    id,
                    author: None,
                    relays: vec![],
                    marker: None,
                });
            } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                erefs.push(EventReference::Addr(ea));
            }
        }

        if erefs.is_empty() {
            None
        } else {
            Some((erefs, self.content.clone()))
        }
    }

    /// Can this event be deleted by the given public key?
    pub fn delete_author_allowed(&self, by: PublicKey) -> bool {
        // Author can always delete
        if self.pubkey == by {
            return true;
        }

        if self.kind == EventKind::GiftWrap {
            for tag in self.tags.iter() {
                if let Ok((pk, _, _)) = tag.parse_pubkey() {
                    return by == pk;
                }
            }
        }

        false
    }

    /// If this event zaps another event, get data about that.
    ///
    /// Errors returned from this are not fatal, but may be useful for
    /// explaining to a user why a zap receipt is invalid.
    pub fn zaps(&self) -> Result<Option<ZapData>, Error> {
        if self.kind != EventKind::Zap {
            return Ok(None);
        }

        let (zap_request, bolt11invoice, payer_p_tag, target_event_from_tags): (
            EventV3,
            Bolt11Invoice,
            Option<PublicKey>,
            Option<EventReference>,
        ) = {
            let mut zap_request: Option<EventV3> = None;
            let mut bolt11invoice: Option<Bolt11Invoice> = None;
            let mut payer_p_tag: Option<PublicKey> = None;
            let mut target_event_from_tags: Option<EventReference> = None;

            for tag in self.tags.iter() {
                if tag.tagname() == "description" {
                    let request_string = tag.value();
                    if let Ok(e) = serde_json::from_str::<EventV3>(request_string) {
                        zap_request = Some(e);
                    }
                }
                // we ignore the "p" tag, we have that data from two other places (invoice and
                // request)
                else if tag.tagname() == "P" {
                    if let Ok((pk, _, _)) = tag.parse_pubkey() {
                        payer_p_tag = Some(pk);
                    }
                } else if tag.tagname() == "bolt11" {
                    if tag.value() == "" {
                        return Err(Error::ZapReceipt("missing bolt11 tag value".to_string()));
                    }

                    // Extract as an Invoice
                    let invoice = match Bolt11Invoice::from_str(tag.value()) {
                        Ok(inv) => inv,
                        Err(e) => {
                            return Err(Error::ZapReceipt(format!(
                                "bolt11 failed to parse: {}",
                                e
                            )));
                        }
                    };

                    // Verify the signature
                    if let Err(e) = invoice.check_signature() {
                        return Err(Error::ZapReceipt(format!(
                            "bolt11 signature check failed: {}",
                            e
                        )));
                    }

                    bolt11invoice = Some(invoice);
                }
            }

            let re = self.referred_events();
            if re.len() == 1 {
                target_event_from_tags = Some(re[0].clone());
            }

            // "The zap receipt MUST contain a description tag which is the JSON-encoded zap
            // request."
            if zap_request.is_none() {
                return Ok(None);
            }
            let zap_request = zap_request.unwrap();

            // "The zap receipt MUST have a bolt11 tag containing the description hash
            // bolt11 invoice."
            if bolt11invoice.is_none() {
                return Ok(None);
            }
            let bolt11invoice = bolt11invoice.unwrap();

            (
                zap_request,
                bolt11invoice,
                payer_p_tag,
                target_event_from_tags,
            )
        };

        // Extract data from the invoice
        let (payee_from_invoice, amount_from_invoice): (PublicKey, MilliSatoshi) = {
            // Get the public key
            let secpk = match bolt11invoice.payee_pub_key() {
                Some(pubkey) => pubkey.to_owned(),
                None => bolt11invoice.recover_payee_pub_key(),
            };
            let (xonlypk, _) = secpk.x_only_public_key();
            let pubkeybytes = xonlypk.serialize();
            let pubkey = match PublicKey::from_bytes(&pubkeybytes, false) {
                Ok(pubkey) => pubkey,
                Err(e) => return Err(Error::ZapReceipt(format!("payee public key error: {}", e))),
            };

            if let Some(u) = bolt11invoice.amount_milli_satoshis() {
                (pubkey, MilliSatoshi(u))
            } else {
                return Err(Error::ZapReceipt(
                    "Amount missing from zap receipt".to_string(),
                ));
            }
        };

        // Extract data from request
        let (payer_from_request, amount_from_request, target_event_from_request): (
            PublicKey,
            MilliSatoshi,
            EventReference,
        ) = {
            let mut amount_from_request: Option<MilliSatoshi> = None;
            let mut target_event_from_request: Option<EventReference> = None;
            for tag in zap_request.tags.iter() {
                if tag.tagname() == "amount" {
                    if let Ok(m) = tag.value().parse::<u64>() {
                        amount_from_request = Some(MilliSatoshi(m));
                    }
                }
            }

            let re = zap_request.referred_events();
            if re.len() == 1 {
                target_event_from_request = Some(re[0].clone());
            }

            if amount_from_request.is_none() {
                return Err(Error::ZapReceipt("zap request had no amount".to_owned()));
            }
            let amount_from_request = amount_from_request.unwrap();

            if target_event_from_request.is_none() {
                return Ok(None);
            }
            let target_event_from_request = target_event_from_request.unwrap();

            (
                zap_request.pubkey,
                amount_from_request,
                target_event_from_request,
            )
        };

        if let Some(p) = payer_p_tag {
            if p != payer_from_request {
                return Err(Error::ZapReceipt(
                    "Payer Mismatch between receipt P-tag and invoice".to_owned(),
                ));
            }
        }

        if amount_from_invoice != amount_from_request {
            return Err(Error::ZapReceipt(
                "Amount Mismatch between request and invoice".to_owned(),
            ));
        }

        if let Some(te) = target_event_from_tags {
            if te != target_event_from_request {
                return Err(Error::ZapReceipt(
                    "Zapped event Mismatch receipt and request".to_owned(),
                ));
            }
        }

        Ok(Some(ZapData {
            zapped_event: target_event_from_request,
            amount: amount_from_invoice,
            payee: payee_from_invoice,
            payer: payer_from_request,
            provider_pubkey: self.pubkey,
        }))
    }

    /// If this event specifies the client that created it, return that client
    /// string
    pub fn client(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if tag.tagname() == "client" && !tag.value().is_empty() {
                return Some(tag.value().to_owned());
            }
        }

        None
    }

    /// If this event specifies a subject, return that subject string
    pub fn subject(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let Ok(subject) = tag.parse_subject() {
                return Some(subject);
            }
        }

        None
    }

    /// If this event specifies a title, return that title string
    pub fn title(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let Ok(title) = tag.parse_title() {
                return Some(title);
            }
        }

        None
    }

    /// If this event specifies a summary, return that summary string
    pub fn summary(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let Ok(summary) = tag.parse_summary() {
                return Some(summary);
            }
        }

        None
    }

    /// Is this event an annotation
    pub fn is_annotation(&self) -> bool {
        for tag in self.tags.iter() {
            if tag.get_index(0) == "annotation" {
                return true;
            }
        }
        false
    }

    /// If this event specifies a content warning, return that content warning
    pub fn content_warning(&self) -> Option<Option<String>> {
        for tag in self.tags.iter() {
            if let Ok(optcontentwarning) = tag.parse_content_warning() {
                return Some(optcontentwarning);
            }
        }

        None
    }

    /// If this is a parameterized event, get the parameter
    pub fn parameter(&self) -> Option<String> {
        if self.kind.is_parameterized_replaceable() {
            for tag in self.tags.iter() {
                if let Ok(ident) = tag.parse_identifier() {
                    return Some(ident);
                }
            }
            Some("".to_owned()) // implicit
        } else {
            None
        }
    }

    /// Return all the hashtags this event refers to
    pub fn hashtags(&self) -> Vec<String> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<String> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok(hashtag) = tag.parse_hashtag() {
                output.push(hashtag);
            }
        }

        output
    }

    /// Return all the URLs this event refers to
    pub fn urls(&self) -> Vec<RelayUrl> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<RelayUrl> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok((url, _optusage)) = tag.parse_relay() {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(&url) {
                    output.push(relay_url);
                }
            }
        }

        output
    }

    /// Get the proof-of-work count of leading bits
    pub fn pow(&self) -> u8 {
        // Count leading bits in the Id field
        let zeroes: u8 = crate::types::get_leading_zero_bits(&self.id.0);

        // Check that they meant it
        for tag in self.tags.iter() {
            if let Ok((_nonce, Some(target))) = tag.parse_nonce() {
                let target_zeroes = target as u8;
                return zeroes.min(target_zeroes);
            }
        }

        0
    }

    /// Was this event delegated, was that valid, and if so what is the pubkey
    /// of the delegator?
    pub fn delegation(&self) -> EventDelegation {
        for tag in self.tags.iter() {
            if let Ok((pk, conditions, sig)) = tag.parse_delegation() {
                // Verify the delegation tag
                match conditions.verify_signature(&pk, &self.pubkey, &sig) {
                    Ok(_) => {
                        // Check conditions
                        if let Some(kind) = conditions.kind {
                            if self.kind != kind {
                                return EventDelegation::InvalidDelegation(
                                    "Event Kind not delegated".to_owned(),
                                );
                            }
                        }
                        if let Some(created_after) = conditions.created_after {
                            if self.created_at < created_after {
                                return EventDelegation::InvalidDelegation(
                                    "Event created before delegation started".to_owned(),
                                );
                            }
                        }
                        if let Some(created_before) = conditions.created_before {
                            if self.created_at > created_before {
                                return EventDelegation::InvalidDelegation(
                                    "Event created after delegation ended".to_owned(),
                                );
                            }
                        }
                        return EventDelegation::DelegatedBy(pk);
                    }
                    Err(e) => {
                        return EventDelegation::InvalidDelegation(format!("{e}"));
                    }
                }
            }
        }

        EventDelegation::NotDelegated
    }

    /// If the event came through a proxy, get the (Protocol, Id)
    pub fn proxy(&self) -> Option<(String, String)> {
        for tag in self.tags.iter() {
            if let Ok((protocol, id)) = tag.parse_proxy() {
                return Some((protocol, id));
            }
        }
        None
    }
}

impl Ord for EventV3 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.created_at
            .cmp(&other.created_at)
            .then(self.id.cmp(&other.id))
    }
}

impl PartialOrd for EventV3 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Direct access into speedy-serialized bytes, to avoid alloc-deserialize just
// to peek at one of these fields
#[cfg(feature = "speedy")]
impl EventV3 {
    /// Read the ID of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_id_from_speedy_bytes(bytes: &[u8]) -> Option<Id> {
        if bytes.len() < 32 {
            None
        } else if let Ok(arr) = <[u8; 32]>::try_from(&bytes[0..32]) {
            Some(unsafe { std::mem::transmute::<[u8; 32], id::Id>(arr) })
        } else {
            None
        }
    }

    /// Read the pubkey of the event from a speedy encoding without decoding
    /// (close to zero allocation, VerifyingKey does stuff I didn't check)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_pubkey_from_speedy_bytes(bytes: &[u8]) -> Option<PublicKey> {
        if bytes.len() < 64 {
            None
        } else {
            PublicKey::from_bytes(&bytes[32..64], false).ok()
        }
    }

    /// Read the created_at of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_created_at_from_speedy_bytes(bytes: &[u8]) -> Option<Unixtime> {
        if bytes.len() < 72 {
            None
        } else if let Ok(i) = i64::read_from_buffer(&bytes[64..72]) {
            Some(Unixtime(i))
        } else {
            None
        }
    }

    /// Read the kind of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_kind_from_speedy_bytes(bytes: &[u8]) -> Option<EventKind> {
        if bytes.len() < 76 {
            None
        } else if let Ok(u) = u32::read_from_buffer(&bytes[72..76]) {
            Some(u.into())
        } else {
            None
        }
    }

    // Read the sig of the event from a speedy encoding without decoding
    // (offset would be 76..140

    /// Read the content of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_content_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());

        unsafe {
            Some(std::str::from_utf8_unchecked(
                &bytes[140 + 4..140 + 4 + len as usize],
            ))
        }
    }

    /// Check if any human-readable tag matches the Regex in the speedy encoding
    /// without decoding the whole thing (because our TagV3 representation is so
    /// complicated, we do deserialize the tags for now)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn tag_search_in_speedy_bytes(bytes: &[u8], re: &Regex) -> Result<bool, Error> {
        if bytes.len() < 140 {
            return Ok(false);
        }

        // skip content
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());
        let offset = 140 + 4 + len as usize;

        // Deserialize the tags
        let tags: Vec<TagV3> = Vec::<TagV3>::read_from_buffer(&bytes[offset..])?;

        // Search through them
        for tag in &tags {
            match tag.tagname() {
                "content-warning" => {
                    if let Ok(Some(warning)) = tag.parse_content_warning() {
                        if re.is_match(warning.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                "t" => {
                    if let Ok(hashtag) = tag.parse_hashtag() {
                        if re.is_match(hashtag.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                "subject" => {
                    if let Ok(subject) = tag.parse_subject() {
                        if re.is_match(subject.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                "title" => {
                    if let Ok(title) = tag.parse_title() {
                        if re.is_match(title.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                _ => {
                    if tag.tagname() == "summary" && re.is_match(tag.value()) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

pub(crate) struct UnsignedEventV3(pub PreEventV3);

impl UnsignedEventV3 {
    pub(crate) fn new(
        pubkey: &XOnlyPublicKey,
        kind: u16,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> UnsignedEventV3 {
        let tags = tags.into_iter().map(TagV3).collect();

        UnsignedEventV3(PreEventV3 {
            pubkey: PublicKey::from_bytes(
                &pubkey.public_key(secp256k1::Parity::Even).serialize(),
                false,
            )
            .unwrap(),
            created_at: Unixtime::now(),
            kind: EventKind::from(kind as u32),
            tags,
            content,
        })
    }

    pub(crate) fn sign(self, private_key: &secp256k1::SecretKey) -> Result<EventV3, Error> {
        let id = self.0.hash()?;
        let signer =
            KeySigner::from_private_key(PrivateKey(*private_key, KeySecurity::Medium), "", 1)?;
        let sig = signer.sign_id(id)?;
        Ok(EventV3 {
            id,
            pubkey: self.0.pubkey,
            created_at: self.0.created_at,
            kind: self.0.kind,
            tags: self.0.tags,
            content: self.0.content,
            sig,
        })
    }
}

impl From<EventV3> for RumorV3 {
    fn from(e: EventV3) -> RumorV3 {
        RumorV3 {
            id: e.id,
            pubkey: e.pubkey,
            created_at: e.created_at,
            kind: e.kind,
            content: e.content,
            tags: e.tags,
        }
    }
}

impl From<RumorV3> for PreEventV3 {
    fn from(r: RumorV3) -> PreEventV3 {
        PreEventV3 {
            pubkey: r.pubkey,
            created_at: r.created_at,
            kind: r.kind,
            content: r.content,
            tags: r.tags,
        }
    }
}

impl TryFrom<PreEventV3> for RumorV3 {
    type Error = Error;
    fn try_from(e: PreEventV3) -> Result<RumorV3, Error> {
        RumorV3::new(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{DelegationConditions, Signer, UncheckedUrl};

    test_serde! {EventV3, test_event_serde}

    #[test]
    fn test_event_new_and_verify() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };
        let pubkey = signer.public_key();
        let preevent = PreEventV3 {
            pubkey,
            created_at: Unixtime::mock(),
            kind: EventKind::TextNote,
            tags: vec![TagV3::new_event(
                Id::mock(),
                Some(UncheckedUrl::mock()),
                None,
            )],
            content: "Hello World!".to_string(),
        };
        let id = preevent.hash().unwrap();
        let sig = signer.sign_id(id).unwrap();
        let mut event = EventV3 {
            id,
            pubkey: preevent.pubkey,
            created_at: preevent.created_at,
            kind: preevent.kind,
            tags: preevent.tags,
            content: preevent.content,
            sig,
        };

        assert!(event.verify(None).is_ok());

        // Now make sure it fails when the message has been modified
        event.content = "I'm changing this message".to_string();
        let result = event.verify(None);
        assert!(result.is_err());

        // Change it back
        event.content = "Hello World!".to_string();
        let result = event.verify(None);
        assert!(result.is_ok());

        // Tweak the id only
        event.id = Id([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let result = event.verify(None);
        assert!(result.is_err());
    }

    // helper
    fn create_event_with_delegation<S>(created_at: Unixtime, real_signer: &S) -> EventV3
    where
        S: Signer,
    {
        let delegated_signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let conditions = DelegationConditions::try_from_str(
            "kind=1&created_at>1680000000&created_at<1680050000",
        )
        .unwrap();

        let sig = real_signer
            .generate_delegation_signature(delegated_signer.public_key(), &conditions)
            .unwrap();

        let preevent = PreEventV3 {
            pubkey: delegated_signer.public_key(),
            created_at,
            kind: EventKind::TextNote,
            tags: vec![
                TagV3::new_event(Id::mock(), Some(UncheckedUrl::mock()), None),
                TagV3::new_delegation(real_signer.public_key(), conditions, sig),
            ],
            content: "Hello World!".to_string(),
        };
        let id = preevent.hash().unwrap();
        let sig = delegated_signer.sign_id(id).unwrap();
        EventV3 {
            id,
            pubkey: preevent.pubkey,
            created_at: preevent.created_at,
            kind: preevent.kind,
            tags: preevent.tags,
            content: preevent.content,
            sig,
        }
    }

    #[test]
    fn test_event_with_delegation_ok() {
        let delegator_signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };
        let delegator_pubkey = delegator_signer.public_key();

        let event = create_event_with_delegation(Unixtime(1680000012), &delegator_signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::DelegatedBy(pk) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(pk, delegator_pubkey);
        } else {
            panic!("Expected DelegatedBy result, got {:?}", event.delegation());
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_after() {
        let delegator_privkey = PrivateKey::mock();
        let signer = KeySigner::from_private_key(delegator_privkey, "", 1).unwrap();

        let event = create_event_with_delegation(Unixtime(1690000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created after delegation ended");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_before() {
        let signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };

        let event = create_event_with_delegation(Unixtime(1610000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created before delegation started");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_realworld_event_with_naddr_tag() {
        let raw = r##"{"id":"7760408f6459b9546c3a4e70e3e56756421fba34526b7d460db3fcfd2f8817db","pubkey":"460c25e682fda7832b52d1f22d3d22b3176d972f60dcdc3212ed8c92ef85065c","created_at":1687616920,"kind":1,"tags":[["p","1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411","","mention"],["a","30311:1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411:1687612774","","mention"]],"content":"Watching Karnage's stream to see if I learn something about design. \n\nnostr:naddr1qq9rzd3cxumrzv3hxu6qygqmcu9qzj9n7vtd5vl78jyly037wxkyl7vcqflvwy4eqhxjfa4yzypsgqqqwens0qfplk","sig":"dbc5d05a24bfe990a1faaedfcb81a98940d86a105711dbdad9145d05b0ad0f46e3e24eaa3fc283818f27e057fe836a029fd9a68e7f1de06ff477493199d64064"}"##;
        let _: EventV3 = serde_json::from_str(&raw).unwrap();
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_encoded_direct_field_access() {
        use speedy::Writable;

        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let preevent = PreEventV3 {
            pubkey: signer.public_key(),
            created_at: Unixtime(1680000012),
            kind: EventKind::TextNote,
            tags: vec![
                TagV3::new_event(Id::mock(), Some(UncheckedUrl::mock()), None),
                TagV3::new_hashtag("foodstr".to_string()),
            ],
            content: "Hello World!".to_string(),
        };
        let event = signer.sign_event(preevent).unwrap();
        let bytes = event.write_to_vec().unwrap();

        let id = EventV3::get_id_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(id, event.id);

        let pubkey = EventV3::get_pubkey_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(pubkey, event.pubkey);

        let created_at = EventV3::get_created_at_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(created_at, Unixtime(1680000012));

        let kind = EventV3::get_kind_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(kind, event.kind);

        let content = EventV3::get_content_from_speedy_bytes(&bytes);
        assert_eq!(content, Some(&*event.content));

        let re = regex::Regex::new("foodstr").unwrap();
        let found_foodstr = EventV3::tag_search_in_speedy_bytes(&bytes, &re).unwrap();
        assert!(found_foodstr);

        // Print to work out encoding
        //   test like this to see printed data:
        //   cargo test --features=speedy test_speedy_encoded_direct_field_access --
        // --nocapture
        println!("EVENT BYTES: {:?}", bytes);
        println!("ID: {:?}", event.id.0);
        println!("PUBKEY: {:?}", event.pubkey.as_slice());
        println!("CREATED AT: {:?}", event.created_at.0.to_ne_bytes());
        let kind32: u32 = event.kind.into();
        println!("KIND: {:?}", kind32.to_ne_bytes());
        println!("SIG: {:?}", event.sig.0.as_ref());
        println!(
            "CONTENT: [len={:?}] {:?}",
            (event.content.as_bytes().len() as u32).to_ne_bytes(),
            event.content.as_bytes()
        );
        println!("TAGS: [len={:?}]", (event.tags.len() as u32).to_ne_bytes());
    }

    #[test]
    fn test_event_gift_wrap() {
        let signer1 = {
            let sec1 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap();
            KeySigner::from_private_key(sec1, "", 1).unwrap()
        };

        let signer2 = {
            let sec2 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap();
            KeySigner::from_private_key(sec2, "", 1).unwrap()
        };

        let pre = PreEventV3 {
            pubkey: signer1.public_key(),
            created_at: Unixtime(1692_000_000),
            kind: EventKind::TextNote,
            content: "Hey man, this rocks! Please reply for a test.".to_string(),
            tags: vec![],
        };

        let gift_wrap = signer1.giftwrap(pre.clone(), signer2.public_key()).unwrap();
        let rumor = signer2.unwrap_giftwrap(&gift_wrap).unwrap();
        let output_pre: PreEventV3 = rumor.into();

        assert_eq!(pre, output_pre);
    }

    #[test]
    fn test_a_tags_as_replies() {
        let raw = r#"{"id":"d4fb3aeae033baa4a9504027bff8fd065ba1bbd635c501a5e4f8c7ab0bd37c34","pubkey":"7bdef7be22dd8e59f4600e044aa53a1cf975a9dc7d27df5833bc77db784a5805","created_at":1716980987,"kind":1,"sig":"903ae95893082835a42706eda1328ea85a8bf6fbb172bb2f8696b66fccfebfae8756992894a0fb7bb592cb3f78939bdd5fac4cd1eb49138cbf3ea8069574a1dc","content":"The article is interesting, but why compiling everything when configuring meta tags in dist/index.html is sufficient? (like you did in the first version, if I'm not wrong)\nOne main selling point of Oracolo is that it does not require complex server side setup.\n\n> Every time you access the web page, the web page is compiled\n\nThis is not technically correct :)\nJavaScript code is not compiled, it is simply executed; it fetches Nostr data and so builds the page.","tags":[["p","b12b632c887f0c871d140d37bcb6e7c1e1a80264d0b7de8255aa1951d9e1ff79"],["a","30023:b12b632c887f0c871d140d37bcb6e7c1e1a80264d0b7de8255aa1951d9e1ff79:1716928135712","","root"],["r","index.html"]]}"#;
        let event: EventV3 = serde_json::from_str(&raw).unwrap();
        if let Some(parent) = event.replies_to() {
            assert!(matches!(parent, EventReference::Addr(_)));
        } else {
            panic!("a tag reply not recognized");
        }
    }
}
```

---

### metadata.rs

**Size:** 6538 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};
use serde_json::{json, Map, Value};

/// Metadata about a user
///
/// Note: the value is an Option because some real-world data has been found to
/// contain JSON nulls as values, and we don't want deserialization of those
/// events to fail. We treat these in our get() function the same as if the key
/// did not exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataV1 {
    /// username
    pub name: Option<String>,

    /// about
    pub about: Option<String>,

    /// picture URL
    pub picture: Option<String>,

    /// nip05 dns id
    pub nip05: Option<String>,

    /// Additional fields not specified in NIP-01 or NIP-05
    pub other: Map<String, Value>,
}

impl Default for MetadataV1 {
    fn default() -> Self {
        MetadataV1 {
            name: None,
            about: None,
            picture: None,
            nip05: None,
            other: Map::new(),
        }
    }
}

impl MetadataV1 {
    /// Create new empty Metadata
    pub fn new() -> MetadataV1 {
        MetadataV1::default()
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> MetadataV1 {
        let mut map = Map::new();
        let _ = map.insert(
            "display_name".to_string(),
            Value::String("William Caserin".to_string()),
        );
        MetadataV1 {
            name: Some("jb55".to_owned()),
            about: None,
            picture: None,
            nip05: Some("jb55.com".to_owned()),
            other: map,
        }
    }

    /// Get the lnurl for the user, if available via lud06 or lud16
    pub fn lnurl(&self) -> Option<String> {
        if let Some(Value::String(lud06)) = self.other.get("lud06") {
            if let Ok(data) = bech32::decode(lud06) {
                if data.0 == *crate::types::HRP_LNURL {
                    return Some(String::from_utf8_lossy(&data.1).to_string());
                }
            }
        }

        if let Some(Value::String(lud16)) = self.other.get("lud16") {
            let vec: Vec<&str> = lud16.split('@').collect();
            if vec.len() == 2 {
                let user = &vec[0];
                let domain = &vec[1];
                return Some(format!("https://{domain}/.well-known/lnurlp/{user}"));
            }
        }

        None
    }
}

impl Serialize for MetadataV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4 + self.other.len()))?;
        map.serialize_entry("name", &json!(&self.name))?;
        map.serialize_entry("about", &json!(&self.about))?;
        map.serialize_entry("picture", &json!(&self.picture))?;
        map.serialize_entry("nip05", &json!(&self.nip05))?;
        for (k, v) in &self.other {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for MetadataV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MetadataV1Visitor)
    }
}

struct MetadataV1Visitor;

impl<'de> Visitor<'de> for MetadataV1Visitor {
    type Value = MetadataV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<MetadataV1, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut m: MetadataV1 = Default::default();

        if let Some(Value::String(s)) = map.remove("name") {
            m.name = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("about") {
            m.about = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("picture") {
            m.picture = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("nip05") {
            m.nip05 = Some(s);
        }

        m.other = map;

        Ok(m)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {MetadataV1, test_metadata_serde}

    #[test]
    fn test_metadata_print_json() {
        // I want to see if JSON serialized metadata is network appropriate
        let m = MetadataV1::mock();
        println!("{}", serde_json::to_string(&m).unwrap());
    }

    #[test]
    fn test_tolerate_nulls() {
        let json = r##"{"name":"monlovesmango","picture":"https://astral.ninja/aura/monlovesmango.svg","about":"building on nostr","nip05":"monlovesmango@astral.ninja","lud06":null,"testing":"123"}"##;
        let m: MetadataV1 = serde_json::from_str(json).unwrap();
        assert_eq!(m.name, Some("monlovesmango".to_owned()));
        assert_eq!(m.other.get("lud06"), Some(&Value::Null));
        assert_eq!(
            m.other.get("testing"),
            Some(&Value::String("123".to_owned()))
        );
    }

    #[test]
    fn test_metadata_lnurls() {
        // test lud06
        let json = r##"{"name":"mikedilger","about":"Author of Gossip client: https://github.com/mikedilger/gossip\nexpat American living in New Zealand","picture":"https://avatars.githubusercontent.com/u/1669069","nip05":"_@mikedilger.com","banner":"https://mikedilger.com/banner.jpg","display_name":"Michael Dilger","location":"New Zealand","lud06":"lnurl1dp68gurn8ghj7ampd3kx2ar0veekzar0wd5xjtnrdakj7tnhv4kxctttdehhwm30d3h82unvwqhkgetrv4h8gcn4dccnxv563ep","website":"https://mikedilger.com"}"##;
        let m: MetadataV1 = serde_json::from_str(json).unwrap();
        assert_eq!(
            m.lnurl().as_deref(),
            Some("https://walletofsatoshi.com/.well-known/lnurlp/decentbun13")
        );

        // test lud16
        let json = r##"{"name":"mikedilger","about":"Author of Gossip client: https://github.com/mikedilger/gossip\nexpat American living in New Zealand","picture":"https://avatars.githubusercontent.com/u/1669069","nip05":"_@mikedilger.com","banner":"https://mikedilger.com/banner.jpg","display_name":"Michael Dilger","location":"New Zealand","lud16":"decentbun13@walletofsatoshi.com","website":"https://mikedilger.com"}"##;
        let m: MetadataV1 = serde_json::from_str(json).unwrap();
        assert_eq!(
            m.lnurl().as_deref(),
            Some("https://walletofsatoshi.com/.well-known/lnurlp/decentbun13")
        );
    }
}
```

---

### mod.rs

**Size:** 1413 bytes | **Modified:** 2026-01-20 14:02:27

```rust
pub(crate) mod client_message1;
pub use client_message1::ClientMessageV1;

pub(crate) mod client_message2;
pub use client_message2::ClientMessageV2;

pub(crate) mod client_message3;
pub use client_message3::ClientMessageV3;

pub(crate) mod event1;
pub use event1::{EventV1, PreEventV1, RumorV1};

pub(crate) mod event2;
pub use event2::{EventV2, PreEventV2, RumorV2};

pub(crate) mod event3;
pub use event3::{EventV3, PreEventV3, RumorV3};

pub(crate) mod metadata;
pub use metadata::MetadataV1;

pub(crate) mod nip05;
pub use nip05::Nip05V1;

pub(crate) mod relay_information_document1;
pub use relay_information_document1::{
    FeeV1, RelayFeesV1, RelayInformationDocumentV1, RelayLimitationV1, RelayRetentionV1,
};
pub(crate) mod relay_information_document2;
pub use relay_information_document2::{RelayInformationDocumentV2, RelayLimitationV2};

pub(crate) mod relay_message1;
pub use relay_message1::RelayMessageV1;

pub(crate) mod relay_message2;
pub use relay_message2::RelayMessageV2;

pub(crate) mod relay_message3;
pub use relay_message3::RelayMessageV3;

pub(crate) mod relay_message4;
pub use relay_message4::RelayMessageV4;

pub(crate) mod relay_message5;
pub use relay_message5::{RelayMessageV5, Why};

pub(crate) mod tag1;
pub use tag1::TagV1;

pub(crate) mod tag2;
pub use tag2::TagV2;

pub(crate) mod tag3;
pub use tag3::TagV3;

pub(crate) mod zap_data;
pub use zap_data::{ZapDataV1, ZapDataV2};
```

---

### nip05.rs

**Size:** 2571 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{PublicKeyHex, UncheckedUrl};

/// The content of a webserver's /.well-known/nostr.json file used in NIP-05 and
/// NIP-35 This allows lookup and verification of a nostr user via a
/// `user@domain` style identifier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Nip05V1 {
    /// DNS names mapped to public keys
    pub names: HashMap<String, PublicKeyHex>,

    /// Public keys mapped to arrays of relays where they post
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub relays: HashMap<PublicKeyHex, Vec<UncheckedUrl>>,
}

impl Nip05V1 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Nip05V1 {
        let pubkey = PublicKeyHex::try_from_str(
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9",
        )
        .unwrap();

        let mut names: HashMap<String, PublicKeyHex> = HashMap::new();
        let _ = names.insert("bob".to_string(), pubkey.clone());

        let mut relays: HashMap<PublicKeyHex, Vec<UncheckedUrl>> = HashMap::new();
        let _ = relays.insert(
            pubkey,
            vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com"),
            ],
        );

        Nip05V1 { names, relays }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {Nip05V1, test_nip05_serde}

    #[test]
    fn test_nip05_example() {
        let body = r#"{
  "names": {
    "bob": "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9"
  },
  "relays": {
    "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9": [ "wss://relay.example.com", "wss://relay2.example.com" ]
  }
}"#;

        let nip05: Nip05V1 = serde_json::from_str(body).unwrap();

        let bobs_pk: PublicKeyHex = nip05.names.get("bob").unwrap().clone();
        assert_eq!(
            bobs_pk.as_str(),
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9"
        );

        let bobs_relays: Vec<UncheckedUrl> = nip05.relays.get(&bobs_pk).unwrap().to_owned();

        assert_eq!(
            bobs_relays,
            vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com")
            ]
        );
    }
}
```

---

### relay_information_document1.rs

**Size:** 21000 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

//use serde::de::Error as DeError;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use serde_json::{json, Map, Value};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{EventKind, EventKindOrRange, PublicKeyHex, Url};

/// Relay limitations
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayLimitationV1 {
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
}

impl fmt::Display for RelayLimitationV1 {
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
        Ok(())
    }
}

/// Relay retention
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayRetentionV1 {
    /// kinds
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKindOrRange>,

    /// time
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub time: Option<usize>,

    /// count
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub count: Option<usize>,
}

impl fmt::Display for RelayRetentionV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Retention:")?;
        write!(f, " Kinds=\"{:?}\"", self.kinds)?;
        if let Some(time) = &self.time {
            write!(f, " Time=\"{time}\"")?;
        }
        if let Some(count) = &self.count {
            write!(f, " Count=\"{count}\"")?;
        }
        Ok(())
    }
}

/// Fee
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct FeeV1 {
    /// Amount of the fee
    pub amount: usize,

    /// Unit of the amount
    pub unit: String,

    /// Kinds of events
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKindOrRange>,

    /// Period purchase lasts for
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub period: Option<usize>,
}

impl fmt::Display for FeeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fee=[{} {}", self.amount, self.unit)?;
        write!(f, " Kinds=\"{:?}\"", self.kinds)?;
        if let Some(period) = &self.period {
            write!(f, " Period=\"{}\"", period)?;
        }
        write!(f, "]")
    }
}

/// Relay fees
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayFeesV1 {
    /// Admission fee (read and write)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub admission: Vec<FeeV1>,

    /// Subscription fee (read)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub subscription: Vec<FeeV1>,

    /// Publication fee (write)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub publication: Vec<FeeV1>,
}

impl fmt::Display for RelayFeesV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Fees:")?;
        write!(f, " Admission=[")?;
        for fee in &self.admission {
            write!(f, "{} ", fee)?;
        }
        write!(f, "],Subscription=[")?;
        for fee in &self.subscription {
            write!(f, "{} ", fee)?;
        }
        write!(f, "],Publication=[")?;
        for fee in &self.publication {
            write!(f, "{} ", fee)?;
        }
        write!(f, "]")
    }
}

/// Relay information document as described in NIP-11, supplied by a relay
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayInformationDocumentV1 {
    /// Name of the relay
    pub name: Option<String>,

    /// Description of the relay in plain text
    pub description: Option<String>,

    /// Public key of an administrative contact of the relay
    pub pubkey: Option<PublicKeyHex>,

    /// An administrative contact for the relay. Should be a URI.
    pub contact: Option<String>,

    /// A list of NIPs supported by the relay
    pub supported_nips: Vec<u32>,

    /// The software running the relay
    pub software: Option<String>,

    /// The software version
    pub version: Option<String>,

    /// limitation
    pub limitation: Option<RelayLimitationV1>,

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

impl Default for RelayInformationDocumentV1 {
    fn default() -> RelayInformationDocumentV1 {
        RelayInformationDocumentV1 {
            name: None,
            description: None,
            pubkey: None,
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

impl RelayInformationDocumentV1 {
    /// If the relay supports the queried `nip`
    pub fn supports_nip(&self, nip: u32) -> bool {
        self.supported_nips.contains(&nip)
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayInformationDocumentV1 {
        let mut m = Map::new();
        let _ = m.insert(
            "early_nips".to_string(),
            Value::Array(vec![
                Value::Number(5.into()),
                Value::Number(6.into()),
                Value::Number(7.into()),
            ]),
        );
        RelayInformationDocumentV1 {
            name: Some("Crazy Horse".to_string()),
            description: Some("A really wild horse".to_string()),
            pubkey: Some(PublicKeyHex::mock()),
            contact: None,
            supported_nips: vec![11, 12, 13, 14],
            software: None,
            version: None,
            limitation: Some(RelayLimitationV1 {
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

impl fmt::Display for RelayInformationDocumentV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Information:")?;
        if let Some(name) = &self.name {
            write!(f, " Name=\"{name}\"")?;
        }
        if let Some(desc) = &self.description {
            write!(f, " Description=\"{desc}\"")?;
        }
        if let Some(pubkey) = &self.pubkey {
            write!(f, " Pubkey=\"{pubkey}\"")?;
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

impl Serialize for RelayInformationDocumentV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(7 + self.other.len()))?;
        if self.name.is_some() {
            map.serialize_entry("name", &json!(&self.name))?;
        }
        if self.description.is_some() {
            map.serialize_entry("description", &json!(&self.description))?;
        }
        if self.pubkey.is_some() {
            map.serialize_entry("pubkey", &json!(&self.pubkey))?;
        }
        if self.contact.is_some() {
            map.serialize_entry("contact", &json!(&self.contact))?;
        }
        map.serialize_entry("supported_nips", &json!(&self.supported_nips))?;
        if self.software.is_some() {
            map.serialize_entry("software", &json!(&self.software))?;
        }
        if self.version.is_some() {
            map.serialize_entry("version", &json!(&self.version))?;
        }
        if self.limitation.is_some() {
            map.serialize_entry("limitation", &json!(&self.limitation))?;
        }
        if !self.retention.is_empty() {
            map.serialize_entry("retention", &json!(&self.retention))?;
        }
        if !self.relay_countries.is_empty() {
            map.serialize_entry("relay_countries", &json!(&self.relay_countries))?;
        }
        if !self.language_tags.is_empty() {
            map.serialize_entry("language_tags", &json!(&self.language_tags))?;
        }
        if !self.tags.is_empty() {
            map.serialize_entry("tags", &json!(&self.tags))?;
        }
        if self.posting_policy.is_some() {
            map.serialize_entry("posting_policy", &json!(&self.posting_policy))?;
        }
        if self.payments_url.is_some() {
            map.serialize_entry("payments_url", &json!(&self.payments_url))?;
        }
        if self.fees.is_some() {
            map.serialize_entry("fees", &json!(&self.fees))?;
        }
        for (k, v) in &self.other {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for RelayInformationDocumentV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RidVisitor)
    }
}

struct RidVisitor;

impl<'de> Visitor<'de> for RidVisitor {
    type Value = RelayInformationDocumentV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<RelayInformationDocumentV1, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut rid: RelayInformationDocumentV1 = Default::default();

        if let Some(Value::String(s)) = map.remove("name") {
            rid.name = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("description") {
            rid.description = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("pubkey") {
            rid.pubkey = PublicKeyHex::try_from_string(s).ok();
        }
        if let Some(Value::String(s)) = map.remove("contact") {
            rid.contact = Some(s);
        }
        if let Some(Value::Array(vec)) = map.remove("supported_nips") {
            for elem in vec.iter() {
                if let Value::Number(num) = elem {
                    if let Some(u) = num.as_u64() {
                        rid.supported_nips.push(u as u32);
                    }
                }
            }
        }
        if let Some(Value::String(s)) = map.remove("software") {
            rid.software = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("version") {
            rid.version = Some(s);
        }
        if let Some(v) = map.remove("limitation") {
            rid.limitation =
                serde_json::from_value::<Option<RelayLimitationV1>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("retention") {
            rid.retention = serde_json::from_value::<Vec<RelayRetentionV1>>(v).unwrap_or_default();
        }
        if let Some(v) = map.remove("relay_countries") {
            rid.relay_countries = serde_json::from_value::<Vec<String>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("language_tags") {
            rid.language_tags = serde_json::from_value::<Vec<String>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("tags") {
            rid.tags = serde_json::from_value::<Vec<String>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("posting_policy") {
            rid.posting_policy = serde_json::from_value::<Option<Url>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("payments_url") {
            rid.payments_url = serde_json::from_value::<Option<Url>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("fees") {
            rid.fees = serde_json::from_value::<Option<RelayFeesV1>>(v).unwrap_or_default()
        }

        rid.other = map;

        Ok(rid)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayInformationDocumentV1, test_relay_information_document_serde}

    #[test]
    fn test_to_json_only() {
        // This is so you can see the JSON limitation.
        // Run with "cargo test toest_to_json_only -- --nocapture"
        let mock = RelayInformationDocumentV1::mock();
        let s = serde_json::to_string(&mock).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_relay_information_document_json() {
        let json = r##"{ "name": "A Relay", "description": null, "myfield": [1,2], "supported_nips": [11,12], "retention": [
    { "kinds": [0, 1, [5, 7], [40, 49]], "time": 3600 },
    { "kinds": [[40000, 49999]], "time": 100 },
    { "kinds": [[30000, 39999]], "count": 1000 },
    { "time": 3600, "count": 10000 }
  ] }"##;
        let rid: RelayInformationDocumentV1 = serde_json::from_str(json).unwrap();
        let json2 = serde_json::to_value(&rid).unwrap();
        let expected_json2: Value = serde_json::from_str(r##"{"name":"A Relay","supported_nips":[11,12],"retention":[{"kinds":[0,1,[5,7],[40,49]],"time":3600},{"kinds":[[40000,49999]],"time":100},{"kinds":[[30000,39999]],"count":1000},{"time":3600,"count":10000}],"myfield":[1,2]}"##).unwrap();
        assert_eq!(json2, expected_json2);
    }
}
```

---

### relay_information_document2.rs

**Size:** 15789 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

//use serde::de::Error as DeError;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use serde_json::{json, Map, Value};
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
```

---

### relay_list.rs

**Size:** 0 bytes | **Modified:** 2026-01-20 14:02:27

```rust
```

---

### relay_message1.rs

**Size:** 5123 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV1;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV1 {
    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV1>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// Used to notify clients if an event was successuful
    Ok(Id, bool, String),

    /// Used to send authentication challenges
    Auth(String),
}

impl Serialize for RelayMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV1::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV1::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV1::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV1::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV1::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV1, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV1> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV1 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV1::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV1::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV1::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV1::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV1::Auth(challenge));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}
```

---

### relay_message2.rs

**Size:** 5438 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV2;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV2 {
    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV2>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// Used to notify clients if an event was successuful
    Ok(Id, bool, String),

    /// Used to send authentication challenges
    Auth(String),
}

impl RelayMessageV2 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV2 {
        RelayMessageV2::Event(SubscriptionId::mock(), Box::new(EventV2::mock()))
    }
}

impl Serialize for RelayMessageV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV2::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV2::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV2::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV2::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV2::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV2;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV2, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV2> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV2::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV2::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV2::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV2::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV2::Auth(challenge));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV2, test_relay_message_serde}
}
```

---

### relay_message3.rs

**Size:** 7579 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{EventV2, Why};
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV3 {
    /// Used to send authentication challenges
    Auth(String),

    /// Used to indicate that a subscription was ended on the server side
    /// Every ClientMessage::Req _may_ trigger a RelayMessage::Closed response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted, or error
    Closed(SubscriptionId, String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV2>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// Used to notify clients if an event was successuful
    /// Every ClientMessage::Event will trigger a RelayMessage::OK response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted or error
    Ok(Id, bool, String),
}

impl RelayMessageV3 {
    /// Translate the machine-readable prefix from the message
    pub fn why(&self) -> Option<Why> {
        let s = match *self {
            RelayMessageV3::Closed(_, ref s) => s,
            RelayMessageV3::Ok(_, _, ref s) => s,
            _ => return None,
        };

        match s.split(':').next() {
            Some("auth-required") => Some(Why::AuthRequired),
            Some("blocked") => Some(Why::Blocked),
            Some("duplicate") => Some(Why::Duplicate),
            Some("error") => Some(Why::Error),
            Some("invalid") => Some(Why::Invalid),
            Some("pow") => Some(Why::Pow),
            Some("rate-limited") => Some(Why::RateLimited),
            Some("restricted") => Some(Why::Restricted),
            _ => None,
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV3 {
        RelayMessageV3::Event(SubscriptionId::mock(), Box::new(EventV2::mock()))
    }
}

impl Serialize for RelayMessageV3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV3::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
            RelayMessageV3::Closed(id, message) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("CLOSED")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV3::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV3::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV3::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV3::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV3;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV3, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV3> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV3::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV3::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV3::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV3::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV3::Auth(challenge));
        } else if word == "CLOSED" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message messing id field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV3::Closed(id, message));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV3, test_relay_message_serde}
}
```

---

### relay_message4.rs

**Size:** 8123 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV3;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV4 {
    /// Used to send authentication challenges
    Auth(String),

    /// Used to indicate that a subscription was ended on the server side
    /// Every ClientMessage::Req _may_ trigger a RelayMessage::Closed response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted, or error
    Closed(SubscriptionId, String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV3>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// Used to notify clients if an event was successuful
    /// Every ClientMessage::Event will trigger a RelayMessage::OK response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted or error
    Ok(Id, bool, String),
}

/// The reason why a relay issued an OK or CLOSED message
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Why {
    /// Authentication is required
    AuthRequired,

    /// You have been blocked from this relay
    Blocked,

    /// Your request is a duplicate
    Duplicate,

    /// Other error
    Error,

    /// Your request is invalid
    Invalid,

    /// Proof-of-work is required
    Pow,

    /// Rejected due to rate limiting
    RateLimited,

    /// The action you requested is restricted to your identity
    Restricted,
}

impl RelayMessageV4 {
    /// Translate the machine-readable prefix from the message
    pub fn why(&self) -> Option<Why> {
        let s = match *self {
            RelayMessageV4::Closed(_, ref s) => s,
            RelayMessageV4::Ok(_, _, ref s) => s,
            _ => return None,
        };

        match s.split(':').next() {
            Some("auth-required") => Some(Why::AuthRequired),
            Some("blocked") => Some(Why::Blocked),
            Some("duplicate") => Some(Why::Duplicate),
            Some("error") => Some(Why::Error),
            Some("invalid") => Some(Why::Invalid),
            Some("pow") => Some(Why::Pow),
            Some("rate-limited") => Some(Why::RateLimited),
            Some("restricted") => Some(Why::Restricted),
            _ => None,
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV4 {
        RelayMessageV4::Event(SubscriptionId::mock(), Box::new(EventV3::mock()))
    }
}

impl Serialize for RelayMessageV4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV4::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
            RelayMessageV4::Closed(id, message) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("CLOSED")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV4::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV4::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV4::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV4::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV4;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV4, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV4> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV4::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV4::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV4::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV4::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV4::Auth(challenge));
        } else if word == "CLOSED" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message messing id field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV4::Closed(id, message));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV4, test_relay_message_serde}
}
```

---

### relay_message5.rs

**Size:** 8674 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV3;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV5 {
    /// Used to send authentication challenges
    Auth(String),

    /// Used to indicate that a subscription was ended on the server side
    /// Every ClientMessage::Req _may_ trigger a RelayMessage::Closed response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted, or error
    Closed(SubscriptionId, String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV3>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// A human readable notice for the end user
    Notify(String),

    /// Used to notify clients if an event was successuful
    /// Every ClientMessage::Event will trigger a RelayMessage::OK response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted or error
    Ok(Id, bool, String),
}

/// The reason why a relay issued an OK or CLOSED message
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Why {
    /// Authentication is required
    AuthRequired,

    /// You have been blocked from this relay
    Blocked,

    /// Your request is a duplicate
    Duplicate,

    /// Other error
    Error,

    /// Your request is invalid
    Invalid,

    /// Proof-of-work is required
    Pow,

    /// Rejected due to rate limiting
    RateLimited,

    /// The action you requested is restricted to your identity
    Restricted,
}

impl RelayMessageV5 {
    /// Translate the machine-readable prefix from the message
    pub fn why(&self) -> Option<Why> {
        let s = match *self {
            RelayMessageV5::Closed(_, ref s) => s,
            RelayMessageV5::Ok(_, _, ref s) => s,
            _ => return None,
        };

        match s.split(':').next() {
            Some("auth-required") => Some(Why::AuthRequired),
            Some("blocked") => Some(Why::Blocked),
            Some("duplicate") => Some(Why::Duplicate),
            Some("error") => Some(Why::Error),
            Some("invalid") => Some(Why::Invalid),
            Some("pow") => Some(Why::Pow),
            Some("rate-limited") => Some(Why::RateLimited),
            Some("restricted") => Some(Why::Restricted),
            _ => None,
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV5 {
        RelayMessageV5::Event(SubscriptionId::mock(), Box::new(EventV3::mock()))
    }
}

impl Serialize for RelayMessageV5 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV5::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
            RelayMessageV5::Closed(id, message) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("CLOSED")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV5::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV5::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV5::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV5::Notify(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTIFY")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV5::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV5 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV5;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV5, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV5> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV5::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Notice(s));
        } else if word == "NOTIFY" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Notify(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV5::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV5::Auth(challenge));
        } else if word == "CLOSED" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message messing id field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Closed(id, message));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV5, test_relay_message_serde}
}
```

---

### tag1.rs

**Size:** 31053 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    DelegationConditions, Error, EventKind, Id, PublicKeyHex, SignatureHex, UncheckedUrl, Unixtime,
};

/// A tag on an Event
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum TagV1 {
    /// Address 'a' tag to a parameterized replaceable event
    Address {
        /// EventKind
        kind: EventKind,

        /// Author
        pubkey: PublicKeyHex,

        /// d-tag identifier
        d: String,

        /// Relay URL
        relay_url: Option<UncheckedUrl>,

        /// Trailing
        trailing: Vec<String>,
    } = 0,

    /// Content Warning to alert client to hide content until user approves
    ContentWarning {
        /// Content warning
        warning: String,

        /// Trailing
        trailing: Vec<String>,
    } = 1,

    /// Delegation (Delegated Event Signing)
    Delegation {
        /// Public key of the delegator
        pubkey: PublicKeyHex,

        /// Conditions query string
        conditions: DelegationConditions,

        /// 64-byte schnorr signature of the sha256 hash of the delegation token
        sig: SignatureHex,

        /// Trailing
        trailing: Vec<String>,
    } = 2,

    /// This is a reference to an event, where the first string is the event Id.
    /// The second string is defined in NIP-01 as an optional URL, but
    /// subsequent 'e' NIPs define more data and interpretations.
    Event {
        /// The Id of some other event that this event refers to
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// A marker (commonly things like 'reply')
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 3,

    /// A time when the event should be considered expired
    Expiration {
        /// Expiration Time
        time: Unixtime,

        /// Trailing
        trailing: Vec<String>,
    } = 4,

    /// 'p' This is a reference to a user by public key, where the first string
    /// is the PublicKey. The second string is defined in NIP-01 as an
    /// optional URL, but subsqeuent NIPs define more data and
    /// interpretations.
    Pubkey {
        /// The public key of the identity that this event refers to
        pubkey: PublicKeyHex,

        /// A recommended relay URL to find information on that public key
        recommended_relay_url: Option<UncheckedUrl>,

        /// A petname given to this identity by the event author
        petname: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 5,

    /// 't' A hashtag
    Hashtag {
        /// Hashtag
        hashtag: String,

        /// Trailing
        trailing: Vec<String>,
    } = 6,

    /// 'r' A reference to a URL
    Reference {
        /// A relay url
        url: UncheckedUrl,

        /// An optional marker
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 7,

    /// 'g' A geohash
    Geohash {
        /// A geohash
        geohash: String,

        /// Trailing
        trailing: Vec<String>,
    } = 8,

    /// 'd' Identifier tag
    Identifier {
        /// 'd' indentifier
        d: String,

        /// Trailing
        trailing: Vec<String>,
    } = 9,

    /// A subject. The first string is the subject. Should only be in TextNote
    /// events.
    Subject {
        /// The subject
        subject: String,

        /// Trailing
        trailing: Vec<String>,
    } = 10,

    /// A nonce tag for Proof of Work
    Nonce {
        /// A random number that makes the event hash meet the proof of work
        /// required
        nonce: String,

        /// The target number of bits for the proof of work
        target: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 11,

    /// There is no known nostr tag like this. This was a mistake, but we can't
    /// remove it or deserialization of data serialized with this in mind
    /// will break.
    Parameter {
        /// Parameter
        param: String,

        /// Trailing
        trailing: Vec<String>,
    } = 12,

    /// Title (30023 long form)
    Title {
        /// Title
        title: String,

        /// Trailing
        trailing: Vec<String>,
    } = 13,

    /// Any other tag
    Other {
        /// The tag name
        tag: String,

        /// The subsequent fields
        data: Vec<String>,
    } = 14,

    /// An empty array (kept so signature remains valid across ser/de)
    Empty = 15,

    /// Direct parent of an event, 'E' tag
    /// This is from <https://github.com/nostr-protocol/nips/pull/830> which may not happen
    /// We should not create these, but we can support them if we encounter
    /// them.
    EventParent {
        /// The id of some other event that is the direct parent to this event
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// Trailing
        trailing: Vec<String>,
    } = 16,

    /// Kind number 'k'
    Kind {
        /// Event kind
        kind: EventKind,

        /// Trailing
        trailing: Vec<String>,
    } = 17,
}

impl TagV1 {
    /// Get the tag name for the tag (the first string in the array)a
    pub fn tagname(&self) -> String {
        match self {
            TagV1::Address { .. } => "address".to_string(),
            TagV1::ContentWarning { .. } => "content-warning".to_string(),
            TagV1::Delegation { .. } => "delegation".to_string(),
            TagV1::Event { .. } => "e".to_string(),
            TagV1::EventParent { .. } => "E".to_string(),
            TagV1::Expiration { .. } => "expiration".to_string(),
            TagV1::Kind { .. } => "k".to_string(),
            TagV1::Pubkey { .. } => "p".to_string(),
            TagV1::Hashtag { .. } => "t".to_string(),
            TagV1::Reference { .. } => "r".to_string(),
            TagV1::Geohash { .. } => "g".to_string(),
            TagV1::Identifier { .. } => "d".to_string(),
            TagV1::Subject { .. } => "subject".to_string(),
            TagV1::Nonce { .. } => "nonce".to_string(),
            TagV1::Parameter { .. } => "parameter".to_string(),
            TagV1::Title { .. } => "title".to_string(),
            TagV1::Other { tag, .. } => tag.clone(),
            TagV1::Empty => "".to_string(),
        }
    }

    /// Get the string value of the tag at an array index
    pub fn value(&self, index: usize) -> Result<String, Error> {
        use serde_json::Value;
        let json = serde_json::to_value(self)?;
        match json {
            Value::Array(vec) => match vec.get(index) {
                Some(val) => match val {
                    Value::String(s) => Ok(s.to_owned()),
                    _ => Err(Error::AssertionFailed(
                        "Tag field is not a string".to_owned(),
                    )),
                },
                None => Ok("".to_owned()),
            },
            _ => Err(Error::AssertionFailed(
                "Tag JSON is not an array".to_owned(),
            )),
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> TagV1 {
        TagV1::Event {
            id: Id::mock(),
            recommended_relay_url: Some(UncheckedUrl::mock()),
            marker: None,
            trailing: Vec::new(),
        }
    }
}

impl Serialize for TagV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("a")?;
                let k: u32 = From::from(*kind);
                let s = format!("{}:{}:{}", k, pubkey, d);
                seq.serialize_element(&s)?;
                if let Some(ru) = relay_url {
                    seq.serialize_element(ru)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::ContentWarning { warning, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("content-warning")?;
                seq.serialize_element(warning)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("delegation")?;
                seq.serialize_element(pubkey)?;
                seq.serialize_element(conditions)?;
                seq.serialize_element(sig)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("e")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if marker.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::EventParent {
                id,
                recommended_relay_url,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("E")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Expiration { time, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("expiration")?;
                seq.serialize_element(time)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Kind { kind, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("k")?;
                // in tags, we must use string types only
                let k: u32 = From::from(*kind);
                let s = format!("{}", k);
                seq.serialize_element(&s)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("p")?;
                seq.serialize_element(pubkey)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if petname.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(pn) = petname {
                    seq.serialize_element(pn)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Hashtag { hashtag, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("t")?;
                seq.serialize_element(hashtag)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Reference {
                url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("r")?;
                seq.serialize_element(url)?;
                if let Some(m) = marker {
                    seq.serialize_element(m)?
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Geohash { geohash, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("g")?;
                seq.serialize_element(geohash)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Identifier { d, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("d")?;
                seq.serialize_element(d)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Subject { subject, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("subject")?;
                seq.serialize_element(subject)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Nonce {
                nonce,
                target,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("nonce")?;
                seq.serialize_element(nonce)?;
                if let Some(t) = target {
                    seq.serialize_element(t)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Parameter { param, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("parameter")?;
                seq.serialize_element(param)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Title { title, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("title")?;
                seq.serialize_element(title)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Other { tag, data } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(tag)?;
                for s in data.iter() {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Empty => {
                let seq = serializer.serialize_seq(Some(0))?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for TagV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TagVisitor)
    }
}

struct TagVisitor;

impl<'de> Visitor<'de> for TagVisitor {
    type Value = TagV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<TagV1, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(e) => e,
            None => return Ok(TagV1::Empty),
        };
        if tagname == "a" {
            if let Some(a) = seq.next_element::<&str>()? {
                let relay_url: Option<UncheckedUrl> = seq.next_element()?;
                let mut trailing: Vec<String> = Vec::new();
                while let Some(s) = seq.next_element()? {
                    trailing.push(s);
                }

                let fail = || -> TagV1 {
                    match relay_url {
                        Some(ref url) => {
                            let mut fv = vec![a.to_string(), url.as_str().to_owned()];
                            fv.extend(trailing.clone());
                            TagV1::Other {
                                tag: tagname.to_string(),
                                data: fv,
                            }
                        }
                        None => TagV1::Other {
                            tag: tagname.to_string(),
                            data: vec![a.to_string()],
                        },
                    }
                };

                let parts: Vec<&str> = a.split(':').collect();
                if parts.len() < 3 {
                    return Ok(fail());
                }
                let kindnum: u32 = match parts[0].parse::<u32>() {
                    Ok(u) => u,
                    Err(_) => return Ok(fail()),
                };
                let kind: EventKind = From::from(kindnum);
                let pubkey: PublicKeyHex = match PublicKeyHex::try_from_str(parts[1]) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(fail()),
                };
                Ok(TagV1::Address {
                    kind,
                    pubkey,
                    d: parts[2].to_string(),
                    relay_url,
                    trailing,
                })
            } else {
                Ok(TagV1::Other {
                    tag: tagname.to_string(),
                    data: vec![],
                })
            }
        } else if tagname == "content-warning" {
            let msg = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::ContentWarning {
                warning: msg,
                trailing,
            })
        } else if tagname == "delegation" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(pk) => pk,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let conditions: DelegationConditions = match seq.next_element()? {
                Some(c) => c,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string()],
                    });
                }
            };
            let sig: SignatureHex = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string(), conditions.as_string()],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            })
        } else if tagname == "e" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            })
        } else if tagname == "E" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::EventParent {
                id,
                recommended_relay_url,
                trailing,
            })
        } else if tagname == "expiration" {
            let time = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Expiration { time, trailing })
        } else if tagname == "p" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(p) => p,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let petname: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            })
        } else if tagname == "t" {
            let tag = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Hashtag {
                hashtag: tag,
                trailing,
            })
        } else if tagname == "r" {
            let refr: UncheckedUrl = match seq.next_element()? {
                Some(r) => r,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Reference {
                url: refr,
                marker,
                trailing,
            })
        } else if tagname == "g" {
            let geo = match seq.next_element()? {
                Some(g) => g,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Geohash {
                geohash: geo,
                trailing,
            })
        } else if tagname == "d" {
            let id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    // Implicit empty value
                    return Ok(TagV1::Identifier {
                        d: "".to_string(),
                        trailing: Vec::new(),
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Identifier { d: id, trailing })
        } else if tagname == "k" {
            let mut parts: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                parts.push(s);
            }
            if parts.is_empty() {
                return Ok(TagV1::Other {
                    tag: tagname.to_string(),
                    data: parts,
                });
            }
            let kindnum: u32 = match parts[0].parse::<u32>() {
                Ok(u) => u,
                Err(_) => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: parts,
                    });
                }
            };
            let kind: EventKind = From::from(kindnum);
            Ok(TagV1::Kind {
                kind,
                trailing: parts[1..].to_owned(),
            })
        } else if tagname == "subject" {
            let sub = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Subject {
                subject: sub,
                trailing,
            })
        } else if tagname == "nonce" {
            let nonce = match seq.next_element()? {
                Some(n) => n,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let target: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Nonce {
                nonce,
                target,
                trailing,
            })
        } else if tagname == "parameter" {
            let param = match seq.next_element()? {
                Some(s) => s,
                None => "".to_owned(), // implicit parameter
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Parameter { param, trailing })
        } else if tagname == "title" {
            let title = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Title { title, trailing })
        } else {
            let mut data = Vec::new();
            loop {
                match seq.next_element()? {
                    None => {
                        return Ok(TagV1::Other {
                            tag: tagname.to_string(),
                            data,
                        });
                    }
                    Some(s) => data.push(s),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {TagV1, test_tag_serde}

    #[test]
    fn test_a_tag() {
        let tag = TagV1::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            relay_url: Some(UncheckedUrl("wss://relay.mikedilger.com/".to_string())),
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);

        let tag = TagV1::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            relay_url: None,
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);
    }
}
```

---

### tag2.rs

**Size:** 31823 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    DelegationConditions, Error, EventKind, Id, PublicKeyHex, SignatureHex, UncheckedUrl, Unixtime,
};

/// A tag on an Event
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum TagV2 {
    /// Address 'a' tag to a parameterized replaceable event
    Address {
        /// EventKind
        kind: EventKind,

        /// Author
        pubkey: PublicKeyHex,

        /// d-tag identifier
        d: String,

        /// Relay URL
        relay_url: Option<UncheckedUrl>,

        /// A marker (commonly things like 'reply')
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 0,

    /// Content Warning to alert client to hide content until user approves
    ContentWarning {
        /// Content warning
        warning: String,

        /// Trailing
        trailing: Vec<String>,
    } = 1,

    /// Delegation (Delegated Event Signing)
    Delegation {
        /// Public key of the delegator
        pubkey: PublicKeyHex,

        /// Conditions query string
        conditions: DelegationConditions,

        /// 64-byte schnorr signature of the sha256 hash of the delegation token
        sig: SignatureHex,

        /// Trailing
        trailing: Vec<String>,
    } = 2,

    /// This is a reference to an event, where the first string is the event Id.
    /// The second string is defined in NIP-01 as an optional URL, but
    /// subsequent 'e' NIPs define more data and interpretations.
    Event {
        /// The Id of some other event that this event refers to
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// A marker (commonly things like 'reply')
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 3,

    /// A time when the event should be considered expired
    Expiration {
        /// Expiration Time
        time: Unixtime,

        /// Trailing
        trailing: Vec<String>,
    } = 4,

    /// 'p' This is a reference to a user by public key, where the first string
    /// is the PublicKey. The second string is defined in NIP-01 as an
    /// optional URL, but subsqeuent NIPs define more data and
    /// interpretations.
    Pubkey {
        /// The public key of the identity that this event refers to
        pubkey: PublicKeyHex,

        /// A recommended relay URL to find information on that public key
        recommended_relay_url: Option<UncheckedUrl>,

        /// A petname given to this identity by the event author
        petname: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 5,

    /// 't' A hashtag
    Hashtag {
        /// Hashtag
        hashtag: String,

        /// Trailing
        trailing: Vec<String>,
    } = 6,

    /// 'r' A reference to a URL
    Reference {
        /// A relay url
        url: UncheckedUrl,

        /// An optional marker
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 7,

    /// 'g' A geohash
    Geohash {
        /// A geohash
        geohash: String,

        /// Trailing
        trailing: Vec<String>,
    } = 8,

    /// 'd' Identifier tag
    Identifier {
        /// 'd' indentifier
        d: String,

        /// Trailing
        trailing: Vec<String>,
    } = 9,

    /// A subject. The first string is the subject. Should only be in TextNote
    /// events.
    Subject {
        /// The subject
        subject: String,

        /// Trailing
        trailing: Vec<String>,
    } = 10,

    /// A nonce tag for Proof of Work
    Nonce {
        /// A random number that makes the event hash meet the proof of work
        /// required
        nonce: String,

        /// The target number of bits for the proof of work
        target: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 11,

    /// There is no known nostr tag like this. This was a mistake, but we can't
    /// remove it or deserialization of data serialized with this in mind
    /// will break.
    Parameter {
        /// Parameter
        param: String,

        /// Trailing
        trailing: Vec<String>,
    } = 12,

    /// Title (30023 long form)
    Title {
        /// Title
        title: String,

        /// Trailing
        trailing: Vec<String>,
    } = 13,

    /// Any other tag
    Other {
        /// The tag name
        tag: String,

        /// The subsequent fields
        data: Vec<String>,
    } = 14,

    /// An empty array (kept so signature remains valid across ser/de)
    Empty = 15,

    /// Direct parent of an event, 'E' tag
    /// This is from <https://github.com/nostr-protocol/nips/pull/830> which may not happen
    /// We should not create these, but we can support them if we encounter
    /// them.
    EventParent {
        /// The id of some other event that is the direct parent to this event
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// Trailing
        trailing: Vec<String>,
    } = 16,

    /// Kind number 'k'
    Kind {
        /// Event kind
        kind: EventKind,

        /// Trailing
        trailing: Vec<String>,
    } = 17,
}

impl TagV2 {
    /// Get the tag name for the tag (the first string in the array)a
    pub fn tagname(&self) -> String {
        match self {
            TagV2::Address { .. } => "address".to_string(),
            TagV2::ContentWarning { .. } => "content-warning".to_string(),
            TagV2::Delegation { .. } => "delegation".to_string(),
            TagV2::Event { .. } => "e".to_string(),
            TagV2::EventParent { .. } => "E".to_string(),
            TagV2::Expiration { .. } => "expiration".to_string(),
            TagV2::Kind { .. } => "k".to_string(),
            TagV2::Pubkey { .. } => "p".to_string(),
            TagV2::Hashtag { .. } => "t".to_string(),
            TagV2::Reference { .. } => "r".to_string(),
            TagV2::Geohash { .. } => "g".to_string(),
            TagV2::Identifier { .. } => "d".to_string(),
            TagV2::Subject { .. } => "subject".to_string(),
            TagV2::Nonce { .. } => "nonce".to_string(),
            TagV2::Parameter { .. } => "parameter".to_string(),
            TagV2::Title { .. } => "title".to_string(),
            TagV2::Other { tag, .. } => tag.clone(),
            TagV2::Empty => "".to_string(),
        }
    }

    /// Get the string value of the tag at an array index
    pub fn value(&self, index: usize) -> Result<String, Error> {
        use serde_json::Value;
        let json = serde_json::to_value(self)?;
        match json {
            Value::Array(vec) => match vec.get(index) {
                Some(val) => match val {
                    Value::String(s) => Ok(s.to_owned()),
                    _ => Err(Error::AssertionFailed(
                        "Tag field is not a string".to_owned(),
                    )),
                },
                None => Ok("".to_owned()),
            },
            _ => Err(Error::AssertionFailed(
                "Tag JSON is not an array".to_owned(),
            )),
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> TagV2 {
        TagV2::Event {
            id: Id::mock(),
            recommended_relay_url: Some(UncheckedUrl::mock()),
            marker: None,
            trailing: Vec::new(),
        }
    }
}

impl Serialize for TagV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("a")?;
                let k: u32 = From::from(*kind);
                let s = format!("{}:{}:{}", k, pubkey, d);
                seq.serialize_element(&s)?;
                if let Some(ru) = relay_url {
                    seq.serialize_element(ru)?;
                } else if marker.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::ContentWarning { warning, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("content-warning")?;
                seq.serialize_element(warning)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("delegation")?;
                seq.serialize_element(pubkey)?;
                seq.serialize_element(conditions)?;
                seq.serialize_element(sig)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("e")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if marker.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::EventParent {
                id,
                recommended_relay_url,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("E")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Expiration { time, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("expiration")?;
                seq.serialize_element(time)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Kind { kind, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("k")?;
                // in tags, we must use string types only
                let k: u32 = From::from(*kind);
                let s = format!("{}", k);
                seq.serialize_element(&s)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("p")?;
                seq.serialize_element(pubkey)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if petname.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(pn) = petname {
                    seq.serialize_element(pn)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Hashtag { hashtag, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("t")?;
                seq.serialize_element(hashtag)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Reference {
                url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("r")?;
                seq.serialize_element(url)?;
                if let Some(m) = marker {
                    seq.serialize_element(m)?
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Geohash { geohash, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("g")?;
                seq.serialize_element(geohash)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Identifier { d, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("d")?;
                seq.serialize_element(d)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Subject { subject, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("subject")?;
                seq.serialize_element(subject)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Nonce {
                nonce,
                target,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("nonce")?;
                seq.serialize_element(nonce)?;
                if let Some(t) = target {
                    seq.serialize_element(t)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Parameter { param, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("parameter")?;
                seq.serialize_element(param)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Title { title, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("title")?;
                seq.serialize_element(title)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Other { tag, data } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(tag)?;
                for s in data.iter() {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Empty => {
                let seq = serializer.serialize_seq(Some(0))?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for TagV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TagVisitor)
    }
}

struct TagVisitor;

impl<'de> Visitor<'de> for TagVisitor {
    type Value = TagV2;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<TagV2, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(e) => e,
            None => return Ok(TagV2::Empty),
        };
        if tagname == "a" {
            let a: &str = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };

            let mut a_parse_fail = || -> Result<TagV2, A::Error> {
                let mut data: Vec<String> = Vec::new();
                while let Some(s) = seq.next_element()? {
                    data.push(s);
                }
                Ok(TagV2::Other {
                    tag: tagname.to_string(),
                    data,
                })
            };

            // Parse the main value
            let parts: Vec<&str> = a.split(':').collect();
            if parts.len() < 3 {
                return a_parse_fail();
            }
            let kindnum: u32 = match parts[0].parse::<u32>() {
                Ok(u) => u,
                Err(_) => return a_parse_fail(),
            };
            let kind: EventKind = From::from(kindnum);
            let pubkey: PublicKeyHex = match PublicKeyHex::try_from_str(parts[1]) {
                Ok(pk) => pk,
                Err(_) => return a_parse_fail(),
            };
            let relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Address {
                kind,
                pubkey,
                d: parts[2].to_string(),
                relay_url,
                marker,
                trailing,
            })
        } else if tagname == "content-warning" {
            let msg = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::ContentWarning {
                warning: msg,
                trailing,
            })
        } else if tagname == "delegation" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(pk) => pk,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let conditions: DelegationConditions = match seq.next_element()? {
                Some(c) => c,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string()],
                    });
                }
            };
            let sig: SignatureHex = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string(), conditions.as_string()],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            })
        } else if tagname == "e" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            })
        } else if tagname == "E" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::EventParent {
                id,
                recommended_relay_url,
                trailing,
            })
        } else if tagname == "expiration" {
            let time = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Expiration { time, trailing })
        } else if tagname == "p" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(p) => p,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let petname: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            })
        } else if tagname == "t" {
            let tag = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Hashtag {
                hashtag: tag,
                trailing,
            })
        } else if tagname == "r" {
            let refr: UncheckedUrl = match seq.next_element()? {
                Some(r) => r,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Reference {
                url: refr,
                marker,
                trailing,
            })
        } else if tagname == "g" {
            let geo = match seq.next_element()? {
                Some(g) => g,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Geohash {
                geohash: geo,
                trailing,
            })
        } else if tagname == "d" {
            let id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    // Implicit empty value
                    return Ok(TagV2::Identifier {
                        d: "".to_string(),
                        trailing: Vec::new(),
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Identifier { d: id, trailing })
        } else if tagname == "k" {
            let mut parts: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                parts.push(s);
            }
            if parts.is_empty() {
                return Ok(TagV2::Other {
                    tag: tagname.to_string(),
                    data: parts,
                });
            }
            let kindnum: u32 = match parts[0].parse::<u32>() {
                Ok(u) => u,
                Err(_) => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: parts,
                    });
                }
            };
            let kind: EventKind = From::from(kindnum);
            Ok(TagV2::Kind {
                kind,
                trailing: parts[1..].to_owned(),
            })
        } else if tagname == "subject" {
            let sub = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Subject {
                subject: sub,
                trailing,
            })
        } else if tagname == "nonce" {
            let nonce = match seq.next_element()? {
                Some(n) => n,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let target: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Nonce {
                nonce,
                target,
                trailing,
            })
        } else if tagname == "parameter" {
            let param = match seq.next_element()? {
                Some(s) => s,
                None => "".to_owned(), // implicit parameter
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Parameter { param, trailing })
        } else if tagname == "title" {
            let title = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Title { title, trailing })
        } else {
            let mut data = Vec::new();
            loop {
                match seq.next_element()? {
                    None => {
                        return Ok(TagV2::Other {
                            tag: tagname.to_string(),
                            data,
                        });
                    }
                    Some(s) => data.push(s),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {TagV2, test_tag_serde}

    #[test]
    fn test_a_tag() {
        let tag = TagV2::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            relay_url: Some(UncheckedUrl("wss://relay.mikedilger.com/".to_string())),
            marker: None,
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);

        let tag = TagV2::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            // NOTE, real tags from relays could not get a None here anyways:
            relay_url: Some(UncheckedUrl("".to_owned())),
            marker: Some("reply".to_owned()),
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);

        let json = r#"["a","34550:d0debf9fb12def81f43d7c69429bb784812ac1e4d2d53a202db6aac7ea4b466c:git",""]"#;
        let tag: TagV2 = serde_json::from_str(&json).unwrap();
        if let TagV2::Address { ref relay_url, .. } = tag {
            assert_eq!(*relay_url, Some(UncheckedUrl("".to_string())));
        } else {
            panic!("Tag not an address");
        }
        let json2 = serde_json::to_string(&tag).unwrap();
        assert_eq!(json, json2);
    }
}
```

---

### tag3.rs

**Size:** 20130 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    DelegationConditions, Error, EventKind, Id, NAddr, PublicKey, Signature, UncheckedUrl,
};

/// A tag on an Event
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct TagV3(pub Vec<String>);

impl TagV3 {
    const EMPTY_STRING: &'static str = "";

    /// Create a new tag
    pub fn new(fields: &[&str]) -> TagV3 {
        TagV3(fields.iter().map(|f| (*f).to_owned()).collect())
    }

    /// Create a new tag without copying
    pub fn from_strings(fields: Vec<String>) -> TagV3 {
        TagV3(fields)
    }

    /// Into a `Vec<String>`
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }

    /// Get the string at the given index
    pub fn get_index(&self, index: usize) -> &str {
        if self.0.len() > index {
            &self.0[index]
        } else {
            Self::EMPTY_STRING
        }
    }

    /// Set the string at the given index
    pub fn set_index(&mut self, index: usize, value: String) {
        while self.0.len() <= index {
            self.0.push("".to_owned());
        }
        self.0[index] = value;
    }

    /// Push more values onto the tag
    pub fn push_values(&mut self, mut values: Vec<String>) {
        for value in values.drain(..) {
            self.0.push(value);
        }
    }

    /// Get the tag name for the tag (the first string in the array)
    pub fn tagname(&self) -> &str {
        self.get_index(0)
    }

    /// Get the tag value (index 1, after the tag name)
    pub fn value(&self) -> &str {
        self.get_index(1)
    }

    /// Get the marker (if relevant), else ""
    pub fn marker(&self) -> &str {
        if self.tagname() == "e" {
            self.get_index(3)
        } else if self.tagname() == "a" {
            self.get_index(2)
        } else {
            Self::EMPTY_STRING
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> TagV3 {
        TagV3(vec!["e".to_string(), UncheckedUrl::mock().0])
    }

    /// Create a new 'a' address tag
    pub fn new_address(naddr: &NAddr, marker: Option<String>) -> TagV3 {
        let mut vec = vec![
            "a".to_owned(),
            format!(
                "{}:{}:{}",
                Into::<u32>::into(naddr.kind),
                naddr.author.as_hex_string(),
                naddr.d
            ),
        ];
        if !naddr.relays.is_empty() {
            vec.push(naddr.relays[0].0.clone());
        }
        if let Some(marker) = marker {
            vec.push(marker);
        }
        TagV3(vec)
    }

    /// Parse an 'a' tag
    /// `['a', 'kind:pubkeyhex:d', <optrelay>, <optmarker>]`
    pub fn parse_address(&self) -> Result<(NAddr, Option<String>), Error> {
        let strings = &self.0;

        if strings.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &strings[0] != "a" {
            return Err(Error::TagMismatch);
        }

        let (kind, author, d) = {
            let parts: Vec<&str> = strings[1].split(':').collect();
            if parts.len() < 3 {
                return Err(Error::TagMismatch);
            }
            let kind: EventKind = {
                let kindnum: u32 = parts[0].parse::<u32>()?;
                From::from(kindnum)
            };
            if !kind.is_replaceable() {
                return Err(Error::NonReplaceableAddr);
            }
            let author: PublicKey = PublicKey::try_from_hex_string(parts[1], true)?;
            let d = parts[2].to_string();
            (kind, author, d)
        };

        let relays: Vec<UncheckedUrl> = if strings.len() > 2 {
            vec![UncheckedUrl(strings[2].clone())]
        } else {
            vec![]
        };

        let na = NAddr {
            d,
            relays,
            kind,
            author,
        };

        let marker = if strings.len() >= 4 {
            Some(strings[3].clone())
        } else {
            None
        };

        Ok((na, marker))
    }

    /// Create a "content-warning" tag
    pub fn new_content_warning(warning: &str) -> TagV3 {
        TagV3(vec!["content-warning".to_string(), warning.to_string()])
    }

    /// Parse a "content-warning" tag
    pub fn parse_content_warning(&self) -> Result<Option<String>, Error> {
        if self.0.is_empty() {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "content-warning" {
            return Err(Error::TagMismatch);
        }
        if self.0.len() >= 2 {
            Ok(Some(self.0[1].to_string()))
        } else {
            Ok(None)
        }
    }

    /// Create an "e" tag
    pub fn new_event(
        id: Id,
        recommended_relay_url: Option<UncheckedUrl>,
        marker: Option<String>,
    ) -> TagV3 {
        let mut v: Vec<String> = vec!["e".to_owned(), id.as_hex_string()];
        if let Some(rurl) = recommended_relay_url {
            v.push(rurl.0);
        } else if marker.is_some() {
            v.push("".to_owned())
        }
        if let Some(mark) = marker {
            v.push(mark);
        }
        TagV3(v)
    }

    /// Parse an "e" tag
    /// `['e', <id>, <rurl>, <marker>]`
    pub fn parse_event(&self) -> Result<(Id, Option<UncheckedUrl>, Option<String>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "e" {
            return Err(Error::TagMismatch);
        }
        let id = Id::try_from_hex_string(&self.0[1])?;
        let url = if self.0.len() >= 3 {
            Some(UncheckedUrl(self.0[2].to_owned()))
        } else {
            None
        };
        let marker = if self.0.len() >= 4 {
            Some(self.0[3].to_owned())
        } else {
            None
        };
        Ok((id, url, marker))
    }

    /// Create a "q" tag
    pub fn new_quote(id: Id, recommended_relay_url: Option<UncheckedUrl>) -> TagV3 {
        let mut v: Vec<String> = vec!["q".to_owned(), id.as_hex_string()];
        if let Some(rurl) = recommended_relay_url {
            v.push(rurl.0);
        }
        TagV3(v)
    }

    /// Parse a "q" tag
    pub fn parse_quote(&self) -> Result<(Id, Option<UncheckedUrl>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "q" {
            return Err(Error::TagMismatch);
        }
        let id = Id::try_from_hex_string(&self.0[1])?;
        let url = if self.0.len() >= 3 {
            Some(UncheckedUrl(self.0[2].to_owned()))
        } else {
            None
        };
        Ok((id, url))
    }

    /// Create a "p" tag
    pub fn new_pubkey(
        pubkey: PublicKey,
        relay_url: Option<UncheckedUrl>,
        petname: Option<String>,
    ) -> TagV3 {
        let mut v: Vec<String> = vec!["p".to_owned(), pubkey.as_hex_string()];
        if let Some(rurl) = relay_url {
            v.push(rurl.0);
        } else if petname.is_some() {
            v.push("".to_owned())
        }
        if let Some(pet) = petname {
            v.push(pet);
        }
        TagV3(v)
    }

    /// Parse a "p" tag
    pub fn parse_pubkey(&self) -> Result<(PublicKey, Option<UncheckedUrl>, Option<String>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "p" {
            return Err(Error::TagMismatch);
        }
        let pubkey = PublicKey::try_from_hex_string(&self.0[1], true)?;
        let url = if self.0.len() >= 3 {
            Some(UncheckedUrl(self.0[2].to_owned()))
        } else {
            None
        };
        let petname = if self.0.len() >= 4 {
            if self.0[3].is_empty() {
                None
            } else {
                Some(self.0[3].to_owned())
            }
        } else {
            None
        };
        Ok((pubkey, url, petname))
    }

    /// Create a "t" tag
    pub fn new_hashtag(hashtag: String) -> TagV3 {
        TagV3(vec!["t".to_string(), hashtag])
    }

    /// Parse an "t" tag
    pub fn parse_hashtag(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "t" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create an "r" tag
    pub fn new_relay(url: UncheckedUrl, usage: Option<String>) -> TagV3 {
        let mut v = vec!["r".to_owned(), url.0];
        if let Some(u) = usage {
            v.push(u)
        }
        TagV3(v)
    }

    /// Parse an "r" tag
    pub fn parse_relay(&self) -> Result<(UncheckedUrl, Option<String>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "r" {
            return Err(Error::TagMismatch);
        }
        let relay = UncheckedUrl(self.0[1].clone());
        let marker = if self.0.len() >= 3 {
            Some(self.0[2].clone())
        } else {
            None
        };
        Ok((relay, marker))
    }

    /// Create a new 'd' identifier tag
    pub fn new_identifier(identifier: String) -> TagV3 {
        TagV3(vec!["d".to_string(), identifier])
    }

    /// Parse a 'd' tag
    pub fn parse_identifier(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "d" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a new 'name' tag
    pub fn new_name(name: String) -> TagV3 {
        TagV3(vec!["name".to_string(), name])
    }

    /// Create a new 'image' tag
    pub fn new_image(url: UncheckedUrl, width: Option<u64>, height: Option<u64>) -> TagV3 {
        let mut v = vec!["image".to_owned(), url.0];
        if let Some(w) = width {
            v.push(format!("{}", w));
        }
        if let Some(h) = height {
            v.push(format!("{}", h));
        }
        TagV3(v)
    }

    /// Create a new 'thumb' tag
    pub fn new_thumb(url: UncheckedUrl, width: Option<u64>, height: Option<u64>) -> TagV3 {
        let mut v = vec!["thumb".to_owned(), url.0];
        if let Some(w) = width {
            v.push(format!("{}", w));
        }
        if let Some(h) = height {
            v.push(format!("{}", h));
        }
        TagV3(v)
    }

    /// Create a new 'subject' tag
    pub fn new_subject(subject: String) -> TagV3 {
        TagV3(vec!["subject".to_string(), subject])
    }

    /// Parse a "subject" tag
    pub fn parse_subject(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "subject" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a "nonce" tag
    pub fn new_nonce(nonce: u32, target: Option<u32>) -> TagV3 {
        let mut v = vec!["nonce".to_owned(), format!("{}", nonce)];
        if let Some(targ) = target {
            v.push(format!("{}", targ));
        }
        TagV3(v)
    }

    /// Parse a "nonce" tag
    pub fn parse_nonce(&self) -> Result<(u64, Option<u32>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "nonce" {
            return Err(Error::TagMismatch);
        }
        let nonce = self.0[1].parse::<u64>()?;
        let target = if self.0.len() >= 3 {
            Some(self.0[2].parse::<u32>()?)
        } else {
            None
        };
        Ok((nonce, target))
    }

    /// Create a "title" tag
    pub fn new_title(title: String) -> TagV3 {
        TagV3(vec!["title".to_string(), title])
    }

    /// Parse a "title" tag
    pub fn parse_title(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "title" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a "summary" tag
    pub fn new_summary(summary: String) -> TagV3 {
        TagV3(vec!["summary".to_string(), summary])
    }

    /// Parse a "summary" tag
    pub fn parse_summary(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "summary" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a "k" tag
    pub fn new_kind(kind: EventKind) -> TagV3 {
        TagV3(vec!["k".to_owned(), format!("{}", Into::<u32>::into(kind))])
    }

    /// Parse a "k" tag
    pub fn parse_kind(&self) -> Result<EventKind, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "k" {
            return Err(Error::TagMismatch);
        }
        let u = self.0[1].parse::<u32>()?;
        Ok(u.into())
    }

    /// New delegation tag
    pub fn new_delegation(
        pubkey: PublicKey,
        conditions: DelegationConditions,
        sig: Signature,
    ) -> TagV3 {
        TagV3(vec![
            "delegation".to_owned(),
            pubkey.as_hex_string(),
            conditions.as_string(),
            sig.as_hex_string(),
        ])
    }

    /// parse delegation tag
    pub fn parse_delegation(&self) -> Result<(PublicKey, DelegationConditions, Signature), Error> {
        if self.0.len() < 4 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "delegation" {
            return Err(Error::TagMismatch);
        }
        let pk = PublicKey::try_from_hex_string(&self.0[1], true)?;
        let conditions = DelegationConditions::try_from_str(&self.0[2])?;
        let sig = Signature::try_from_hex_string(&self.0[3])?;
        Ok((pk, conditions, sig))
    }

    /// New proxy tag
    pub fn proxy(protocol: String, id: String) -> TagV3 {
        TagV3(vec!["proxy".to_owned(), protocol, id])
    }

    /// Create a generic tag with a name and value
    pub fn new_tag(tagname: &str, value: &str) -> TagV3 {
        TagV3(vec![tagname.to_owned(), value.to_owned()])
    }

    /// parse proxy tag
    pub fn parse_proxy(&self) -> Result<(String, String), Error> {
        if self.0.len() < 3 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "proxy" {
            return Err(Error::TagMismatch);
        }
        let protocol = self.0[1].to_owned();
        let id = self.0[2].to_owned();
        Ok((protocol, id))
    }
}

impl fmt::Display for TagV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[({})]", self.0.join(", "))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {TagV3, test_tag_serde}

    #[test]
    fn test_a_tag() {
        let na = NAddr {
            d: "blog-20231029".to_owned(),
            relays: vec![UncheckedUrl("badurl".to_owned())],
            kind: EventKind::LongFormContent,
            author: PublicKey::mock_deterministic(),
        };

        let tag = TagV3::new_address(&na, None);
        let (na2, _optmarker) = tag.parse_address().unwrap();
        // Equal only because there is just 1 UncheckedUrl, else might have dropped
        // the rest
        assert_eq!(na, na2);

        // Test a known JSON a tag:
        let json =
            r#"["a","34550:d0debf9fb12def81f43d7c69429bb784812ac1e4d2d53a202db6aac7ea4b466c:git"]"#;
        let tag: TagV3 = serde_json::from_str(&json).unwrap();
        assert!(tag.parse_address().is_ok());

        let tag = TagV3::new(&[
            "a",
            "30023:b12b632c887f0c871d140d37bcb6e7c1e1a80264d0b7de8255aa1951d9e1ff79:1716928135712",
            "",
            "root",
        ]);
        let (_, marker) = tag.parse_address().unwrap();
        assert!(marker.as_deref().unwrap() == "root");
    }

    #[test]
    fn test_content_warning_tag() {
        let tag = TagV3::new(&["content-warning"]);
        assert_eq!(tag.parse_content_warning().unwrap(), None);

        let tag = TagV3::new_content_warning("danger");
        assert_eq!(
            tag.parse_content_warning().unwrap(),
            Some("danger".to_owned())
        );

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_content_warning().is_err());
    }

    #[test]
    fn test_event_tag() {
        let tag = TagV3::new_event(Id::mock(), None, None);
        assert_eq!(tag.parse_event().unwrap(), (Id::mock(), None, None));

        let data = (
            Id::mock(),
            Some(UncheckedUrl("dummy".to_owned())),
            Some("foo".to_owned()),
        );
        let tag = TagV3::new_event(data.0, data.1.clone(), data.2.clone());
        assert_eq!(tag.parse_event().unwrap(), data);
    }

    #[test]
    fn test_pubkey_tag() {
        let tag = TagV3::new_pubkey(PublicKey::mock_deterministic(), None, None);
        assert_eq!(
            tag.parse_pubkey().unwrap(),
            (PublicKey::mock_deterministic(), None, None)
        );

        let data = (
            PublicKey::mock(),
            Some(UncheckedUrl("dummy".to_owned())),
            Some("foo".to_owned()),
        );
        let tag = TagV3::new_pubkey(data.0, data.1.clone(), data.2.clone());
        assert_eq!(tag.parse_pubkey().unwrap(), data);
    }

    #[test]
    fn test_hashtag_tag() {
        let tag = TagV3::new(&["t"]);
        assert!(tag.parse_hashtag().is_err());

        let tag = TagV3::new_hashtag("footstr".to_owned());
        assert_eq!(tag.parse_hashtag().unwrap(), "footstr".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_hashtag().is_err());
    }

    #[test]
    fn test_relay_tag() {
        let tag = TagV3::new(&["r", "wss://example.com", "read"]);
        let parsed = tag.parse_relay().unwrap();
        let data = (
            UncheckedUrl("wss://example.com".to_owned()),
            Some("read".to_owned()),
        );
        assert_eq!(parsed, data);

        let tag2 = TagV3::new_relay(data.0, data.1);
        assert_eq!(tag, tag2);
    }

    #[test]
    fn test_identifier_tag() {
        let tag = TagV3::new(&["d"]);
        assert!(tag.parse_identifier().is_err());

        let tag = TagV3::new_identifier("myblog123".to_owned());
        assert_eq!(tag.parse_identifier().unwrap(), "myblog123".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_identifier().is_err());
    }

    #[test]
    fn test_subject_tag() {
        let tag = TagV3::new(&["subject"]);
        assert!(tag.parse_subject().is_err());

        let tag = TagV3::new_subject("Attn: Nurses".to_owned());
        assert_eq!(tag.parse_subject().unwrap(), "Attn: Nurses".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_subject().is_err());
    }

    #[test]
    fn test_nonce_tag() {
        let tag = TagV3::new(&["nonce"]);
        assert!(tag.parse_nonce().is_err());

        let tag = TagV3::new_nonce(132345, Some(20));
        assert_eq!(tag.parse_nonce().unwrap(), (132345, Some(20)));

        let tag = TagV3::new_nonce(132345, None);
        assert_eq!(tag.parse_nonce().unwrap(), (132345, None));

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_nonce().is_err());
    }

    #[test]
    fn test_title_tag() {
        let tag = TagV3::new(&["title"]);
        assert!(tag.parse_title().is_err());

        let tag = TagV3::new_title("Attn: Nurses".to_owned());
        assert_eq!(tag.parse_title().unwrap(), "Attn: Nurses".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_title().is_err());
    }

    #[test]
    fn test_kind_tag() {
        let tag = TagV3::new(&["k", "30023"]);
        assert_eq!(tag.parse_kind().unwrap(), EventKind::LongFormContent);

        let tag = TagV3::new(&["k"]);
        assert!(tag.parse_kind().is_err());

        let tag = TagV3::new_kind(EventKind::ZapRequest);
        assert_eq!(tag.parse_kind().unwrap(), EventKind::ZapRequest);
    }
}
```

---

### zap_data.rs

**Size:** 1035 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use crate::types::{EventReference, Id, MilliSatoshi, PublicKey};

/// Data about a Zap
#[derive(Clone, Debug)]
pub struct ZapDataV2 {
    /// The event that was zapped. If missing we can't use the zap receipt
    /// event.
    pub zapped_event: EventReference,

    /// The amount that the event was zapped
    pub amount: MilliSatoshi,

    /// The public key of the person who received the zap
    pub payee: PublicKey,

    /// The public key of the person who paid the zap, if it was in the receipt
    pub payer: PublicKey,

    /// The public key of the zap provider, for verification purposes
    pub provider_pubkey: PublicKey,
}

/// Data about a Zap
#[derive(Clone, Debug, Copy)]
pub struct ZapDataV1 {
    /// The event that was zapped
    pub id: Id,

    /// The amount that the event was zapped
    pub amount: MilliSatoshi,

    /// The public key of the person who provided the zap
    pub pubkey: PublicKey,

    /// The public key of the zap provider, for verification purposes
    pub provider_pubkey: PublicKey,
}
```

---


---
*Generated by code2prompt.sh on 2026-01-21 13:49:15*
