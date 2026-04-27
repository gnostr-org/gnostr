use log::debug;
use nostr_sdk::prelude::RelayUrl;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ActiveRelayList {
    active_relays: Arc<Mutex<Vec<RelayUrl>>>,
}

impl ActiveRelayList {
    pub fn new() -> Self {
        Self {
            active_relays: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_relay(&self, relay_url: RelayUrl) {
        let mut active_relays = self.active_relays.lock().unwrap();
        if !active_relays.contains(&relay_url) {
            debug!("Adding relay to active list: {}", relay_url);
            active_relays.push(relay_url);
        }
    }

    pub fn remove_relay(&self, relay_url: &RelayUrl) {
        let mut active_relays = self.active_relays.lock().unwrap();
        let initial_len = active_relays.len();
        active_relays.retain(|r| r != relay_url);
        if active_relays.len() < initial_len {
            debug!("Removed relay from active list: {}", relay_url);
        }
    }

    pub fn get_active_relays(&self) -> Vec<RelayUrl> {
        self.active_relays.lock().unwrap().clone()
    }

}

impl Default for ActiveRelayList {
    fn default() -> Self {
        Self::new()
    }
}
