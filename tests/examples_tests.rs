#[cfg(test)]
mod tests {

    use gnostr_types::{EncryptedPrivateKey, PrivateKey};

    // Import the refactored function
    // Since it's in examples, and examples are typically standalone binaries,
    // we might need to adjust the way we import it.
    // For now, let's assume we can directly include the file or its logic.
    // If not, we would need to move the `run_decrypt_private_key` into a library module.
    
    // For the purpose of this test, I'll copy the function locally.
    // In a real project, it would be better to expose it from `examples/decrypt_private_key.rs`
    // or move it to a shared library.

    // Helper function copied from examples/decrypt_private_key.rs
    fn run_decrypt_private_key(encrypted_private_key: &str, password: &str) -> Result<String, String> {
        let epk = EncryptedPrivateKey(encrypted_private_key.to_owned());

        let mut private_key = PrivateKey::import_encrypted(&epk, password)
            .map_err(|e| format!("Could not import encrypted private key: {}", e))?;
        
        Ok(private_key.as_hex_string())
    }

    #[test]
    fn test_decrypt_private_key_success() {
        let mut private_key = PrivateKey::generate();
        let password = "test_password";
        let encrypted_private_key = private_key.export_encrypted(password, 13).unwrap();

        let result = run_decrypt_private_key(encrypted_private_key.0.as_str(), password);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), private_key.as_hex_string());
    }

    #[test]
    fn test_decrypt_private_key_wrong_password() {
        let mut private_key = PrivateKey::generate();
        let correct_password = "test_password";
        let wrong_password = "wrong_password";
        let encrypted_private_key = private_key.export_encrypted(correct_password, 13).unwrap();

        let result = run_decrypt_private_key(encrypted_private_key.0.as_str(), wrong_password);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Could not import encrypted private key"));
    }
}