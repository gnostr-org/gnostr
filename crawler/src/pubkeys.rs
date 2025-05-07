use nostr_sdk::prelude::{FromBech32, ToBech32, XOnlyPublicKey};
use std::collections::HashMap;

pub struct PubKeys {
    p: HashMap<XOnlyPublicKey, u64>,
}

impl Default for PubKeys {
    fn default() -> Self {
        Self::new()
    }
}

impl PubKeys {
    pub fn new() -> Self {
        Self {
            p: HashMap::default(),
        }
    }

    pub fn add(&mut self, p: &XOnlyPublicKey) -> u64 {
        if !self.p.contains_key(p) {
            let n = self.p.len() as u64;
            self.p.insert(*p, n);
            // print!("pks {n}  ");
        } else {
            // print!(".");
        }
        self.p[p]
    }

    #[allow(dead_code)]
    pub fn add_str(&mut self, p: &str) -> u64 {
        let pp = match XOnlyPublicKey::from_bech32(p) {
            Err(_) => return 0,
            Ok(pp) => pp,
        };
        self.add(&pp)
    }

    // pub fn count(&self) -> usize {
    //     self.p.len()
    // }

    pub fn dump(&self) {
        println!("Found  {}  public keys:", self.p.len());
        for pk in self.p.keys() {
            println!("  {}", pk.to_bech32().unwrap());
        }
        println!();
    }
}
