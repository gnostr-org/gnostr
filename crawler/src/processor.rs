use crate::pubkeys::PubKeys;
use crate::stats::Stats;
use log::{debug, info};
use nostr_sdk::prelude::{Event, Kind, Tag, Timestamp};

pub const BOOTSTRAP_RELAY1: &str = "wss://nos.lol";
pub const BOOTSTRAP_RELAY2: &str = "wss://relay.damus.io";
pub const BOOTSTRAP_RELAY3: &str = "wss://e.nos.lol";
//pub const APP_SECRET_KEY: &str = "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85";
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
        //println!("{:?}", event.id);
        //println!("{:}", event.as_json());
        //println!("age {:?}  created_at {:?}", Self::age(event.created_at), event.created_at);
        match event.kind {
            //Kind::Metadata => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::TextNote => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::EncryptedDirectMessage => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::EventDeletion => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Repost => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Reaction => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ChannelCreation => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ChannelMetadata => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ChannelMessage => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ChannelHideMessage => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ChannelMuteUser => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::PublicChatReserved45 => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::PublicChatReserved46 => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::PublicChatReserved47 => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::PublicChatReserved48 => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::PublicChatReserved49 => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Reporting => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ZapRequest => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Zap => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Authentication => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::NostrConnect => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::RelayList => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Replaceable(u16) => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Ephemeral(u16) => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::ParameterizedReplaceable(u16) => {
            //    println!("{:?}", event.kind);
            //}
            //Kind::Custom(u64) => {
            //    println!("{:?}", event.kind);
            //}
            Kind::ContactList => {
                self.stats.add_contacts();
                // count p tags
                //let mut cnt = 0;
                for t in &event.tags {
                    if let Tag::PubKey(pk, _s) = t {
                        self.pubkeys.add(pk);
                        //cnt += 1;
                    }
                }
                //println!("Contacts {} \t ", cnt); // event.pubkey.to_bech32().unwrap(),
                // self.print_summary();

                //println!("{:?}", event);
            }
            Kind::RecommendRelay => {
                self.stats.add_relays();
                //println!("{:?}", event);
            }
            _ => {
                //println!("{:?}", event.kind)
                println!("processing...")
            }
        }
    }

    // fn print_summary(&self) {
    //     print!("pks {} \t ", self.pubkeys.count());
    //     self.stats.print_summary();
    // }

    pub fn dump(&self) {
        //println!();
        //println!(
        //    "Number of ContactList events:      \t {}",
        //    self.stats.count_contacts
        //);
        //println!(
        //    "Number of RecommendedRelay events: \t {}",
        //    self.stats.count_relays
        //);
        //println!();
        //self.pubkeys.dump();
    }
}
