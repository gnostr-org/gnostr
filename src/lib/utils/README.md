# Gnostr Utilities (`utils` module)

The `src/lib/utils` directory in the Gnostr project houses a collection of essential utility functions and modules designed to enhance the robustness, functionality, and ease of use of the Gnostr ecosystem. These utilities cover a range of common programming tasks, from handling retries and file system operations to managing Nostr-specific data and performing network requests.

## Key Utilities

### Retry Mechanisms (`retry` module)

The `retry` module provides a powerful and flexible way to handle operations that might fail due to transient issues. It offers various retry strategies to ensure operations are resilient and can recover from temporary errors.

*   **Strategies**:
    *   **Linear Retry**: Retries operations with a constant delay between attempts.
    *   **Exponential Retry**: Retries operations with a delay that doubles after each failed attempt, allowing for more aggressive retries in critical situations.
    *   **Asynchronous Variants**: Both linear and exponential strategies have asynchronous counterparts (`LinearAsync`, `ExponentialAsync`) for use in non-blocking operations, requiring the `async` feature.
*   **Execution**:
    *   **`run`**: Executes a synchronous closure with the chosen retry strategy.
    *   **`run_async`**: Executes an asynchronous closure with the chosen retry strategy.
*   **Traits**: `SyncReturn` and `AsyncReturn` traits define the interface for operations that can be retried, ensuring compatibility with various function types.
*   **Examples and Tests**: Comprehensive examples and unit tests are provided to demonstrate usage and verify functionality.

### Path and File System Utilities (`pwd` module)

This module provides essential functions for interacting with the file system, particularly for managing the current working directory.

*   **`get_current_working_dir()`**: Retrieves the current working directory of the process.
*   **`pwd::pwd()`**: Another function to obtain the current working directory.

### JSON Handling

Utilities for parsing and manipulating JSON data:

*   **`parse_json(json_string: &str)`**: Parses a JSON string into a `serde_json::Value`.
*   **`split_value_by_newline(json_value: &Value)`**: Splits a JSON string value by newline characters.
*   **`value_to_string(value: &Value)`**: Converts a `serde_json::Value` into its string representation.
*   **`split_json_string(value: &Value, separator: &str)`**: Splits a JSON string value using a specified separator.

### Key and ID Management

Functions for handling Nostr-related keys and identifiers:

*   **`parse_private_key(private_key: Option<String>, print_keys: bool)`**: Parses a private key from a bech32 or hex string, or generates a new one if none is provided. It can also optionally print the generated keys.
*   **`create_client(keys: &Keys, relays: Vec<String>, difficulty: u8)`**: Creates and configures a Nostr client with specified keys, relays, and difficulty settings.
*   **`parse_key_or_id_to_hex_string(input: String)`**: Converts various Nostr identifiers (like `npub`, `nsec`, `note`, `nprofile`) into their hexadecimal string representations.

### String Manipulation

Helper functions for string processing:

*   **`truncate_chars(s: &str, max_chars: usize)`**: Truncates a string to a specified maximum number of characters.
*   **`strip_trailing_newline(input: &str)`**: Removes trailing newline characters (`\n` or `\r\n`) from a string.

### HTTP Request Utilities

Functions for making HTTP requests with improved error handling and timeouts:

*   **`ureq_sync(url: String)`**: Performs synchronous HTTP GET requests using the `ureq` crate, with configurable timeouts and graceful error handling.
*   **`ureq_async(url: String)`**: Performs asynchronous HTTP GET requests using `tokio` and `ureq`, also featuring robust error handling and timeouts.

### Time Utilities

Functions for retrieving time-related information:

*   **`get_epoch_secs()`**: Returns the current Unix epoch time in seconds as an `f64`.
*   **`get_epoch_millisecs()`**: Returns the current Unix epoch time in milliseconds as an `f64`.

### Hex Conversion

*   **`byte_array_to_hex_string(byte_array: &[u8; 32])`**: Converts a 32-byte array into its hexadecimal string representation.

These utilities collectively provide a robust set of tools for common tasks within the Gnostr project, enhancing code quality, resilience, and developer productivity.
