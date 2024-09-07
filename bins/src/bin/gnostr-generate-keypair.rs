// Copyright 2022-2023 nostr-bins Developers
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according to
// those terms.

#![allow(clippy::uninlined_format_args)]
use gnostr_types::{PrivateKey, PublicKey};
use k256::schnorr::SigningKey;
use rand_core::OsRng;
use zeroize::Zeroize;

fn main() {
    let signing_key = SigningKey::random(&mut OsRng);
    let mut private_key =
        PrivateKey::try_from_hex_string(&format!("{:x}", signing_key.to_bytes())).unwrap();
    let mut public_key = PublicKey::try_from_hex_string(
        &format!("{:x}", signing_key.verifying_key().to_bytes()),
        true,
    )
    .unwrap();
    let verifying_key = signing_key.verifying_key();

    let mut private_bech32 = private_key.as_bech32_string();
    let mut public_bech32 = public_key.as_bech32_string();
    println!(
        "[\"KEYS\",{{\"nsec\":\"{}\",\"npub\":\"{}\"}},{{\"private\":\"{:x}\",\"public\":\"{:x}\"\
         }}]",
        private_bech32,
        public_bech32,
        signing_key.to_bytes(),
        verifying_key.to_bytes()
    );
    private_bech32.zeroize();
    public_bech32.zeroize();

    //println!("PUBLIC: {:x}", verifying_key.to_bytes());
    //println!("PRIVATE: {:x}", signing_key.to_bytes());
}
