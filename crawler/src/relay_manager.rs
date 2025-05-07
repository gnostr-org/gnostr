use crate::processor::Processor;
use crate::relays::Relays;
use crate::CliArgs;
use crate::APP_SECRET_KEY;
use nostr_sdk::prelude::FromSkStr;
use nostr_sdk::{
    prelude::{
        Client, Event, Filter, Keys, Kind, Options, RelayPoolNotification, Result, Tag, Timestamp,
        Url,
    },
    RelayMessage, RelayStatus,
};
use std::collections::HashSet;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::time::Duration;

use clap::Parser;

use git2::Repository;
use std::str;

use log::debug;
use log::info;
use log::trace;

const MAX_ACTIVE_RELAYS: usize = 2; //usize::MAX;
const PERIOD_START_PAST_SECS: u64 = 6 * 60 * 60;

/// Keeps a set of active connections to relays
pub struct RelayManager {
    // app_keys: Keys,
    relays: Relays,
    relay_client: Client,
    pub processor: Processor,
    /// Time of last event seen (real time, Unix timestamp)
    time_last_event: u64,
}

impl RelayManager {
    pub fn new(app_keys: Keys, processor: Processor) -> Self {
        let opts = Options::new(); //.wait_for_send(false);
        let relay_client = Client::new_with_opts(&app_keys, opts);
        let _proxy = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050)));
        Self {
            // app_keys,
            relays: Relays::new(),
            relay_client,
            processor,
            time_last_event: Self::now(),
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
        let repo = Repository::open(path)?;
        let revwalk = repo.revwalk()?;
        for commit in revwalk {
            println!("\n\n\n\n\n{:?}\n\n\n\n", commit);
        }

        //async {
        let opts = Options::new(); //.wait_for_send(true);
        let app_keys = Keys::from_sk_str(APP_SECRET_KEY).unwrap();
        let relay_client = Client::new_with_opts(&app_keys, opts);
        //let _ = relay_client.publish_text_note(path, &[]).await;
        //let _ = relay_client
        //    .publish_text_note("relay_manager:1<--------------------------<<<<<", &[])
        //    .await;
        //let _ = relay_client
        //    .publish_text_note("2<--------------------------<<<<<", &[])
        //    .await;
        //let _ = relay_client
        //    .publish_text_note("3<--------------------------<<<<<", &[])
        //    .await;
        //let _ = relay_client
        //    .publish_text_note("4<--------------------------<<<<<", &[])
        //    .await;
        let _ = relay_client.publish_text_note("#gnostr", &[]).await;
        //};

        for r in some_relays {
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
        self.add_bootstrap_relays_if_needed(bootstrap_relays);
        self.add_some_relays().await?;
        let some_relays = self.relays.get_some(MAX_ACTIVE_RELAYS);
        for url in &some_relays {
            self.relay_client.add_relay(url.to_string(), None).await?;
        }
        self.connect().await?;

        self.wait_and_handle_messages().await?;

        debug!("STOPPED");
        debug!("======================================================");
        debug!("\n");
        self.relays.dump_list();

        Ok(())
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
            .subscribe(vec![Filter::new()
                // .pubkey(keys.public_key())
                // .kind(Kind::RecommendRelay)
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
            debug!("relaynotif {:?}", notification);
            match notification {
                RelayPoolNotification::Event(_url, event) => {
                    self.handle_event(&event);
                    // invoke callback
                    self.processor.handle_event(&event);
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
                    } => {}
                    _ => {
                        debug!("{{\"{:?}\":\"{url}\"}}", relaymsg);
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
                let mut cnt = 0;
                for t in &event.tags {
                    //if let Tag::PubKey(_pk, Some(ss)) = t {
                    //  state.pubkeys.add(pk);
                    //if let Some(ss) = s {
                    //debug!("    {ss}");
                    //let _ = self.relays.add(ss);
                    //}
                    cnt += 1;
                    //}
                }
            }
            Kind::RelayList => {
                debug!("{:?}", event.kind);
            }
            Kind::Replaceable(u16) => {
                debug!("{:?}", event.kind);
            }
            Kind::Ephemeral(u16) => {
                debug!("{:?}", event.kind);
            }
            Kind::ParameterizedReplaceable(u16) => {
                debug!("{:?}", event.kind);
            }
            Kind::Custom(u64) => {
                debug!("{:?}", event.kind);
            }
            Kind::ContactList => {
                self.update_event_time();
                // count p tags
                let mut count = 0;
                for t in &event.tags {
                    if let Tag::PubKey(pk, Some(ss)) = t {
                        //state.pubkeys.add(pk);
                        //if let Some(ss) = s {
                        debug!("    {ss}");
                        let _ = self.relays.add(ss);
                        let _pub_future = self.relay_client.publish_text_note(ss.to_string(), &[]);
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
            _ => {
                debug!("Unsupported event {:?}", event.kind)
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
