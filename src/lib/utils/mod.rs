pub mod pwd;
pub mod retry;

use log::{debug, error};
use nostr_sdk_0_32_0::prelude::*;
use serde_json;
use serde_json::{Result as SerdeJsonResult, Value};
use std::env;
use std::fmt::Write;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use ureq::Agent;

use std::path::PathBuf;

use std::net::TcpListener as StdTcpListener;
//use actix_rt::net::TcpListener as ActixRtTcpListener;
use actix_web::rt::net::TcpListener as ActixWebTcpListener;
use async_std::net::TcpListener as AsyncStdTcpListener;

/// parse_json
pub fn parse_json(json_string: &str) -> SerdeJsonResult<Value> {
    serde_json::from_str(json_string)
}

/// split_value_by_newline
pub fn split_value_by_newline(json_value: &Value) -> Option<Vec<String>> {
    if let Value::String(s) = json_value {
        let lines: Vec<String> = s.lines().map(|line| line.to_string()).collect();
        Some(lines)
    } else {
        None // Return None if the Value is not a string
    }
}

/// value_to_string
pub fn value_to_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(value_to_string).collect();
            format!("[{}]", elements.join(", "))
        }
        Value::Object(obj) => {
            let pairs: Vec<String> = obj
                .iter()
                .map(|(k, v)| format!("\"{}\": {}", k, value_to_string(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
    }
}

/// split_json_string
pub fn split_json_string(value: &Value, separator: &str) -> Vec<String> {
    if let Value::String(s) = value {
        s.split(&separator).map(|s| s.to_string()).collect()
    } else {
        vec![String::from("")]
    }
}

/// parse_private_key
pub async fn parse_private_key(private_key: Option<String>, print_keys: bool) -> Result<Keys> {
    // Parse and validate private key
    let keys = match private_key {
        Some(pk) => {
            if pk.starts_with("nsec") {
                Keys::new(SecretKey::from_bech32(pk)?)
            } else {
                // We assume it's a hex formatted private key
                Keys::new(SecretKey::from_hex(pk)?)
            }
        }
        None => {
            // create a new identity with a new keypair
            println!("No private key provided, generating new identity");
            Keys::generate()
        }
    };

    if print_keys {
        println!("Private key:");
        println!("{}", keys.secret_key()?.to_bech32()?);
        println!("{}", keys.secret_key()?.display_secret());

        println!("Public key:");
        println!("{}", keys.public_key().to_bech32()?);
        println!("{}", keys.public_key());
    }

    Ok(keys)
}

// Creates the websocket client that is used for communicating with relays
pub async fn create_client(keys: &Keys, relays: Vec<String>, difficulty: u8) -> Result<Client> {
    let opts = Options::new()
        .send_timeout(Some(Duration::from_secs(15)))
        .wait_for_send(true)
        .difficulty(difficulty);
    let client = Client::with_opts(keys, opts);
    client.add_relays(relays).await?;
    client.connect().await;
    Ok(client)
}

pub async fn parse_key_or_id_to_hex_string(
    input: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let hex_key_or_id = if input.starts_with("npub") {
        PublicKey::from_bech32(input.clone()).unwrap().to_hex()
    } else if input.starts_with("nsec") {
        SecretKey::from_bech32(input)?.display_secret().to_string()
    } else if input.starts_with("note") {
        EventId::from_bech32(input)?.to_hex()
    } else if input.starts_with("nprofile") {
        Nip19Profile::from_bech32(input)
            .unwrap()
            .public_key
            .to_hex()
    } else {
        // If the key is not bech32 encoded, return it as is
        input.clone()
    };

    Ok(hex_key_or_id)
}

pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Prefix {
    Npub,
    Nsec,
    Note,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Keyformat {
    Hex,
    Bech32,
}

pub fn byte_array_to_hex_string(byte_array: &[u8; 32]) -> String {
    let mut hex_string = String::new();
    for byte in byte_array {
        write!(&mut hex_string, "{:02x}", byte).unwrap();
    }
    hex_string
}

/// Synchronous HTTP request using ureq.
/// Handles errors gracefully instead of panicking.
pub fn ureq_sync(url: String) -> Result<String, String> {
    // Build the ureq agent with more generous timeouts.
    // 5 seconds for read and write should be more robust for network operations.
    let agent: Agent = ureq::AgentBuilder::new()
        .timeout_read(Duration::from_secs(5)) // Increased timeout
        .timeout_write(Duration::from_secs(5)) // Increased timeout
        .build();

    // Attempt to make the GET request and handle potential errors.
    match agent.get(&url).call() {
        Ok(response) => {
            // If the call was successful, try to convert the response into a string.
            match response.into_string() {
                Ok(body) => {
                    debug!("ureq_sync:body:\n{}", body); // Debug log the body
                    Ok(body)
                }
                Err(e) => {
                    // Log an error if converting the response to string fails.
                    error!(
                        "Failed to convert ureq_sync response to string for URL {}: {}",
                        url, e
                    );
                    Err(format!("Failed to convert response to string: {}", e))
                }
            }
        }
        Err(e) => {
            // Log a detailed error if the ureq call fails.
            // This will show up in your logs if the log level is configured to show errors.
            error!("ureq_sync:agent.get(&url) failed for URL {}: {:?}", url, e);
            Err(format!("HTTP request failed: {}", e))
        }
    }
}

/// Asynchronous HTTP request using tokio and ureq.
/// Handles errors gracefully instead of panicking.
pub async fn ureq_async(url: String) -> Result<String, String> {
    let s = tokio::spawn(async move {
        // Build the ureq agent with more generous timeouts.
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5)) // Increased timeout
            .timeout_write(Duration::from_secs(5)) // Increased timeout
            .build();

        // Attempt to make the GET request and handle potential errors.
        match agent.get(&url).call() {
            Ok(response) => {
                // If the call was successful, try to convert the response into a string.
                match response.into_string() {
                    Ok(body) => {
                        debug!("ureq_async:body:\n{}", body); // Debug log the body
                        Ok(body)
                    }
                    Err(e) => {
                        // Log an error if converting the response to string fails.
                        error!(
                            "Failed to convert ureq_async response to string for URL {}: {}",
                            url, e
                        );
                        Err(format!("Failed to convert response to string: {}", e))
                    }
                }
            }
            Err(e) => {
                // Log a detailed error if the ureq call fails.
                error!("ureq_async:agent.get(&url) failed for URL {}: {:?}", url, e);
                Err(format!("HTTP request failed: {}", e))
            }
        }
    });

    // Await the spawned task and handle its result.
    // The `?` operator here will propagate any `Err` from the spawned task.
    s.await
        .map_err(|e| format!("Asynchronous task failed: {}", e))?
}

/// pub fn get_epoch_secs() -> f64
pub fn get_epoch_secs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}
/// pub fn get_epoch_millisecs() -> f64
pub fn get_epoch_millisecs() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 1000f64
    //.as_millis()
}
/// pub fn get_current_working_dir() -> std::io::Result\<PathBuf\>
pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}
/// pub fn strip_trailing_newline(input: &str) -> &str
pub fn strip_trailing_newline(input: &str) -> &str {
    input
        .strip_suffix("\r\n")
        .or(input.strip_suffix("\n"))
        .unwrap_or(input)
}

