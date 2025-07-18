use std::process;
use k256::schnorr::{SigningKey, VerifyingKey};

pub fn is_valid(verifying_key_string: String, signing_key_string: String) -> bool {
    let verifying_key_bytes: Vec<u8> = match hex::decode(verifying_key_string) {
        Ok(v) => v,
        Err(e) => {
            println!("FAILURE: public key is not valid hex: {:?}", e);
            process::exit(1);
        }
    };
    let verifying_key = match VerifyingKey::from_bytes(&verifying_key_bytes) {
        Ok(k) => k,
        Err(e) => {
            println!("FAILURE: public key is not valid: {:?}", e);
            process::exit(1);
        }
    };

    let signing_key_bytes: Vec<u8> = match hex::decode(signing_key_string) {
        Ok(v) => v,
        Err(e) => {
            println!("FAILURE: private key is not valid hex: {:?}", e);
            process::exit(1);
        }
    };
    let signing_key = match SigningKey::from_bytes(&signing_key_bytes) {
        Ok(k) => k,
        Err(e) => {
            println!("FAILURE: private key is not valid: {:?}", e);
            process::exit(1);
        }
    };

    let matching_key = signing_key.verifying_key();

    if verifying_key != *matching_key {
        println!("FAILURE: Keys are NOT a valid pair");
        false//process::exit(1);
    } else {
        true//println!("SUCCESS: Keys match.");
    }
}
