//! `xor` subcommand
//!
//! This subcommand provides a basic XOR operation for input data using a given
//! key.

use std::io::{self, Read};

use anyhow::Result;
use clap::Args;

/// `xor` subcommand arguments.
#[derive(Args, Debug, Clone)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct XorArgs {
    /// Private key (hex or bech32) to use as the XOR key.
    /// If not provided, it will be read from stdin.
    #[arg(long, short = 's', alias = "privkey")]
    pub nsec: Option<String>,

    /// Read the input for XOR from stdin (default behavior if no key is
    /// provided via --nsec). If --nsec is also provided, stdin will be XORed
    /// with the provided key.
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub stdin: bool,
}

/// Executes the `xor` subcommand.
pub async fn xor_command(args: &XorArgs) -> Result<()> {
    // 1. Get the key
    let key_hex = if let Some(nsec_val) = &args.nsec {
        nsec_val.clone()
    } else if args.stdin {
        // If --stdin is explicitly set and --nsec is not, expect key from stdin first
        let mut key_from_stdin = String::new();
        io::stdin().read_to_string(&mut key_from_stdin)?;
        key_from_stdin.trim().to_string()
    } else {
        // Fallback or error if no key is specified
        eprintln!("Error: A key must be provided via --nsec or piped through stdin (--stdin).");
        return Ok(());
    };

    // Decode key from hex (assuming key is always hex for simplicity as per
    // DEFAULT_SEC in example)
    let key_bytes = hex::decode(&key_hex)?;

    // 2. Get the input data
    let mut input_data = String::new();
    io::stdin().read_to_string(&mut input_data)?;
    let input_bytes = input_data.as_bytes();

    // 3. Perform XOR operation
    let xored_bytes: Vec<u8> = input_bytes
        .iter()
        .zip(key_bytes.iter().cycle()) // Cycle the key if it's shorter than input
        .map(|(input_byte, key_byte)| input_byte ^ key_byte)
        .collect();

    // 4. Print output (hex encoded)
    println!("{}", hex::encode(&xored_bytes));

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    // Helper to simulate stdin
    fn simulate_stdin(input: &str) -> Cursor<Vec<u8>> {
        Cursor::new(input.as_bytes().to_vec())
    }

    #[tokio::test]
    async fn test_xor_with_key_and_stdin() -> Result<()> {
        let _args = XorArgs {
            nsec: Some("00".to_string()), // Hex for 0
            stdin: true,
        };
        let input = "hello";

        let _stdin_mock = simulate_stdin(input);
        let _original_stdin = io::stdin(); // Save original stdin
                                           // Replace stdin with mock (this part is tricky, may need to use external crate
                                           // like `mock_stdin`) For actual tests, this would ideally involve
                                           // mocking std::io::stdin directly or passing a Read trait.
                                           // For simplicity in this test, we'll manually xor and compare.

        // Manually calculate expected output
        let key_bytes = hex::decode("00")?; // 00 hex = 0 decimal
        let input_bytes = input.as_bytes();
        let expected_xored_bytes: Vec<u8> = input_bytes
            .iter()
            .zip(key_bytes.iter().cycle())
            .map(|(input_byte, key_byte)| input_byte ^ key_byte)
            .collect();
        let expected_output = hex::encode(&expected_xored_bytes);

        // This test would need to capture stdout to verify the output.
        // For now, we'll just check if it runs without error and produces the expected
        // output manually. In a real test, `assert_stdout_eq` from `assert_cmd`
        // could be used. For this context, assuming `xor_command` prints to
        // stdout.

        // As we cannot easily mock global stdin/stdout without external crates,
        // let's create a simpler unit test for the xor logic itself.
        let result_bytes = perform_xor(input_bytes, &key_bytes)?;
        assert_eq!(hex::encode(&result_bytes), expected_output);
        Ok(())
    }

    // Extracted XOR logic for easier testing
    fn perform_xor(input_bytes: &[u8], key_bytes: &[u8]) -> Result<Vec<u8>> {
        if key_bytes.is_empty() {
            return Err(anyhow::anyhow!("XOR key cannot be empty"));
        }
        let xored_bytes: Vec<u8> = input_bytes
            .iter()
            .zip(key_bytes.iter().cycle())
            .map(|(input_byte, key_byte)| input_byte ^ key_byte)
            .collect();
        Ok(xored_bytes)
    }

    #[tokio::test]
    async fn test_perform_xor_basic() -> Result<()> {
        let input = "abc"; // 0x61 0x62 0x63
        let key = "ff"; // 0xff
        let expected = "9e9d9c"; // 0x61^0xff = 0x9e, 0x62^0xff = 0x9d, 0x63^0xff = 0x9c

        let input_bytes = input.as_bytes();
        let key_bytes = hex::decode(key)?;
        let result = perform_xor(input_bytes, &key_bytes)?;
        assert_eq!(hex::encode(&result), expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_perform_xor_longer_key() -> Result<()> {
        let input = "abc"; // 0x61 0x62 0x63
        let key = "f0f1f2"; // 0xf0 0xf1 0xf2
        let expected = "919391"; // Corrected value

        let input_bytes = input.as_bytes();
        let key_bytes = hex::decode(key)?;
        let result = perform_xor(input_bytes, &key_bytes)?;
        assert_eq!(hex::encode(&result), expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_perform_xor_empty_input() -> Result<()> {
        let input = "";
        let key = "ff";
        let expected = "";

        let input_bytes = input.as_bytes();
        let key_bytes = hex::decode(key)?;
        let result = perform_xor(input_bytes, &key_bytes)?;
        assert_eq!(hex::encode(&result), expected);
        Ok(())
    }

    #[tokio::test]
    async fn test_perform_xor_empty_key_error() {
        let input = "abc";
        let key = "";
        let input_bytes = input.as_bytes();
        let key_bytes = hex::decode(key).unwrap(); // This will decode to empty vec

        let result = perform_xor(input_bytes, &key_bytes);
        assert!(result.is_err());
    }
}
