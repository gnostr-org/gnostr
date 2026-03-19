#[allow(unused_imports)]
use crate::crawler::processor::Processor;
use crate::crawler::relays::Relays;
use crate::crawler::CliArgs;
use crate::crawler::APP_SECRET_KEY;
use nostr_sdk_0_34_0::{
    prelude::{
        Client, Event, Filter, Keys, Kind, Options, RelayPoolNotification, Result, /*Tag, */Timestamp,
        Url, TagStandard
    },
    RelayMessage, RelayStatus,
};
use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::mpsc;

use clap::Parser;

use git2::Repository;
use std::str;

use log::debug;
use log::info;
use log::trace;

const MAX_ACTIVE_RELAYS: usize = 2; //usize::MAX;
const PERIOD_START_PAST_SECS: u64 = 6 * 60 * 60;

use std::sync::{Arc, Mutex};

/// A thread-safe, shared list of active Nostr relays.
#[derive(Clone)]
pub struct ActiveRelayList {
    active_relays: Arc<Mutex<Vec<Url>>>,
}

impl ActiveRelayList {
    /// Creates a new, empty ActiveRelayList.
    pub fn new() -> Self {
        Self {
            active_relays: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a relay to the active list.
    pub fn add_relay(&self, relay_url: Url) {
        let mut active_relays = self.active_relays.lock().unwrap();
        if !active_relays.contains(&relay_url) {
            debug!("Adding relay to active list: {}", relay_url);
            active_relays.push(relay_url);
        }
    }

    /// Removes a relay from the active list.
    pub fn remove_relay(&self, relay_url: &Url) {
        let mut active_relays = self.active_relays.lock().unwrap();
        let initial_len = active_relays.len();
        active_relays.retain(|r| r != relay_url);
        if active_relays.len() < initial_len {
            debug!("Removed relay from active list: {}", relay_url);
        }
    }

    /// Returns a clone of the current list of active relays.
    pub fn get_active_relays(&self) -> Vec<Url> {
        debug!("Getting current active relays.");
        self.active_relays.lock().unwrap().clone()
    }
}

impl Default for ActiveRelayList {
    fn default() -> Self {
        Self::new()
    }
}

/// Keeps a set of active connections to relays
pub struct RelayManager {
    pub relays: Relays,
    relay_client: Client,
    pub processor: Processor,
    time_last_event: u64,
    active_relay_list: ActiveRelayList,
    update_sender: tokio::sync::mpsc::Sender<Vec<Url>>,
}

impl RelayManager {
    pub fn new(
        app_keys: Keys,
        processor: Processor,
        active_relay_list: ActiveRelayList,
        update_sender: tokio::sync::mpsc::Sender<Vec<Url>>,
    ) -> Self {
        let opts = Options::new(); //.wait_for_send(false);
        let relay_client = Client::with_opts(&app_keys, opts);
        let _proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));
        Self {
            relays: Relays::new(),
            relay_client,
            processor,
            time_last_event: Self::now(),
            active_relay_list,
            update_sender,
        }
    }

    fn add_bootstrap_relays_if_needed(&mut self, bootstrap_relays: Vec<&str>) {
        for us in &bootstrap_relays {
            if self.relays.count() >= MAX_ACTIVE_RELAYS {
                return;
            }
            self.relays.add(us);
        }
    }

    async fn add_some_relays(&mut self) -> Result<()> {
        // remove all
        loop {
            let relays = self.relay_client.relays().await;
            let relay_urls: Vec<&Url> = relays.keys().collect();
            if relay_urls.is_empty() {
                break;
            }
            self.relay_client
                .remove_relay(relay_urls[0].to_string())
                .await?;
        }
        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);

        let args = CliArgs::parse();

        let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
        let repo = Repository::discover(path)?;
        let revwalk = repo.revwalk()?;
        for commit in revwalk {
            println!("\n\n\n\n\n{:?}\n\n\n\n", commit);
        }

