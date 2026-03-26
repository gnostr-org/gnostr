use serde::{Deserialize, Serialize};

/// Relay information document as described in NIP-11, supplied by a relay.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelayInfo {
    /// Contact email of the relay operator.
    pub contact: String,
    /// Description of the relay.
    pub description: String,
    /// Name of the relay.
    pub name: String,
    /// Name of the relay software.
    pub software: String,
    /// NIPs supported by the relay.
    pub supported_nips: Vec<i32>,
    /// Version of the relay software.
    pub version: String,
}
