use crate::processor::Processor;
use crate::relays::record_live_kind;
use crate::relays::Relays;

use nostr_sdk::{
    prelude::{
        Client, Event, EventBuilder, Filter, Keys, Kind, RelayPoolNotification, RelayUrl, Result,
        TagStandard, Timestamp,
    },
    RelayMessage, RelayStatus,
};
use std::collections::HashSet;
use std::time::Duration;

use log::debug;
use log::info;
use log::trace;
use log::warn;

const MAX_ACTIVE_RELAYS: usize = 3; //usize::MAX;
const PERIOD_START_PAST_SECS: u64 = 6 * 60 * 60;

mod active;
mod sources;

use active::ActiveRelayList;
use sources::{is_shitlisted, load_relay_sources, seed_bootstrap_relays};

/// Keeps a set of active connections to relays
pub struct RelayManager {
    // app_keys: Keys,
    pub relays: Relays,
    relay_client: Client,
    pub processor: Processor,
    /// Time of last event seen (real time, Unix timestamp)
    time_last_event: u64,
    active_relay_list: ActiveRelayList,
}

impl RelayManager {
    pub async fn new(app_keys: Keys, processor: Processor) -> Self {
        let relay_client = Client::new(app_keys);
        let relays_instance = load_relay_sources().await;

        Self {
            // app_keys,
            relays: relays_instance,
            relay_client,
            processor,
            time_last_event: Self::now(),
            active_relay_list: ActiveRelayList::new(),
        }
    }

    fn add_bootstrap_relays_if_needed(&mut self, bootstrap_relays: Vec<&str>) {
        debug!("relay_manager::add_bootstrap_relays_if_needed");
        if self.relays.count() >= MAX_ACTIVE_RELAYS {
            return;
        }
        seed_bootstrap_relays(&mut self.relays, bootstrap_relays.as_slice());
    }

