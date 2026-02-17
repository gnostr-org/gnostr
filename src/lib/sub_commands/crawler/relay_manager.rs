use super::processor::Processor;
use super::{load_file};
use super::relays::{Relays, fetch_online_relays};

use super::APP_SECRET_KEY;
use nostr_sdk_0_19_1::prelude::FromSkStr;
use nostr_sdk_0_19_1::{
    prelude::{
        Client, Event, Filter, Keys, Kind, Options, RelayPoolNotification, Result, Tag, Timestamp,
        Url,
    },
    RelayMessage, RelayStatus,
};
use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;




use std::str;

use log::debug;
use log::info;
use log::trace;
use log::warn;

const MAX_ACTIVE_RELAYS: usize = 3; //usize::MAX;
const PERIOD_START_PAST_SECS: u64 = 6 * 60 * 60;

/// Keeps a set of active connections to relays
pub struct RelayManager {
    app_keys: Keys,
    pub relays: Relays,
    relay_client: Client,
    pub processor: Processor,
    /// Time of last event seen (real time, Unix timestamp)
    time_last_event: u64,
}

impl RelayManager {
    pub async fn new(app_keys: Keys, processor: Processor) -> Self {
        let opts = Options::new(); //.wait_for_send(false);
        let relay_client = Client::new_with_opts(&app_keys, opts);
        let _proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));

        let mut relays_instance = Relays::new();

        // Load relays from relays.yaml
        match load_file("relays.yaml") {
            Ok(urls) => {
                for url_str in urls {
                    debug!("relay_manager::new::url_str={}", &url_str);
                    relays_instance.add(&url_str);
                }
                debug!("Loaded {} relays from relays.yaml", relays_instance.count());
            },
            Err(e) => debug!("Could not load relays.yaml: {}", e),
        }

        // Fetch online relays (permissionlesstech/bitchat)
        let bitchat_online_relays_url = "https://raw.githubusercontent.com/permissionlesstech/bitchat/refs/heads/main/relays/online_relays_gps.csv";
        match fetch_online_relays(bitchat_online_relays_url).await {
            Ok(urls) => {
                for url_str in urls {
                    relays_instance.add(&url_str);
                }
                debug!("Loaded {} relays from online CSV (bitchat)", relays_instance.count());
            },
            Err(e) => debug!("Could not fetch online relays from bitchat: {}", e),
        }

        // Fetch online relays (sesseor/nostr-relays-list)
        let sesseor_online_relays_url = "https://raw.githubusercontent.com/sesseor/nostr-relays-list/main/relays.txt";
        match fetch_online_relays(sesseor_online_relays_url).await {
            Ok(urls) => {
                for url_str in urls {
                    relays_instance.add(&url_str);
                }
                debug!("Loaded {} relays from online TXT (sesseor)", relays_instance.count());
            },
            Err(e) => debug!("Could not fetch online relays from sesseor: {}", e),
        }

        Self {
            app_keys,
            relays: relays_instance,
            relay_client,
            processor,
            time_last_event: Self::now(),
        }
    }

    fn add_bootstrap_relays_if_needed(&mut self, bootstrap_relays: Vec<&str>) {

        debug!("relay_manager::add_bootstrap_relays_if_needed");
        for us in &bootstrap_relays {
            if self.relays.count() >= MAX_ACTIVE_RELAYS {
                //return;
            }
            self.relays.add(us);
        }
    }

    async fn add_some_relays(&mut self) -> Result<()> {
        debug!("relay_manager::add_some_relays");
        // remove all
        loop {
            let relays = self.relay_client.relays().await;
            let relay_urls: Vec<&Url> = relays.keys().collect();
            if relay_urls.is_empty() {
                break;
            }
            for relay_url in &relay_urls {
                debug!("removing relay_url:{}", relay_url.to_string());
            }
            //self.relay_client
            //    .remove_relay(relay_urls[0].to_string())
            //    .await?;
        }
        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);



        //async {
        let opts = Options::new(); //.wait_for_send(true);
        let app_keys = Keys::from_sk_str(APP_SECRET_KEY).unwrap();
        let _relay_client = Client::new_with_opts(&app_keys, opts);
        //};

        for r in some_relays {
            debug!("r={}", &r);
            //self.relay_client.add_relay(r, None).await?;
            self.relay_client.add_relay(r.clone(), None).await?;
            //self.relay_client
            //    .publish_text_note("relay_manager:5<--------<<<<<<<<<", &[])
            //    .await?;
            //self.relay_client
            //    .publish_text_note("6<--------<<<<<<<<<", &[])
            //    .await?;
            //self.relay_client
            //    .publish_text_note("7<--------<<<<<<<<<", &[])
            //    .await?;
            //self.relay_client
            //    .publish_text_note("888888<--------<<<<<<<<<", &[])
            //    .await?;
            self.relay_client
                .publish_text_note(format!("{}", r), &[])
                .await?;
        }
        Ok(())
    }

    pub async fn run(&mut self, bootstrap_relays: Vec<&str>) -> Result<()> {
        debug!("relay_manager::run");
        self.add_bootstrap_relays_if_needed(bootstrap_relays);
        self.add_some_relays().await?;
        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);
        for url in &some_relays {
            //if url NOT contain
            if !url.as_str().contains("")
                || !url.as_str().contains("")
                || !url.as_str().contains("")
            {
                self.relay_client.add_relay(url.to_string(), None).await?;
            }
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

    async fn connect(&mut self) -> Result<()> {
        debug!("relay_manager::connect");
        let relays = self.relay_client.relays().await;
        info!("Connecting to {} relays ...", relays.len());
        for u in relays.keys() {
            debug!("u={:?} ", u.to_string())
        }
        debug!("
");

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
                match relay.status().await {
                    RelayStatus::Connected => {
                        info!("Relay {} is connected.", relay.url().to_string());
                    },
                    RelayStatus::Disconnected | RelayStatus::Terminated | RelayStatus::Connecting => {
                        warn!("Relay {} is not yet connected. Status: {:?}", relay.url().to_string(), relay.status().await);
                        all_connected = false;
                    }
                    _ => {
                        warn!("Relay {} has unknown status: {:?}", relay.url().to_string(), relay.status().await);
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
            warn!("Failed to connect to all relays after {} attempts.", max_attempts);
            //TODO append to shitlist.yaml in user space
            Ok(())
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.relay_client.disconnect().await?;
        debug!("Disconnected");
        Ok(())
    }

    async fn subscribe(&mut self, time_start: Timestamp, time_end: Timestamp) -> Result<()> {
        self.relay_client
            .subscribe(vec![Filter::new()
                .pubkey(self.app_keys.public_key())
                .kinds(vec![Kind::ContactList, Kind::RecommendRelay])
                .since(time_start)
                .until(time_end)])
            .await;
        debug!("Subscribed to relay events",);
        self.relay_client
            .publish_text_note(format!("{}", time_start), &[])
            .await?;
        self.relay_client
            .publish_text_note(format!("{}", time_end), &[])
            .await?;
        Ok(())
    }

    async fn unsubscribe(&mut self) -> Result<()> {
        self.relay_client.unsubscribe().await;
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
                .publish_text_note(format!("{}", connected_relays), &[])
                .await?;
            self.relay_client
                .publish_text_note(format!("{}", available_relays), &[])
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
            trace!("relay_manager::wait_and_handle_messages::relaynotif {:?}", notification);
            match notification {
                RelayPoolNotification::Event(_url, event) => {
                    self.handle_event(&event); //self.handle_event
                    // invoke callback
                    self.processor.handle_event(&event); //self.processor.handle_event
                }
                RelayPoolNotification::Message(url, relaymsg) => match relaymsg {
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
                    } => {},
                    RelayMessage::Empty => {
                        trace!("Received empty message from {url}");
                    }
                    _ => {
                        debug!("Received unhandled relay message from {url}: {{\"{:?}\":\"{}\"}} ", relaymsg, url);
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
            Kind::Zap => {
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
                    },
                    Err(e) => {
                        warn!("Failed to parse Kind::RelayList event content as JSON: {}. Content: {}", e, event.content);
                    }
                }
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
                    if let Tag::PubKey(_pk, Some(ss)) = _t {
                        //state.pubkeys.add(pk); //TODO neccesary?
                        //if let Some(ss) = s {
                        trace!("    {ss}");
                        let _ = self.relays.add(ss);
                        let _pub_future = self.relay_client.publish_text_note(ss.to_string(), &[]);
                        //}
                        trace!("    {}", count);
                        count += 1;
                    }
                }
            }
            Kind::RecommendRelay => {
                self.update_event_time();
                debug!("
490:Relay(s): {}
", event.content);
                let _ = self.relays.add(&event.content);
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
