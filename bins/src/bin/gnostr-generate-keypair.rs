// Copyright 2022-2023 nostr-bins Developers
// Copyright 2023-2024 gnostr-bins Developers
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according to
// those terms.

#![allow(clippy::uninlined_format_args)]
use gnostr_types::{PrivateKey, PublicKey};
use k256::schnorr::SigningKey;
use zeroize::Zeroize;

fn main() {
    let _buffer_min = &[
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 1,
    ];
    #[cfg(debug_assertions)]
    let _buffer_max = &[
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    ];

    //let iter = _buffer_min.chunks_exact(1);
    //for num in iter {
    ////print!("{}", u8::from_le_bytes(num.try_into().unwrap()));
    //}
    //let iter = _buffer_max.chunks_exact(1);
    //for num in iter {
    ////print!("{}", u8::from_le_bytes(num.try_into().unwrap()));
    //}

    let mut signing_key_vec: Vec<SigningKey> = Vec::new();

    #[cfg(not(debug_assertions))]
    use rand_core::OsRng;
    #[cfg(not(debug_assertions))]
    signing_key_vec.push(SigningKey::random(&mut OsRng));
    #[cfg(not(debug_assertions))]
    signing_key_vec.push(SigningKey::random(&mut OsRng));
    #[cfg(not(debug_assertions))]
    signing_key_vec.push(SigningKey::random(&mut OsRng));
    #[cfg(not(debug_assertions))]
    signing_key_vec.push(SigningKey::random(&mut OsRng));

    #[cfg(debug_assertions)]
    signing_key_vec.push(SigningKey::from_bytes(_buffer_min).unwrap());
    #[cfg(debug_assertions)]
    signing_key_vec.push(SigningKey::from_bytes(_buffer_min).unwrap());
    #[cfg(debug_assertions)]
    signing_key_vec.push(SigningKey::from_bytes(_buffer_min).unwrap());
    #[cfg(debug_assertions)]
    signing_key_vec.push(SigningKey::from_bytes(_buffer_min).unwrap());
    let mut private_key =
        PrivateKey::try_from_hex_string(&format!("{:x}", signing_key_vec[0].to_bytes())).unwrap();
    let public_key = PublicKey::try_from_hex_string(
        &format!("{:x}", signing_key_vec[1].verifying_key().to_bytes()),
        true,
    )
    .unwrap();
    let verifying_key = signing_key_vec[2].verifying_key().clone();
    let mut private_bech32 = private_key.as_bech32_string();
    let mut public_bech32 = public_key.as_bech32_string();
    println!(
        "[\"KEYS\",{{\"nsec\":\"{}\",\"npub\":\"{}\"}},{{\"private\":\"{:x}\",\"public\":\"{:x}\"\
         }}]",
        private_bech32,
        public_bech32,
        signing_key_vec[3].to_bytes(),
        verifying_key.to_bytes()
    );
    private_bech32.zeroize();
    public_bech32.zeroize();
}
