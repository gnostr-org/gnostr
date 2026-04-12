use crate::types::{
    Id,
    versioned::{
        event3::{EventV3, PreEventV3, RumorV3, UnsignedEventV3},
        zap_data::ZapDataV2,
    },
};

/// The main event type
pub type Event = EventV3;

/// The event ID
pub(crate) type EventId = Id;

/// Data used to construct an event
pub type PreEvent = PreEventV3;

/// An UnsignedEvent is an Event without a signature
pub(crate) type UnsignedEvent = UnsignedEventV3;

/// A Rumor is an Event without a signature
pub type Rumor = RumorV3;

/// Data about a Zap
pub type ZapData = ZapDataV2;
