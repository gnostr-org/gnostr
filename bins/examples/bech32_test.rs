use gnostr::types::PublicKey;

fn main() {
    let bech32_str = "npub10elfcs4fr0l0r8af98jlmgdh9c8efajjp7d99q03f6tbpkct3uhqsfq0z6";
    println!("Decoding: {}", bech32_str);
    match PublicKey::try_from_bech32_string(bech32_str, false) {
        Ok(pubkey) => {
            println!("Successfully decoded pubkey: {}", pubkey);
        }
        Err(e) => {
            println!("Failed to decode pubkey: {}", e);
        }
    }
}
