use super::pubkeys::PubKeys;
use super::stats::Stats;
use log::debug;

use nostr_sdk_0_19_1::prelude::{Event, Kind, Tag, Timestamp};
use std::sync::LazyLock;

pub const LOCALHOST_8080: &str = "ws://127.0.0.1:8080";

pub static BOOTSTRAP_RELAYS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let relays_yaml_bytes = include_bytes!("../../../../config/relays.yaml");
    let relays_yaml_content = String::from_utf8_lossy(relays_yaml_bytes);
    relays_yaml_content.lines()
        .filter(|line: &&str| !line.trim().is_empty())
        .map(|line: &str| String::from(line))
        .collect()
});
pub static SHITLIST_RELAYS: LazyLock<Vec<String>> = LazyLock::new(|| {
    let relays_yaml_bytes = include_bytes!("../../../../config/shitlist.yaml");
    let relays_yaml_content = String::from_utf8_lossy(relays_yaml_bytes);
    relays_yaml_content.lines()
        .filter(|line: &&str| !line.trim().is_empty())
        .map(|line: &str| String::from(line))
        .collect()
});

// try gnostr bech32-to-any nsec1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sv7rt36
pub const APP_SECRET_KEY: &str = "nsec1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sv7rt36";
pub struct Processor {
    pubkeys: PubKeys,
    stats: Stats,
}

impl Default for Processor {
    fn default() -> Self {
        Self::new()
    }
}

impl Processor {
    pub fn new() -> Self {
        Self {
            pubkeys: PubKeys::new(),
            stats: Stats::new(),
        }
    }

    #[allow(dead_code)]
    fn age(t: Timestamp) -> i64 {
        Timestamp::now().as_i64() - t.as_i64()
    }

    pub fn handle_event(&mut self, event: &Event) {
        //TODO: forward (proxy)
        debug!("{:?}", event.id);
        //println!("{:}", event.as_json());
        debug!("age {:?}  created_at {:?}", Self::age(event.created_at), event.created_at);
        match event.kind {
            Kind::Metadata => {
                debug!("Kind::Metadata={:?}", event.kind);
            }
            Kind::TextNote => {
                debug!("Kind::TextNote={:?}", event.kind);
            }
            Kind::EncryptedDirectMessage => {
                debug!("Kind::EncryptedDirectMessage={:?}", event.kind);
            }
            Kind::EventDeletion => {
                println!("{:?}", event.kind);
            }
            Kind::Repost => {
                println!("{:?}", event.kind);
            }
            Kind::Reaction => {
                println!("{:?}", event.kind);
            }
            Kind::ChannelCreation => {
                println!("{:?}", event.kind);
            }
            Kind::ChannelMetadata => {
                println!("{:?}", event.kind);
            }
            Kind::ChannelMessage => {
                println!("{:?}", event.kind);
            }
            Kind::ChannelHideMessage => {
                println!("{:?}", event.kind);
            }
            Kind::ChannelMuteUser => {
                println!("{:?}", event.kind);
            }
            Kind::PublicChatReserved45 => {
                println!("{:?}", event.kind);
            }
            Kind::PublicChatReserved46 => {
                println!("{:?}", event.kind);
            }
            Kind::PublicChatReserved47 => {
                println!("{:?}", event.kind);
            }
            Kind::PublicChatReserved48 => {
                println!("{:?}", event.kind);
            }
            Kind::PublicChatReserved49 => {
                println!("{:?}", event.kind);
            }
            Kind::Reporting => {
                println!("{:?}", event.kind);
            }
            Kind::ZapRequest => {
                println!("{:?}", event.kind);
            }
            Kind::Zap => {
                println!("{:?}", event.kind);
            }
            Kind::Authentication => {
                println!("{:?}", event.kind);
            }
            Kind::NostrConnect => {
                println!("{:?}", event.kind);
            }
            Kind::RelayList => {
                println!("{:?}", event.kind);
            }
            Kind::Replaceable(_u16) => {
                println!("{:?}", event.kind);
            }
            Kind::Ephemeral(_u16) => {
                println!("{:?}", event.kind);
            }
            Kind::ParameterizedReplaceable(_u16) => {
                println!("{:?}", event.kind);
            }
            Kind::Custom(_u64) => {
                println!("{:?}", event.kind);
            }
            Kind::ContactList => {
                self.stats.add_contacts();
                // count p tags
                let mut cnt = 0;
                for t in &event.tags {
                    if let Tag::PubKey(pk, _s) = t {
                        self.pubkeys.add(pk);
                        cnt += 1;
                    }
                }
                debug!("Contacts {} 	 ", cnt); // event.pubkey.to_bech32().unwrap(),
                // self.print_summary();

                //println!("{:?}", event);
            }
            Kind::RecommendRelay => {
                self.stats.add_relays();
                debug!("{:?}", event);
            }
            _ => {
                debug!("{:?}", event.kind);
                println!("processing...");
            }
        }
    }

    // fn print_summary(&self) {
    //     print!("pks {} 	 ", self.pubkeys.count());
    //     self.stats.print_summary();
    // }

    pub fn dump(&self) {
        //println!();
        //println!(
        //    "Number of ContactList events:      	 {}",
        //    self.stats.count_contacts
        //);
        //println!(
        //    "Number of RecommendedRelay events: 	 {}",
        //    self.stats.count_relays
        //);
        //println!();
        //self.pubkeys.dump();
    }
}
