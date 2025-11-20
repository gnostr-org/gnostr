// TEMPORARILY
#![allow(clippy::uninlined_format_args)]

use gnostr_types::{EncryptedPrivateKey, PrivateKey};

fn run_decrypt_private_key(encrypted_private_key: &str, password: &str) -> Result<String, String> {
    let epk = EncryptedPrivateKey(encrypted_private_key.to_owned());

    let mut private_key = PrivateKey::import_encrypted(&epk, password)
        .map_err(|e| format!("Could not import encrypted private key: {}", e))?;
    
    Ok(private_key.as_hex_string())
}

fn main() {
    println!("DANGER this exposes the private key.");
    println!("encrypted private key: ");
    let mut epk_input = String::new();
    let stdin = std::io::stdin();
    stdin.read_line(&mut epk_input).unwrap();
    let epk = epk_input.trim();

    let password = rpassword::prompt_password("Password: ").unwrap();
    
    match run_decrypt_private_key(epk, &password) {
        Ok(private_key_hex) => println!("Private key: {}", private_key_hex),
        Err(e) => eprintln!("Error: {}", e),
    }
}
