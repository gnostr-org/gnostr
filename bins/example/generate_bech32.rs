use gnostr::types::PrivateKey;

fn main() {
    let private_key = PrivateKey::generate();
    let public_key = private_key.public_key();
    println!("{}", public_key.as_bech32_string());
}