pub fn find_available_port() -> u16 {
      StdTcpListener::bind("127.0.0.1:0")
          .unwrap()
          .local_addr()
          .unwrap()
          .port()
}
pub async fn async_find_available_port() -> u16 {
      AsyncStdTcpListener::bind("127.0.0.1:0")
          .await.unwrap()
          .local_addr()
          .unwrap()
          .port()
}

// Example usage (you would typically put this in a main function or a test)
#[cfg(test)]
mod tests {
    use super::*;

    // Initialize logging for tests
    fn setup_logging() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_ureq_async_success() {
        setup_logging();
        // Use a reliable test URL, e.g., a public API that returns JSON
        let url = "https://jsonplaceholder.typicode.com/todos/1".to_string();
        let result = ureq_async(url).await;
        assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
        let body = result.unwrap();
        assert!(!body.is_empty());
        // You can add more assertions here to check the content of the body
        println!("Async success body: {}", body);
    }

    #[tokio::test]
    async fn test_ureq_async_failure() {
        setup_logging();
        // Use a URL that is expected to fail or time out quickly
        let url = "http://127.0.0.1:9999/nonexistent".to_string(); // Localhost non-existent port
        let result = ureq_async(url).await;
        assert!(result.is_err());
        println!("Async failure error: {:?}", result.err());
    }

