//! A crate providing the `get_file_hash!` procedural macro.
//!
//! This macro allows you to compute the SHA-256 hash of a file at compile time,
//! embedding the resulting hash string directly into your Rust executable.

use url::Url;

pub use gnostr_filehash_core::get_file_hash;
pub use gnostr_filehash_core::get_git_tracked_files;
pub use gnostr_filehash_core::should_remove_relay;

use crate::types::{Client, Error, EventBuilder, EventKind, Id, Keys, Options, Tag};

const ONLINE_RELAYS_GPS_CSV: &[u8] = include_bytes!("core/src/online_relays_gps.csv");

/// The SHA-256 hash of this crate's `build.rs` at the time of compilation.
pub const BUILD_HASH: &str = env!("BUILD_HASH");

/// The SHA-256 hash of this crate's `Cargo.toml` at the time of compilation.
pub const CARGO_TOML_HASH: &str = env!("CARGO_TOML_HASH");

/// The SHA-256 hash of this crate's `src/lib.rs` at the time of compilation.
pub const LIB_HASH: &str = env!("LIB_HASH");

/// The name of the package as specified in Cargo.toml.
pub const CARGO_PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// The version of the package as specified in Cargo.toml.
pub const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(feature = "nostr")]
/// The git commit hash of the repository at the time of compilation.
pub const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

#[cfg(feature = "nostr")]
/// The git branch of the repository at the time of compilation.
pub const GIT_BRANCH: &str = env!("GIT_BRANCH");

pub fn get_relay_urls() -> Vec<String> {
    let content = String::from_utf8_lossy(ONLINE_RELAYS_GPS_CSV);
    content
        .lines()
        .skip(1)
        .filter_map(|line| {
            let url_str = line.split(',').next()?.trim();
            if url_str.is_empty() {
                return None;
            }
            let full_url_str = if url_str.contains("://") {
                url_str.to_string()
            } else {
                format!("wss://{}", url_str)
            };
            match Url::parse(&full_url_str) {
                Ok(url) if url.scheme() == "wss" => Some(url.to_string()),
                _ => {
                    eprintln!("Warning: Invalid or unsupported relay URL scheme: {}", full_url_str);
                    None
                }
            }
        })
        .collect()
}

pub async fn publish_patch_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&Id>,
) -> Result<Id, Error> {
    let mut client = Client::new(keys, Options::new());
    client.add_relays(relay_urls.to_vec()).await?;
    client.connect().await;

    let mut tags = vec![
        Tag::new_identifier(d_tag_value.to_string()),
        Tag::new_tag("commit", commit_id),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::new_event(*event_id, None, None));
    }

    let event = EventBuilder::new(EventKind::Patches, patch_content.to_string(), tags)
        .to_event(&keys.secret_key()?)?;

    let event_id = client.send_event(event).await?;
    Ok(event_id)
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    /// Verifies that the exported CARGO_TOML_HASH is not empty.
    #[test]
    fn test_injected_hash_exists() {
        assert!(!BUILD_HASH.is_empty());
        println!("Verified build.rs Hash:
{}", BUILD_HASH);

        assert!(!CARGO_TOML_HASH.is_empty());
        println!("Verified Cargo.toml Hash:
{}", CARGO_TOML_HASH);

        assert!(!LIB_HASH.is_empty());
        println!("Verified src/lib.rs Hash:\n{}", LIB_HASH);

        assert!(!CARGO_PKG_NAME.is_empty());
        println!("Verified Package Name:\n{}", CARGO_PKG_NAME);

        assert!(!CARGO_PKG_VERSION.is_empty());
        println!("Verified Package Version:\n{}", CARGO_PKG_VERSION);

        #[cfg(feature = "nostr")]
        {
            assert!(!GIT_COMMIT_HASH.is_empty());
            println!("Verified Git Commit Hash:\n{}", GIT_COMMIT_HASH);

            assert!(!GIT_BRANCH.is_empty());
            println!("Verified Git Branch:\n{}", GIT_BRANCH);
        }
    }

    /// Tests that the `get_file_hash!` macro correctly computes the SHA-256
    /// hash of `lib.rs` and that it matches a manually computed hash of the
    /// same file.
    #[test]
    fn test_get_lib_hash() {
        let file_content = include_bytes!("lib.rs");

        let mut hasher = Sha256::new();
        hasher.update(file_content);
        let expected_hash = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let actual_hash = get_file_hash!("lib.rs");
        assert_eq!(actual_hash, expected_hash);
    }
}
