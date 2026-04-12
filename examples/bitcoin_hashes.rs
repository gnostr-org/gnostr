use bitcoin_hashes::{sha256, Hash};
pub fn hash(input_string: &str) -> Result<String, String> {
    // Convert the input string slice into a byte slice.
    // Hashing algorithms operate on raw bytes.
    let bytes_to_hash = input_string.as_bytes();

    // Compute the SHA256 hash of the byte slice.
    // `sha256::Hash::hash()` is an associated function that takes
    // a byte slice and returns a `sha256::Hash` object.
    let computed_hash = sha256::Hash::hash(bytes_to_hash);

    // Convert the `sha256::Hash` object into its hexadecimal string representation.
    // The `to_string()` method is provided by the `Display` trait, which
    // `sha256::Hash` implements.
    let hash_string = computed_hash.to_string();

    // Wrap the resulting hash string in an `Ok` variant to indicate success.
    Ok(hash_string)
}

fn main() {
    let text_to_hash = "Hello, Bitcoin Hashes!";
    match hash(text_to_hash) {
        Ok(hashed_value) => {
            println!("Original String: \"{}\"", text_to_hash);
            println!("SHA256 Hash:     {}", hashed_value);
        }
        Err(e) => {
            eprintln!("Error hashing string: {}", e);
        }
    }

    println!("\n--- More Examples ---");

    let texts_to_hash = vec![
        "Rust programming is fun",
        "12345",
        "The quick brown fox jumps over the lazy dog",
        "", // Hashing an empty string
    ];

    for text in texts_to_hash {
        match hash(text) {
            Ok(hashed_value) => {
                println!("Original: \"{}\"", text);
                println!("Hash:     {}", hashed_value);
                println!("--------------------------------------------------");
            }
            Err(e) => {
                eprintln!("Error hashing \"{}\": {}", text, e);
                println!("--------------------------------------------------");
            }
        }
    }
}