    #[test]
    fn test_ureq_sync_success() {
        setup_logging();
        let url = "https://jsonplaceholder.typicode.com/todos/1".to_string();
        let result = ureq_sync(url);
        assert!(result.is_ok(), "Expected success, got: {:?}", result.err());
        let body = result.unwrap();
        assert!(!body.is_empty());
        println!("Sync success body: {}", body);
    }

    #[test]
    fn test_ureq_sync_failure() {
        setup_logging();
        let url = "http://127.0.0.1:9999/nonexistent".to_string(); // Localhost non-existent port
        let result = ureq_sync(url);
        assert!(result.is_err());
        println!("Sync failure error: {:?}", result.err());
    }

    #[tokio::test]
    async fn test_parse_key_hex_input() {
        let hex_key =
            String::from("f4deaad98b61fa24d86ef315f1d5d57c1a6a533e1e87e777e5d0b48dcd332cdb");
        let result = parse_key_or_id_to_hex_string(hex_key.clone()).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), hex_key);
    }

    #[tokio::test]
    async fn test_parse_key_bech32_note_input() {
        let bech32_note_id =
            String::from("note1h445ule4je70k7kvddate8kpsh2fd6n77esevww5hmgda2qwssjsw957wk");

        let result = parse_key_or_id_to_hex_string(bech32_note_id).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            String::from("bd6b4e7f35967cfb7acc6b7abc9ec185d496ea7ef6619639d4bed0dea80e8425")
        );
    }

    #[tokio::test]
    async fn test_parse_bech32_public_key_input() {
        let bech32_encoded_key =
            String::from("npub1ktt8phjnkfmfrsxrgqpztdjuxk3x6psf80xyray0l3c7pyrln49qhkyhz0");
        let result = parse_key_or_id_to_hex_string(bech32_encoded_key).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            String::from("b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a")
        );
    }

    #[tokio::test]
    async fn test_parse_bech32_private_key() {
        let bech32_encoded_key =
            String::from("nsec1hdeqm0y8vgzuucqv4840h7rlpy4qfu928ulxh3dzj6s2nqupdtzqagtew3");
        let result = parse_key_or_id_to_hex_string(bech32_encoded_key).await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            String::from("bb720dbc876205ce600ca9eafbf87f092a04f0aa3f3e6bc5a296a0a983816ac4")
        );
    }

    #[test]
    fn test_parse_json_valid() {
        let json_string = r#"{"key": "value", "number": 123}"#;
        let result = parse_json(json_string);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["key"], "value");
        assert_eq!(value["number"], 123);
    }

    #[test]
    fn test_parse_json_invalid() {
        let json_string = r#"{"key": "value", "number": 123"#; // Malformed JSON
        let result = parse_json(json_string);
        assert!(result.is_err());
    }

    #[test]
    fn test_split_value_by_newline_string() {
        let json_value = serde_json::json!("line1
line2
line3");
        let result = split_value_by_newline(&json_value);
        assert!(result.is_some());
        let lines = result.unwrap();
        assert_eq!(lines, vec!["line1", "line2", "line3"]);
    }

    #[test]
    fn test_split_value_by_newline_non_string() {
        let json_value = serde_json::json!(123);
        let result = split_value_by_newline(&json_value);
        assert!(result.is_none());
    }

    #[test]
    fn test_value_to_string_null() {
        let value = serde_json::json!(null);
        assert_eq!(value_to_string(&value), "null");
    }

    #[test]
    fn test_value_to_string_bool() {
        let value = serde_json::json!(true);
        assert_eq!(value_to_string(&value), "true");
    }

    #[test]
    fn test_value_to_string_number() {
        let value = serde_json::json!(123);
        assert_eq!(value_to_string(&value), "123");
    }

    #[test]
    fn test_value_to_string_string() {
        let value = serde_json::json!("hello");
        assert_eq!(value_to_string(&value), "hello");
    }

    #[test]
    fn test_value_to_string_array() {
        let value = serde_json::json!([1, "two", true]);
        assert_eq!(value_to_string(&value), "[1, two, true]");
    }

    #[test]
    fn test_value_to_string_object() {
        let value = serde_json::json!({"a": 1, "b": "two"});
        // Order of keys in object might not be guaranteed, so check both possibilities
        let result = value_to_string(&value);
        assert!(result == r#"{"a": 1, "b": two}"# || result == r#"{"b": two, "a": 1}"#);
    }

    #[test]
    fn test_split_json_string_with_separator() {
        let json_value = serde_json::json!("one,two,three");
        let result = split_json_string(&json_value, ",");
        assert_eq!(result, vec!["one", "two", "three"]);
    }

    #[test]
    fn test_split_json_string_no_separator() {
        let json_value = serde_json::json!("onetwothree");
        let result = split_json_string(&json_value, ",");
        assert_eq!(result, vec!["onetwothree"]);
    }

    #[test]
    fn test_split_json_string_non_string_value() {
        let json_value = serde_json::json!(123);
        let result = split_json_string(&json_value, ",");
        assert_eq!(result, vec![""]);
    }

    #[tokio::test]
    async fn test_parse_private_key_some_nsec() {
        let nsec_key = "nsec1hdeqm0y8vgzuucqv4840h7rlpy4qfu928ulxh3dzj6s2nqupdtzqagtew3".to_string();
        let keys = parse_private_key(Some(nsec_key), false).await.unwrap();
        assert!(keys.secret_key().is_ok());
    }

    #[tokio::test]
    async fn test_parse_private_key_some_hex() {
        let hex_key = "bb720dbc876205ce600ca9eafbf87f092a04f0aa3f3e6bc5a296a0a983816ac4".to_string();
        let keys = parse_private_key(Some(hex_key), false).await.unwrap();
        assert!(keys.secret_key().is_ok());
    }

    #[tokio::test]
    async fn test_parse_private_key_none() {
        let keys = parse_private_key(None, false).await.unwrap();
        assert!(keys.secret_key().is_ok());
    }

    #[test]
    fn test_truncate_chars() {
        assert_eq!(truncate_chars("hello world", 5), "hello");
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("h", 0), "");
        assert_eq!(truncate_chars("", 5), "");
    }

    #[test]
    fn test_byte_array_to_hex_string() {
        let byte_array = [
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54,
            0x32, 0x10, 0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55,
            0x44, 0x33, 0x22, 0x11,
        ];
        let expected_hex = "0123456789abcdeffedcba987654321000ffeeddccbbaa998877665544332211";
        assert_eq!(byte_array_to_hex_string(&byte_array), expected_hex);
    }

    #[test]
    fn test_get_epoch_secs() {
        let secs = get_epoch_secs();
        assert!(secs > 0.0);
        // Check if it's reasonably close to current time (within a few seconds)
        let current_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
        assert!((current_secs - secs).abs() < 5.0);
    }

    #[test]
    fn test_get_epoch_millisecs() {
        let millisecs = get_epoch_millisecs();
        assert!(millisecs > 0.0);
        // Check if it's reasonably close to current time (within a few seconds)
        let current_millisecs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64() * 1000.0;
        assert!((current_millisecs - millisecs).abs() < 5000.0); // Within 5 seconds
    }

    #[test]
    fn test_get_current_working_dir() {
        let cwd = get_current_working_dir();
        assert!(cwd.is_ok());
        let path = cwd.unwrap();
        assert!(path.is_dir());
        // You can add more specific checks if you know the expected CWD during tests
    }

    #[test]
    fn test_strip_trailing_newline_lf() {
        assert_eq!(strip_trailing_newline("hello
"), "hello");
    }

    #[test]
    fn test_strip_trailing_newline_crlf() {
        assert_eq!(strip_trailing_newline("hello
"), "hello");
    }

    #[test]
    fn test_strip_trailing_newline_no_newline() {
        assert_eq!(strip_trailing_newline("hello"), "hello");
    }

    #[test]
    fn test_strip_trailing_newline_empty() {
        assert_eq!(strip_trailing_newline(""), "");
    }

    #[test]
    fn test_pwd() {
        let result = crate::utils::pwd::pwd();
        assert!(result.is_ok());
        let pwd_output = result.unwrap();
        // On macOS/Linux, it should return the last component of the path
        // On Windows, it returns the full path.
        // For now, just check if it's not empty.
        assert!(!pwd_output.is_empty());
    }
}
