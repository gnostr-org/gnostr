use nostr_0_34_1::{PublicKey, nips::nip19::FromBech32};

fn main() {
    let bech32_str = "npub10elfcs4fr0l0r8af98jlmgdh9c8efajjp7d99q03f6tbpkct3uhqsfq0z6";
    println!("Decoding: {}", bech32_str);
    match PublicKey::from_bech32(bech32_str) {
        Ok(pubkey) => {
            println!("Successfully decoded pubkey: {}", pubkey);
        }
        Err(e) => {
            println!("Failed to decode pubkey: {}", e);
        }
    }
}
