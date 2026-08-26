use std::{
    fs::{read_to_string, File},
    path::PathBuf,
};

use anyhow::Context;
use russh_keys::{key::KeyPair, *};

const SERVER_KEY_LOCATION: &str = "server_key";

/// Loads the server's keys if it exists.
fn load_server_keys() -> anyhow::Result<Option<KeyPair>> {
    if !PathBuf::from(SERVER_KEY_LOCATION).exists() {
        return Ok(None);
    }

    let text =
        read_to_string(SERVER_KEY_LOCATION).context("Failed reading server key from file")?;
    let keys = decode_secret_key(&text, None).context("Error decoding server key")?;
    Ok(Some(keys))
}

/// Writes the server keys to the filesystem.
fn write_server_keys(keys: &KeyPair) -> anyhow::Result<()> {
    let file = File::create(SERVER_KEY_LOCATION).context("Could not create server key file")?;
    encode_pkcs8_pem(keys, file).context("Error writing server key to file")?;
    Ok(())
}

/// Get's the server keys, creaing new ones if needed.
pub fn server_keys() -> anyhow::Result<KeyPair> {
    if let Some(keys) = load_server_keys()? {
        Ok(keys)
    } else {
        let keys = russh_keys::key::KeyPair::generate_ed25519().unwrap();
        write_server_keys(&keys)?;
        Ok(keys)
    }
}