    async fn add_some_relays(&mut self) -> Result<()> {
        debug!("relay_manager::add_some_relays");
        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);
        for r in some_relays {
            debug!("r={}", &r);
            self.relay_client.add_relay(r.clone()).await?;
            self.relay_client
                .send_event_builder(EventBuilder::text_note(format!("{}", r)))
                .await?;
        }
        Ok(())
    }

    pub async fn run(&mut self, bootstrap_relays: Vec<&str>) -> Result<()> {
        debug!("relay_manager::run");
        self.add_bootstrap_relays_if_needed(bootstrap_relays);
        self.add_some_relays().await?;

        let active_relay_list = self.active_relay_list.clone();
        let relay_client = self.relay_client.clone();
        tokio::spawn(async move {
            Self::monitor_relays(active_relay_list, relay_client).await;
        });

        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);
        for url in &some_relays {
            if is_shitlisted(url.as_str()) {
                debug!("Skipping shitlisted relay: {}", url);
                continue;
            }
            self.relay_client.add_relay(url.to_string()).await?;
        }
        self.connect().await?;
        self.wait_and_handle_messages().await?;
        debug!("relay_manager::run::self.relays.dump_list()");
        self.relays.dump_list(); //TODO convert relays.dump_list to relays.yaml write operation
                                 //self.relays.print();
                                 //let get_some = self.relays.get_some(50);
                                 //for url in get_some { println!("url={}", url.to_string());}
        let get_all = self.relays.get_all();
        for relay in get_all {
            debug!("relay_manager::run::184 relay={} ", relay);
        }
        Ok(())
    }

    async fn monitor_relays(active_relay_list: ActiveRelayList, relay_client: Client) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.tick().await;

        loop {
            interval.tick().await;
            let mut active = Vec::new();
            let relays = relay_client.relays().await;
            for (url, relay_handle) in relays.into_iter() {
                match relay_handle.status() {
                    RelayStatus::Connected | RelayStatus::Connecting => {
                        active_relay_list.add_relay(url.clone());
                        active.push(url);
                    }
                    _ => {
                        active_relay_list.remove_relay(&url);
                    }
                }
            }
            debug!("monitor_relays: {} active relays", active.len());
        }
    }

    async fn connect(&mut self) -> Result<()> {
        debug!("relay_manager::connect");
        let relays = self.relay_client.relays().await;
        info!("Connecting to {} relays ...", relays.len());
        for u in relays.keys() {
            debug!("u={:?} ", u.to_string())
        }
        debug!("\n");

        // Initiate connection
        self.relay_client.connect().await;

        // Wait for all relays to be connected
        let mut all_connected = false;
        let mut attempts = 0;
        let max_attempts = 2; // Try for 10 seconds (10 * 1 second sleep)

        while !all_connected && attempts < max_attempts {
            all_connected = true;
            let relays = self.relay_client.relays().await;
            for relay in relays.values() {
                match relay.status() {
                    RelayStatus::Connected => {
                        info!("Relay {} is connected.", relay.url().to_string());
                    }
                    RelayStatus::Disconnected
                    | RelayStatus::Terminated
                    | RelayStatus::Connecting => {
                        warn!(
                            "Relay {} is not yet connected. Status: {:?}",
                            relay.url().to_string(),
                            relay.status()
                        );
                        all_connected = false;
                    }
                    _ => {
                        warn!(
                            "Relay {} has unknown status: {:?}",
                            relay.url().to_string(),
                            relay.status()
                        );
                        all_connected = false;
                    }
                }
            }

            if !all_connected {
                tokio::time::sleep(Duration::from_secs(1)).await;
                attempts += 1;
            }
        }

        if all_connected {
            debug!("All relays connected.");
            Ok(())
        } else {
            warn!(
                "Failed to connect to all relays after {} attempts.",
                max_attempts
            );
            //TODO append to shitlist.yaml in user space
            Ok(())
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.relay_client.disconnect().await;
        debug!("Disconnected");
        Ok(())
    }

    async fn subscribe(&mut self, time_start: Timestamp, time_end: Timestamp) -> Result<()> {
        self.relay_client
            .subscribe(
                Filter::new()
                // .pubkey(keys.public_key())
                // .kind(Kind::RecommendRelay)
                .kinds(vec![Kind::ContactList, Kind::RecommendRelay])
                .since(time_start)
                .until(time_end),
                None,
            )
            .await?;
        debug!("Subscribed to relay events",);
        self.relay_client
            .send_event_builder(EventBuilder::text_note(format!("{}", time_start)))
            .await?;
        self.relay_client
            .send_event_builder(EventBuilder::text_note(format!("{}", time_end)))
            .await?;
        Ok(())
    }

    async fn unsubscribe(&mut self) -> Result<()> {
        self.relay_client.unsubscribe_all().await;
        debug!("Unsubscribed from relay events ...");
        Ok(())
    }

    async fn reconnect(&mut self) -> Result<()> {
        let connected_relays = self.relay_client.relays().await.len();
        let available_relays = self.relays.count();
        if connected_relays < MAX_ACTIVE_RELAYS && available_relays > connected_relays {
            debug!(
                "connected_relays={} available_relays={}",
                connected_relays, available_relays
            );
            self.disconnect().await?;
            self.add_some_relays().await?;
            self.connect().await?;
            self.relay_client
                .send_event_builder(EventBuilder::text_note(format!("{}", connected_relays)))
                .await?;
            self.relay_client
                .send_event_builder(EventBuilder::text_note(format!("{}", available_relays)))
                .await?;
        }
        Ok(())
    }

    async fn wait_and_handle_messages(&mut self) -> Result<()> {
        // Keep track of relays with EOSE sent
        let mut eose_relays = HashSet::new();

        let now = Timestamp::now();
        let period_end = now;
        let period_start = period_end - Duration::from_secs(PERIOD_START_PAST_SECS);
        self.subscribe(period_start, period_end).await?;

        let mut notifications = self.relay_client.notifications();
        while let Ok(notification) = notifications.recv().await {
            trace!(
                "relay_manager::wait_and_handle_messages::relaynotif {:?}",
                notification
            );
            match notification {
                RelayPoolNotification::Event {
                    relay_url: _,
                    subscription_id: _,
                    event,
                } => {
                    self.handle_event(&event); //self.handle_event
                                               // invoke callback
                    self.processor.handle_event(&event); //self.processor.handle_event
                }
                RelayPoolNotification::Message {
                    relay_url: url,
                    message: relaymsg,
                } => match relaymsg {
                    RelayMessage::EndOfStoredEvents(_sub_id) => {
                        eose_relays.insert(url.clone());
                        let n1 = eose_relays.len();
                        let n2 = self.relay_client.relays().await.len();
                        let mut n_connected = 0;
                        let mut n_connecting = 0;
                        let relays = self.relay_client.relays().await;
                        for relay in relays.values() {
                            match relay.status() {
                                RelayStatus::Connected => n_connected += 1,
                                RelayStatus::Connecting => n_connecting += 1,
                                _ => {}
                            }
                        }
                        debug!("Received EOSE from {url}, total {n1} ({n2} relays, {n_connected} connected {n_connecting} connecting)");

                        // Check for stop: All connected/connecting relays have signalled EOSE, or
                        if n1 >= (n_connected + n_connecting) && (n_connected + n_connecting > 0) {
                            debug!("STOPPING; All relays signalled EOSE ({n1})");
                            break;
                        }
                    }
                    RelayMessage::Event {
                        subscription_id: _,
                        event: _,
                    } => {}
                    _ => {
                        debug!(
                            "Received unhandled relay message from {url}: {{\"{:?}\":\"{url}\"}}",
                            relaymsg
                        );
                    }
                },
                RelayPoolNotification::Shutdown => break,
            }
            // Check for stop: There was no event in the last few seconds, and there were some EOSE already
            let last_age = self.get_last_event_ago();
            let n1 = eose_relays.len();
            if last_age > 20 && n1 >= 2 {
                debug!(
                    "STOPPING; There were some EOSE-s, and no events in the past {} secs",
                    last_age
                );
                break;
            }

            self.reconnect().await?;
        }
        self.unsubscribe().await?;
        self.disconnect().await?;
        Ok(())
    }

    //#[allow(unused_variables)]
    fn handle_event(&mut self, event: &Event) {
        record_live_kind(format!("{:?}", event.kind));
        match event.kind {
            Kind::Metadata => {
                debug!("{:?}", event.kind);
            }
            Kind::TextNote => {
                debug!("{:?}", event.kind);
            }
            Kind::EncryptedDirectMessage => {
                debug!("{:?}", event.kind);
            }
            Kind::EventDeletion => {
                debug!("{:?}", event.kind);
            }
            Kind::Repost => {
                debug!("{:?}", event.kind);
            }
            Kind::Reaction => {
                debug!("{:?}", event.kind);
            }
            Kind::ChannelCreation => {
                debug!("{:?}", event.kind);
            }
            Kind::ChannelMetadata => {
                debug!("{:?}", event.kind);
            }
            Kind::ChannelMessage => {
                debug!("{:?}", event.kind);
            }
            Kind::ChannelHideMessage => {
                debug!("{:?}", event.kind);
            }
            Kind::ChannelMuteUser => {
                debug!("{:?}", event.kind);
            }
            Kind::PublicChatReserved45 => {
                debug!("{:?}", event.kind);
            }
            Kind::PublicChatReserved46 => {
                debug!("{:?}", event.kind);
            }
            Kind::PublicChatReserved47 => {
                debug!("{:?}", event.kind);
            }
            Kind::PublicChatReserved48 => {
                debug!("{:?}", event.kind);
            }
            Kind::PublicChatReserved49 => {
                debug!("{:?}", event.kind);
            }
            Kind::Reporting => {
                debug!("{:?}", event.kind);
            }
            Kind::ZapRequest => {
                debug!("{:?}", event.kind);
            }
            Kind::ZapReceipt => {
                debug!("{:?}", event.kind);
            }
            Kind::Authentication => {
                debug!("{:?}", event.kind);
            }
            Kind::NostrConnect => {
                debug!("{:?}", event.kind);
            }
            Kind::LongFormTextNote => {
                debug!("{:?}", event.kind);
                self.update_event_time();
                // count p tags
                for (mut _cnt, _t) in event.tags.iter().enumerate() {
                    //if let Tag::PubKey(_pk, Some(ss)) = t {
                    //  state.pubkeys.add(pk);
                    //if let Some(ss) = s {
                    //debug!("    {ss}");
                    //let _ = self.relays.add(ss);
                    //}
                    debug!("    {_cnt}");
                    _cnt += 1;
                    //}
                }
            }
            Kind::RelayList => {
                self.update_event_time();
                debug!("relay_manager::Kind::RelayList={:?}", event.kind);

                match serde_json::from_str::<serde_json::Value>(&event.content) {
                    Ok(json_value) => {
                        if let Some(relays_map) = json_value.as_object() {
                            for (url_str, _read_write) in relays_map {
                                if self.relays.add(url_str) {
                                    trace!("Added relay from Kind::RelayList: {}", url_str);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse Kind::RelayList event content as JSON: {}. Content: {}", e, event.content);
                    }
                }
            }
            kind if kind.is_replaceable() => {
                debug!("{:?}", event.kind);
            }
            kind if kind.is_ephemeral() => {
                debug!("{:?}", event.kind);
            }
            kind if kind.is_addressable() => {
                debug!("{:?}", event.kind);
            }
            Kind::Custom(_u64) => {
                debug!("{:?}", event.kind);
            }
            Kind::ContactList => {
                self.update_event_time();
                // count p tags
                let mut count = 0;
                for _t in event.tags.iter() {
                    if let Some(TagStandard::PublicKey {
                        public_key: _pk,
                        relay_url: Some(ss),
                        alias: _,
                        uppercase: _,
                    }) = _t.as_standardized()
                    {
                        //state.pubkeys.add(pk); //TODO neccesary?
                        //if let Some(ss) = s {
                        trace!("    {ss}");
                        let _ = self.relays.add(ss.as_str());
                        //}
                        trace!("    {}", count);
                        count += 1;
                    }
                }
            }
            Kind::RecommendRelay => {
                self.update_event_time();
                debug!("\n490:Relay(s): {}\n", event.content);
                let _ = self.relays.add(&event.content);
            }
            _ => {
                debug!("{:?}", event.kind);
            }
        }
    }

    fn update_event_time(&mut self) {
        self.time_last_event = Self::now();
    }

    fn get_last_event_ago(&self) -> u64 {
        Self::now() - self.time_last_event
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
