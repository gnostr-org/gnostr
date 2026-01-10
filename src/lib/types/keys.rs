// Dummy Keys struct for now, to replace nostr_sdk::Keys
// TODO: Implement actual Keys functionality

use std::fmt;

use crate::types::{Error, PrivateKey, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    private_key: Option<PrivateKey>,
    public_key: PublicKey,
}

impl fmt::Display for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Keys {{ public_key: {} }}",
            self.public_key.as_hex_string()
        )
    }
}

impl Keys {
    pub fn generate() -> Self {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        Keys {
            private_key: Some(private_key),
            public_key,
        }
    }

    pub fn new(private_key: PrivateKey) -> Self {
        let public_key = private_key.public_key();
        Keys {
            private_key: Some(private_key),
            public_key,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn secret_key(&self) -> Result<PrivateKey, Error> {
        self.private_key.clone().ok_or(Error::NoPrivateKey)
    }

    // TODO: Implement actual vanity key generation
    pub fn vanity(_prefixes: Vec<String>, _bech32: bool, _num_cores: usize) -> Result<Self, Error> {
        println!("Dummy: Vanity key generation not yet implemented, using random key");
        Ok(Self::generate())
    }
}
