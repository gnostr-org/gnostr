//! NIP-53: Live Activities
//!
//! This NIP defines how to represent live activities (streaming, meetings)
//! on Nostr.
//!
//! Event Kinds:
//! - 30311: Live Streaming Event (Parameterized Replaceable Event)
//! - 30312: Meeting Space Event (Parameterized Replaceable Event)
//! - 30313: Meeting Room Events (Parameterized Replaceable Event)
//! - 10312: Room Presence (Regular Event)
//!
//! Data structures are defined using `serde` for serialization/deserialization.

use serde::{Deserialize, Serialize};

use crate::types::{Event, Id, PublicKey, Tag, Unixtime}; // Re-using existing types

/// NIP-53 Live Activity Kinds
pub const LIVE_STREAMING_EVENT_KIND: u32 = 30311;
pub const MEETING_SPACE_EVENT_KIND: u32 = 30312;
pub const MEETING_ROOM_EVENT_KIND: u32 = 30313;
pub const ROOM_PRESENCE_KIND: u32 = 10312;

/// Common fields for parameterized replaceable NIP-53 events (kinds 30311,
/// 30312, 30313) These events typically include a 'd' tag for a unique
/// identifier.
pub trait NIP53ParameterizedReplaceable {
    fn d_tag(&self) -> Option<&str>;
    fn identifier(&self) -> String; // A unique identifier derived from event content/tags
}

/// Represents the content of a NIP-53 Live Streaming Event (Kind 30311)
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#live-streaming-event-kind-30311
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveStreamingEventContent {
    /// Name of the stream.
    pub title: String,
    /// Description of the stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// URL of the stream.
    pub stream_url: String,
    /// URL of the a chat associated with the stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_url: Option<String>,
    /// The stream status ("live", "ended", etc.)
    pub status: String, // TODO: Use an enum for status
}

/// Helper struct for NIP-53 Meeting Space Event (Kind 30312) content.
/// This represents a virtual space for meetings.
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#meeting-space-event-kind-30312
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingSpaceEventContent {
    /// Name of the meeting space.
    pub name: String,
    /// Description of the meeting space.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// URL to join the meeting space.
    pub join_url: String,
    /// The current status of the meeting space ("open", "closed", etc.)
    pub status: String, // TODO: Use an enum for status
}

/// Represents the content of a NIP-53 Meeting Room Event (Kind 30313)
/// This defines a scheduled meeting within a space.
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#meeting-room-event-kind-30313
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingRoomEventContent {
    /// Name of the meeting room.
    pub name: String,
    /// Description of the meeting room.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// URL to join this specific meeting room.
    pub join_url: String,
    /// Start time of the meeting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<Unixtime>,
    /// End time of the meeting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<Unixtime>,
    /// The current status of the meeting room ("scheduled", "in-progress",
    /// "ended", etc.)
    pub status: String, // TODO: Use an enum for status
}

/// Represents the content of a NIP-53 Room Presence Event (Kind 10312)
/// This event signals a user's presence in a specific room/space.
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#room-presence-event-kind-10312
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomPresenceEventContent {
    /// The identifier of the room/space the user is present in. This typically
    /// corresponds to the 'd' tag of a Kind 30312 or 30313 event.
    pub room_id: String,
    /// The status of the user's presence ("active", "away", "left", etc.)
    pub status: String, // TODO: Use an enum for status
}

impl NIP53ParameterizedReplaceable for Event {
    fn d_tag(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "d" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn identifier(&self) -> String {
        // As per NIP-53, the 'd' tag is the unique identifier for PRs.
        // For events without a 'd' tag (e.g., Kind 10312), a suitable fallback
        // would be the event ID itself, or a hash of relevant content.
        // For parameterized replaceable events, a 'd' tag is REQUIRED.
        self.d_tag()
            .map_or_else(|| self.id.as_hex_string(), |d| d.to_string())
    }
}