        for r in some_relays {
            self.relay_client.add_relay(r.clone()).await?;
        }
        Ok(())
    }

    pub async fn run(&mut self, bootstrap_relays: Vec<&str>) -> Result<()> {
        self.add_bootstrap_relays_if_needed(bootstrap_relays);
        self.add_some_relays().await?;

        let active_relay_list_clone = self.active_relay_list.clone();
        let relay_client_clone = self.relay_client.clone();
        let update_sender_clone = self.update_sender.clone();

        tokio::spawn(async move {
            Self::monitor_relays(
                active_relay_list_clone,
                relay_client_clone,
                update_sender_clone,
            ).await;
        });

        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);
        for url in &some_relays {
            //if url NOT contain
            if !url.as_str().contains("")
                || !url.as_str().contains("")
                || !url.as_str().contains("")
            {
                self.relay_client.add_relay(url.to_string()).await?;
            }
        }
        self.connect().await?;
        self.wait_and_handle_messages().await?;
        ////self.relays.dump_list();
        self.relays.print();
        Ok(())
    }

    async fn monitor_relays(
        active_relay_list: ActiveRelayList,
        relay_client: Client,
        update_sender: mpsc::Sender<Vec<Url>>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        interval.tick().await; // consume the first immediate tick

        loop {
            interval.tick().await;
            debug!("Monitoring relays...");

            let mut current_active_relays = Vec::new();
            let relays = relay_client.relays().await;
            for (url, relay_handle) in relays.into_iter() {
                match relay_handle.status().await {
                    RelayStatus::Connected => {
                        debug!("Relay {} is connected.", url);
                        active_relay_list.add_relay(url.clone());
                        current_active_relays.push(url);
                    },
                    RelayStatus::Connecting => {
                        debug!("Relay {} is connecting.", url);
                        // Consider connecting relays as active for now, or add a separate state.
                        active_relay_list.add_relay(url.clone());
                        current_active_relays.push(url);
                    }
                    _ => {
                        debug!("Relay {} is not connected.", url);
                        active_relay_list.remove_relay(&url);
                    }
                }
            }
            
            // Send update if there are changes (simplified check for now)
            if !current_active_relays.is_empty() {
                let _ = update_sender.send(current_active_relays).await;
            }
        }
    }

    async fn connect(&mut self) -> Result<()> {
        let relays = self.relay_client.relays().await;
        debug!("Connecting to {} relays ...", relays.len());
        for u in relays.keys() {
            trace!("{:?} ", u.to_string())
        }
        debug!("\n");
        // Warning: error is not handled here, should check back status
        self.relay_client.connect().await;
        debug!("Connected");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.relay_client.disconnect().await?;
        debug!("Disconnected");
        Ok(())
    }

    async fn subscribe(&mut self, time_start: Timestamp, time_end: Timestamp) -> Result<()> {
        self.relay_client
            .subscribe(
                vec![Filter::new()
                    // .pubkey(keys.public_key())
                    // .kind(Kind::RecommendRelay)
                    .kinds(vec![Kind::ContactList, Kind::RecommendRelay])
                    .since(time_start)
                    .until(time_end)],
                None,
            )
            .await?;
        debug!("Subscribed to relay events",);
        self.relay_client
            .publish_text_note(format!("{}", time_start), [])
            .await?;
        self.relay_client
            .publish_text_note(format!("{}", time_end), [])
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
                .publish_text_note(format!("{}", connected_relays), [])
                .await?;
            self.relay_client
                .publish_text_note(format!("{}", available_relays), [])
                .await?;
        }
        Ok(())
    }

    async fn wait_and_handle_messages(&mut self) -> Result<()> {
        // Keep track of relays with EOSE sent
        let mut eose_relays = HashSet::<Url>::new();

        let now = Timestamp::now();
        let period_end = now;
        let period_start = period_end - Duration::from_secs(PERIOD_START_PAST_SECS);
        self.subscribe(period_start, period_end).await?;

        let mut notifications = self.relay_client.notifications();
        while let Ok(notification) = notifications.recv().await {
            debug!("relaynotif {:?}", notification);
            match notification {
                RelayPoolNotification::Event {
                    relay_url: _,
                    subscription_id: _,
                    event,
                } => {
                    self.handle_event(&event);
                    // invoke callback
                    self.processor.handle_event(&event);
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
                            match relay.status().await {
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
                        debug!("{{\"{:?}\":\"{url}\"}}", relaymsg);
                    }
                },
                RelayPoolNotification::Shutdown => break,
                RelayPoolNotification::RelayStatus { .. } => (),
                RelayPoolNotification::Authenticated { .. } => (),
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
        match event.kind {
            Kind::Metadata => {
                debug!("{:?}", event.kind);
            }
            Kind::TextNote => {
                debug!("{:?}", event.kind);
            }
            Kind::EncryptedDirectMessage => {
                info!("{:?}", event.kind);
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
                debug!("{:?}", event.kind);
            }
            Kind::Replaceable(_u16) => {
                debug!("{:?}", event.kind);
            }
            Kind::Ephemeral(_u16) => {
                debug!("{:?}", event.kind);
            }
            Kind::ParameterizedReplaceable(_u16) => {
                debug!("{:?}", event.kind);
            }
            Kind::Custom(_u64) => {
                debug!("{:?}", event.kind);
            }
            Kind::ContactList => {
                self.update_event_time();
                // count p tags
                let mut count = 0;
                for _t in &event.tags {
                    if let Some(TagStandard::PublicKey {
                        public_key: _,
                        relay_url: Some(ss),
                        ..
                    }) = _t.as_standardized()
                    {
                        //state.pubkeys.add(pk);
                        //if let Some(ss) = s {
                        debug!("    {ss}");
                        let ss_str: &str = &ss.to_string();
                        let _ = self.relays.add(ss_str);
                        let _pub_future = self.relay_client.publish_text_note(ss.to_string(), []);
                        //}
                        debug!("    {}", count);
                        count += 1;
                    }
                }
            }
            Kind::RecommendRelay => {
                self.update_event_time();
                debug!("\n393:Relay(s): {}\n", event.content);
                let _ = self.relays.add(&event.content);
            }
            _ => {}
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
