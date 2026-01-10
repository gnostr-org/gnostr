//! NIP-38: User Statuses
//!
//! This NIP defines how to set a user's status message, which can be short
//! text or an indicator of activity, displayed next to their username.
//! It uses a Parameterized Replaceable Event (Kind 30315).
//!
//! https://github.com/nostr-protocol/nips/blob/master/38.md

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

use crate::types::{Event, Id, PreEvent, PublicKey, Tag, Unixtime}; // Re-using existing types
use crate::types::{event_kind::EventKind, signature::Signature};

/// NIP-38 User Status Event Kind (Parameterized Replaceable Event)
pub const USER_STATUS_KIND: u32 = 30315;

/// Represents the type of user status, defined by the 'd' tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatusType {
    /// General status, e.g., "Working", "Hiking".
    General,
    /// Music status, indicating what the user is listening to.
    Music,
    /// A custom status type not explicitly defined by NIP-38.
    Custom(String),
}

impl ToString for UserStatusType {
    fn to_string(&self) -> String {
        match self {
            UserStatusType::General => "general".to_string(),
            UserStatusType::Music => "music".to_string(),
            UserStatusType::Custom(s) => s.clone(),
        }
    }
}

impl From<&str> for UserStatusType {
    fn from(s: &str) -> Self {
        match s {
            "general" => UserStatusType::General,
            "music" => UserStatusType::Music,
            _ => UserStatusType::Custom(s.to_string()),
        }
    }
}

/// Represents the content of a NIP-38 User Status Event (Kind 30315).
/// The content is a simple string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserStatusContent {
    pub message: String,
}

/// Helper functions for NIP-38 events.
pub trait NIP38Event {
    /// Returns the status type from the 'd' tag.
    fn status_type(&self) -> Option<UserStatusType>;
    /// Returns the expiration timestamp from the 'expiration' tag (NIP-40).
    fn expiration(&self) -> Option<Unixtime>;
    /// Creates a NIP-38 User Status event builder.
    fn new_user_status(
        public_key: PublicKey,
        message: String,
        status_type: UserStatusType,
        expiration: Option<Unixtime>,
        // Other optional tags like r, p, e, a can be added here
    ) -> Result<Event>; // Returns a full Event for now, can be an EventBuilder later.
}

impl NIP38Event for Event {
    fn status_type(&self) -> Option<UserStatusType> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "d" {
                Some(UserStatusType::from(tag.0[1].as_str()))
            } else {
                None
            }
        })
    }

    fn expiration(&self) -> Option<Unixtime> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "expiration" {
                tag.0[1].parse::<i64>().ok().map(Unixtime::from)
            } else {
                None
            }
        })
    }

    fn new_user_status(
        public_key: PublicKey,
        message: String,
        status_type: UserStatusType,
        expiration: Option<Unixtime>,
        // Other optional tags like r, p, e, a can be added here
    ) -> Result<Event> {
        let content = UserStatusContent { message }.message; // Content is just the message string

        let mut tags: Vec<Tag> = vec![Tag::new(&["d", &status_type.to_string()])];

        if let Some(exp) = expiration {
            tags.push(Tag::new(&["expiration", &exp.to_string()]));
        }

        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Replaceable(USER_STATUS_KIND),
            tags: tags.clone(),       // Clone tags for PreEvent
            content: content.clone(), // Clone content for PreEvent
        };

        let id = pre_event.hash().unwrap(); // Calculate event ID

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Replaceable(USER_STATUS_KIND),
            tags,
            content,
            sig: Signature::zeroes(), // Signature will be added during signing
        })
    }
}
