#[cfg(feature = "nostr")]
use serde_json::to_string;
use std::process::Command;
use std::path::PathBuf;
#[cfg(feature = "nostr")]
use nostr_sdk::prelude::{*, EventBuilder, Tag, Kind};
#[cfg(feature = "nostr")]
use serde_json::json;
#[cfg(feature = "nostr")]
use csv::ReaderBuilder;
#[cfg(feature = "nostr")]
use ::url::Url;
#[cfg(feature = "nostr")]
pub use frost_secp256k1_tr as frost;
#[cfg(feature = "nostr")]
use frost::keys::{KeyPackage, PublicKeyPackage, SecretShare};
#[cfg(feature = "nostr")]
use frost::round1::{SigningCommitments, SigningNonces};
#[cfg(feature = "nostr")]
use frost::round2::SignatureShare;
#[cfg(feature = "nostr")]
use frost::SigningPackage;
#[cfg(feature = "nostr")]
use rand::thread_rng;
#[cfg(feature = "nostr")]
pub use frost_secp256k1_tr as frost_bip340;

pub mod frost_mailbox_logic;

#[cfg(feature = "nostr")]
use std::collections::BTreeMap;

pub const DUMMY_BUILD_MANIFEST_ID_STR: &str = "f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0";
pub const DEFAULT_GNOSTR_KEY: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
pub const DEFAULT_PICTURE_URL: &str = "https://avatars.githubusercontent.com/u/135379339?s=400&u=11cb72cccbc2b13252867099546074c50caef1ae&v=4";
pub const DEFAULT_BANNER_URL: &str = "https://raw.githubusercontent.com/gnostr-org/gnostr-icons/refs/heads/master/banner/1024x341.png";

pub const EMPTY_BLOB_SHA1: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
pub const EMPTY_BLOB_SHA256: &str = "473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813";
pub const EMPTY_BLOB_PRIVATE_KEY_NSEC: &str = "nsec1guaq7npmaz5ndqdzvl3mr6d8mndprp2rdls5ram5jys2xqmjrqfsdzhrp6";

pub const EMPTY_TREE_SHA1: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
pub const EMPTY_TREE_SHA256: &str = "6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321";
pub const EMPTY_TREE_PRIVATE_KEY_NSEC: &str = "nsec1dmceksfzt3fknuwpqn29mrv9a75mq4a48v2tfwde88whfhkv2vsslsc46c";

#[cfg(feature = "nostr")]
const ONLINE_RELAYS_GPS_CSV: &[u8] = include_bytes!("online_relays_gps.csv");

/// BIP-64MOD + GCC: Complete NIP-19 Identity Mapping
/// 
/// These constants provide the Bech32 encoded Private (NSEC) and 
/// Public (NPUB) keys for Git-standard empty states.
#[cfg(feature = "nostr")]
pub struct GitEmptyIdentity;

#[cfg(feature = "nostr")]
impl GitEmptyIdentity {
    // === EMPTY BLOB IDENTITY ===
    // Derived from the identity of a 0-byte file.
    pub const BLOB_NSEC: &'static str = "nsec1guaq7npmaz5ndqdzvl3mr6d8mndprp2rdls5ram5jys2xqmjrqfsdzhrp6";
    pub const BLOB_NPUB: &'static str = "npub180cvv07tjdrghvkyh6964p7w9vsqpf3p05868v399v86p8y6f69sq5fdp0";
    pub const BLOB_HEX:  &'static str = "473a0f4c3be8a93681a267e3b1e9a7dcda1185436fe141f7749120a303721813";

    // === EMPTY TREE IDENTITY ===
    // Derived from the identity of an empty directory.
    pub const TREE_NSEC: &'static str = "nsec1dmceksfzt3fknuwpqn29mrv9a75mq4a48v2tfwde88whfhkv2vsslsc46c";
    pub const TREE_NPUB: &'static str = "npub1pxmpep6yk7z6p332u9588k0vscg26rv29pynvscg26rv29pynvsq6erdfh";
    pub const TREE_HEX:  &'static str = "6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321";

    // === NULL / GENESIS IDENTITY ===
    // Often used for the 'System' or 'Root' user in a new GCC chain.
    // Derived from 32-bytes of zeros.
    pub const NULL_NSEC: &'static str = "nsec1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqp3994m";
    pub const NULL_NPUB: &'static str = "npub1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqad8gh8";
    pub const NULL_HEX:  &'static str = "0000000000000000000000000000000000000000000000000000000000000000";
}

/// Example usage for signature verification logic
#[cfg(feature = "nostr")]
pub mod git_empty_state {
    #[cfg(feature = "nostr")]
	use crate::GitEmptyIdentity;

    /// Returns the expected public key for a given Git object hash.
    /// Useful for automated verification of 'Empty State' transitions.
    pub fn get_expected_npub(git_hash: &str) -> Option<&'static str> {
        match git_hash {
            GitEmptyIdentity::BLOB_HEX => Some(GitEmptyIdentity::BLOB_NPUB),
            GitEmptyIdentity::TREE_HEX => Some(GitEmptyIdentity::TREE_NPUB),
            GitEmptyIdentity::NULL_HEX => Some(GitEmptyIdentity::NULL_NPUB),
            _ => None,
        }
    }
}

#[cfg(feature = "nostr")]
pub fn get_relay_urls() -> Vec<String> {
    let content = String::from_utf8_lossy(ONLINE_RELAYS_GPS_CSV);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(content.as_bytes());

    rdr.records()
        .filter_map(|result| {
            match result {
                Ok(record) => {
                    record.get(0).and_then(|url_str| {
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
                },
                Err(e) => {
                    eprintln!("Error reading CSV record: {}", e);
                    None
                }
            }
        })
        .collect()
}

#[cfg(feature = "nostr")]
use std::io::Write;
#[cfg(feature = "nostr")]
use std::fs;

#[cfg(feature = "nostr")]
pub fn should_remove_relay(error_msg: &str) -> bool {
    error_msg.contains("relay not connected") ||
    error_msg.contains("not in web of trust") ||
    error_msg.contains("blocked: not authorized") ||
    error_msg.contains("timeout") ||
    error_msg.contains("blocked: spam not permitted") ||
    error_msg.contains("relay experienced an error trying to publish the latest event") ||
    error_msg.contains("duplicate: event already broadcast")
}

#[cfg(feature = "nostr")]
pub fn write_event_json_to_file(
    output_dir: &PathBuf,
    filename: &str,
    event: &Event,
) -> Option<()> {
    let file_path = output_dir.join(filename);
    if let Some(parent) = file_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            println!("cargo:warning=Failed to create parent directories for {}: {}", file_path.display(), e);
            return None;
        }
    }
    if let Err(e) = fs::File::create(&file_path).and_then(|mut file| write!(file, "{}", event.as_json())) {
        println!("cargo:warning=Failed to write event JSON to file {}: {}", file_path.display(), e);
        None
    } else {
        println!("Successfully wrote event JSON to {}", file_path.display());
        Some(())
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_nostr_event_if_release(
    client: &mut nostr_sdk::Client,
    hash: String,
    keys: Keys,
    event_builder: EventBuilder,
    _relay_urls: &mut Vec<String>,
    file_path_str: &str,
    output_dir: &PathBuf,
    total_bytes_sent: &mut usize,
) -> Option<EventId> {
    let public_key = keys.public_key().to_string();

    let event = client.sign_event_builder(event_builder).await.unwrap();

    match client.send_event(&event).await {        Ok(event_output) => {
            println!("cargo:warning=Published Nostr event for {}: {}", file_path_str, event_output.val);

            let event_json_size = to_string(&event).map(|s| s.as_bytes().len()).unwrap_or(0);
            // Print successful relays
            for relay_url in event_output.success.iter() {
                println!("cargo:warning=Successfully published to relay: {} ({} bytes)", relay_url, event_json_size);
                *total_bytes_sent += event_json_size;
            }
            // Print failed relays and remove "unfriendly" relays from the list
            for (relay_url, error_msg) in event_output.failed.iter() {
                if should_remove_relay(error_msg) {
                    if let Err(e) = client.remove_relay(relay_url).await {
                        println!("cargo:warning=Failed to remove relay {}: {}", relay_url, e);
                    }
                     // println!("cargo:warning=Removed relay {}", relay_url);
                }
            }

            let filename = format!("{}/{}/{}/{}.json", file_path_str, hash, public_key.clone(), event_output.val.to_string());
            write_event_json_to_file(output_dir, &filename, &event);
            Some(event_output.val)
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr event for {}: {}", file_path_str, e);
            None
        },
    }
}

#[cfg(feature = "nostr")]
pub async fn get_repo_announcement_event(
    client: &mut nostr_sdk::Client,
    _keys: &Keys,
    relay_urls: &Vec<String>,
    repo_url: &str,
    repo_name: &str,
    repo_description: &str,
    git_commit_hash: &str,
    git_branch: &str,
    output_dir: &PathBuf,
    public_key_hex: &str,
) -> Option<EventId> {

    let mut tags = vec![
        Tag::parse(["d", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["name", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["description", repo_description].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["web", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["clone", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["r", git_commit_hash, "euc"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["commit", git_commit_hash].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["branch", git_branch].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["maintainers", "gnostr"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        //Tag::parse(["t", "personal-fork"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["t", "gnostr"].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["t", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
    ];

    // Append each relay url
    for relay in relay_urls {
        tags.push(Tag::parse(["relays", relay].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap());
    }
    let event_builder = EventBuilder::new(Kind::Custom(30617), repo_description).tags(tags);
    let event = client.sign_event_builder(event_builder).await.unwrap();

    match client.send_event(&event).await {
        Ok(event_output) => {
            println!("cargo:warning=Published Nostr Repository Announcement for {}: {}", repo_name, event_output.val);
            
            let filename = format!("30617/{}/{}/{}.json", repo_name, public_key_hex, event_output.val.to_string());
            write_event_json_to_file(output_dir, &filename, &event);
            Some(event_output.val)
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr Repository Announcement for {}: {}", repo_name, e);
            None
        },
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_repo_patch_event(
    client: &mut nostr_sdk::Client,
    _keys: &Keys,
    _relay_urls: &Vec<String>,
    repo_url: &str,
    repo_name: &str,
    repo_description: &str,
    git_commit_hash: &str,
    git_branch: &str,
    output_dir: &PathBuf,
    public_key_hex: &str,
) -> Option<EventId> {

    let tags = vec![
        Tag::parse(["r", repo_url].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["name", repo_name].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["description", repo_description].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["commit", git_commit_hash].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
        Tag::parse(["branch", git_branch].iter().map(ToString::to_string).collect::<Vec<String>>()).unwrap(),
    ];

    let event_builder = EventBuilder::new(Kind::Custom(1617), repo_description).tags(tags);
    let event = client.sign_event_builder(event_builder).await.unwrap();

    match client.send_event(&event).await {
        Ok(event_output) => {
            println!("cargo:warning=Published Nostr Repository Announcement for {}: {}", repo_name, event_output.val);
            
            let filename = format!("30617/{}/{}/{}.json", repo_name, public_key_hex, event_output.val.to_string());
            write_event_json_to_file(output_dir, &filename, &event);
            Some(event_output.val)
        },
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr Repository Announcement for {}: {}", repo_name, e);
            None
        },
    }
}

/// Computes the SHA-256 hash of the specified file at compile time.
///
/// This macro takes a string literal representing a file path, reads the file's bytes
/// at compile time, computes its SHA-256 hash, and returns the hash as a hex-encoded `String`.
///
/// # Examples
///
/// ```rust
/// use get_file_hash_core::get_file_hash;
/// use sha2::{Digest, Sha256};
///
/// let hash = get_file_hash!("lib.rs");
/// println!("Hash: {}", hash);
/// ```

#[macro_export]
macro_rules! get_file_hash {
    ($file_path:expr) => {{
        let bytes = include_bytes!($file_path);
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();

        // Convert the GenericArray to a hex string
        result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }};
}

/// Computes the SHA-256 hash of the specified file at compile time and uses it as a Nostr private key.
///
/// This macro takes a string literal representing a file path, computes its SHA-256 hash,
/// and returns a `nostr::Keys` object derived from this hash.
///
/// # Examples
///
/// ```rust
/// use get_file_hash_core::file_hash_as_nostr_private_key;
/// use sha2::{Digest, Sha256};
/// use nostr_sdk::prelude::ToBech32;
///
/// let keys = file_hash_as_nostr_private_key!("lib.rs");
/// println!("Public Key: {}", keys.public_key().to_bech32().unwrap());
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! file_hash_as_nostr_private_key {
    ($file_path:expr) => {{
        let hash_hex = $crate::get_file_hash!($file_path);
        nostr_sdk::Keys::parse(&hash_hex).expect("Failed to create Nostr Keys from file hash")
    }};
}

/// Publishes a NIP-34 repository announcement event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, project details, a clone URL, and a file path.
/// It computes the SHA-256 hash of the file at compile time to use as the "earliest unique commit" (EUC),
/// and then publishes a Kind 30617 event.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::repository_announcement;
/// use get_file_hash_core::get_file_hash;
/// use nostr_sdk::Keys;
/// use sha2::{Digest, Sha256};
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let project_name = "my-awesome-repo";
///     let description = "A fantastic new project.";
///     let clone_url = "git@github.com:user/my-awesome-repo.git";
///
///     repository_announcement!(
///         &keys,
///         &relay_urls,
///         project_name,
///         description,
///         clone_url,
///         "../Cargo.toml", // Use a known file in your project
///         None
///     );
/// }
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! repository_announcement {
    ($keys:expr, $relay_urls:expr, $project_name:expr, $description:expr, $clone_url:expr, $file_for_euc:expr) => {{
        let euc_hash = $crate::get_file_hash!($file_for_euc);
        // The 'd' tag value should be unique for the repository. Using the project_name for simplicity.
        let d_tag_value = $project_name;
        $crate::publish_repository_announcement_event(
            $keys,
            $relay_urls,
            $project_name,
            $description,
            $clone_url,
            &euc_hash,
            d_tag_value,
            None,
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $project_name:expr, $description:expr, $clone_url:expr, $file_for_euc:expr, $build_manifest_event_id:expr) => {{
        let euc_hash = $crate::get_file_hash!($file_for_euc);
        let d_tag_value = $project_name;
        $crate::publish_repository_announcement_event(
            $keys,
            $relay_urls,
            $project_name,
            $description,
            $clone_url,
            &euc_hash,
            d_tag_value,
            $build_manifest_event_id, // Correct: Pass directly
        ).await;
    }};
}

/// Publishes a NIP-34 patch event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the commit ID the patch applies to, and the path to the patch file.
/// The content of the patch file is included directly in the event.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_patch;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0"; // Example commit ID
///
///     publish_patch!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         commit_id,
///         "lib.rs" // Use an existing file for the patch content
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_patch {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $patch_file_path:expr) => {{
        let patch_content = include_str!($patch_file_path);
        $crate::publish_patch_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $commit_id,
            patch_content,
            None, // Pass None for build_manifest_event_id
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $patch_file_path:expr, $build_manifest_event_id:expr) => {{
        let patch_content = include_str!($patch_file_path);
        $crate::publish_patch_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $commit_id,
            patch_content,
            $build_manifest_event_id, // Pass directly, macro arg should be Option<&EventId>
        ).await;
    }};
}

/// Publishes a NIP-34 pull request event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the commit ID of the pull request, a clone URL where the work can be fetched,
/// and an optional title for the pull request.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_pull_request;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
///     let clone_url = "git@github.com:user/my-feature-branch.git";
///     let title = Some("Feat: Add new awesome feature");
///
///     publish_pull_request!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         commit_id,
///         clone_url,
///         title,
///         None
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_pull_request {
    // 5 args: No title, no build_manifest_event_id
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $clone_url:expr) => {{
        $crate::publish_pull_request_event(
            $keys, $relay_urls, $d_tag_value, $commit_id, $clone_url,
            None, // title: Option<&str>
            None, // build_manifest_event_id: Option<&EventId>
        ).await;
    }};

    // 6 args: With title (Option<&str>), no build_manifest_event_id
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $clone_url:expr, $title:expr) => {{
        $crate::publish_pull_request_event(
            $keys, $relay_urls, $d_tag_value, $commit_id, $clone_url,
            $title, // title: Option<&str>
            None, // build_manifest_event_id: Option<&EventId>
        ).await;
    }};

    // 7 args: With title (Option<&str>), with build_manifest_event_id (Option<&EventId>)
    // This needs to be before the 6-arg arm that passes a single Option for build_manifest_event_id if it's not None.
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $clone_url:expr, $title:expr, $build_manifest_event_id:expr) => {{
        $crate::publish_pull_request_event(
            $keys, $relay_urls, $d_tag_value, $commit_id, $clone_url,
            $title, // title: Option<&str>
            $build_manifest_event_id, // build_manifest_event_id: Option<&EventId>
        ).await;
    }};

    // 6 args: No title, with build_manifest_event_id (Option<&EventId>)
    // This must be after the 7-arg arm to avoid ambiguity.
    // The example needs to explicitly pass None for title.
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $commit_id:expr, $clone_url:expr, _none_title:tt, $build_manifest_event_id:expr) => {{ // _none_title as tt to match None
        $crate::publish_pull_request_event(
            $keys, $relay_urls, $d_tag_value, $commit_id, $clone_url,
            None, // title: Option<&str>
            $build_manifest_event_id, // build_manifest_event_id: Option<&EventId>
        ).await;
    }};
}

/// Publishes a NIP-34 PR update event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the event ID of the original pull request, the new commit ID,
/// and the new clone URL.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_pr_update;
/// use nostr_sdk::Keys;
/// use nostr_sdk::EventId;
/// use std::str::FromStr;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let pr_event_id = EventId::from_str("f6e4d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9").unwrap(); // Example PR Event ID
///     let updated_commit_id = "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0";
///     let updated_clone_url = "git@github.com:user/my-feature-branch-v2.git";
///
///     publish_pr_update!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         &pr_event_id,
///         updated_commit_id,
///         updated_clone_url
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_pr_update {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $pr_event_id:expr, $updated_commit_id:expr, $updated_clone_url:expr) => {{
        $crate::publish_pr_update_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $pr_event_id,
            $updated_commit_id,
            $updated_clone_url,
            None, // Pass None for build_manifest_event_id
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $pr_event_id:expr, $updated_commit_id:expr, $updated_clone_url:expr, $build_manifest_event_id:expr) => {{
        $crate::publish_pr_update_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $pr_event_id,
            $updated_commit_id,
            $updated_clone_url,
            $build_manifest_event_id,

        ).await;
    }};
}

/// Publishes a NIP-34 repository state event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// the branch name, and the commit ID for that branch.
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_repository_state;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let branch_name = "main";
///     let commit_id = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0";
///
///     publish_repository_state!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         branch_name,
///         commit_id
///     );
/// }
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_repository_state {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $branch_name:expr, $commit_id:expr) => {{
        $crate::publish_repository_state_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $branch_name,
            $commit_id,
        ).await;
    }};
}

/// Publishes a NIP-34 issue event to Nostr relays.
///
/// This macro takes Nostr keys, relay URLs, the repository's d-tag value,
/// a unique issue ID, the issue's title, and its content (markdown).
///
/// # Examples
///
/// ```no_run
/// use get_file_hash_core::publish_issue;
/// use nostr_sdk::Keys;
///
/// #[tokio::main]
/// async fn main() {
///     let keys = Keys::generate();
///     let relay_urls = vec!["wss://relay.damus.io".to_string()];
///     let d_tag = "my-awesome-repo";
///     let issue_id = "123";
///     let title = "Bug: Fix authentication flow";
///     let content = "The authentication flow is currently broken when users try to log in with invalid credentials. It crashes instead of showing an error message.";
///
///     publish_issue!(
///         &keys,
///         &relay_urls,
///         d_tag,
///         issue_id,
///         title,
///         content
///     );
/// }
/// ```
/// ```
#[cfg(feature = "nostr")]
#[macro_export]
macro_rules! publish_issue {
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $issue_id:expr, $title:expr, $content:expr) => {{
        $crate::publish_issue_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $issue_id,
            $title,
            $content,
            None, // Pass None for build_manifest_event_id
        ).await;
    }};
    ($keys:expr, $relay_urls:expr, $d_tag_value:expr, $issue_id:expr, $title:expr, $content:expr, $build_manifest_event_id:expr) => {{
        $crate::publish_issue_event(
            $keys,
            $relay_urls,
            $d_tag_value,
            $issue_id,
            $title,
            $content,
            $build_manifest_event_id, // Pass Option<&EventId> directly
        ).await;
    }};
}

pub fn get_git_tracked_files(dir: &PathBuf) -> Vec<String> {
    match Command::new("git")
        .arg("ls-files")
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
    {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(|line| Some(String::from(line)))
                .collect()
        }
        Ok(output) => {
            println!("cargo:warning=git ls-files failed or returned empty. Status: {:?}, Stderr: {}", 
                     output.status, String::from_utf8_lossy(&output.stderr));
            Vec::new()
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute git ls-files: {}", e);
            Vec::new()
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_metadata_event(
    keys: &Keys,
    relay_urls: &[String],
    picture_url: &str,
    banner_url: &str,
    file_path_str: &str,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for metadata {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let metadata_json = json!({
        "picture": picture_url,
        "banner": banner_url,
        "name": file_path_str,
        "about": format!("Metadata for file event: {}", file_path_str),
    });

    let metadata = serde_json::from_str::<nostr_sdk::Metadata>(&metadata_json.to_string())
        .expect("Failed to parse metadata JSON");

    match client.send_event_builder(EventBuilder::metadata(&metadata)).await {
        Ok(_event_id) => {
            //println!("cargo:warning=Published Nostr metadata event for {}: {:?}", file_path_str, event_id);
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish Nostr metadata event for {}: {}", file_path_str, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_repository_announcement_event(
    keys: &Keys,
    relay_urls: &[String],
    project_name: &str,
    description: &str,
    clone_url: &str,
    euc: &str, // Earliest Unique Commit hash
    d_tag_value: &str, // d-tag value
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for repository announcement {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::parse(["name", project_name]).expect("Failed to create name tag"),
        Tag::parse(["description", description]).expect("Failed to create description tag"),
        Tag::parse(["clone", clone_url]).expect("Failed to create clone tag"),
        Tag::custom("euc".into(), vec![euc.to_string()]),
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // NIP-33 d-tag
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(30617), // NIP-34 Repository Announcement kind
        "", // Content is empty for repository announcement
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Repository Announcement for {}. Event ID (raw): {:?}, Event ID (bech32): {}", project_name, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Repository Announcement for {}: {}", project_name, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_patch_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    patch_content: &str,
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for patch {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["commit", commit_id]).expect("Failed to create commit tag"),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(1617), // NIP-34 Patch kind
        patch_content,
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=\nPublished NIP-34 Patch event for commit {}.\nEvent ID (raw): {:?},\nEvent ID (bech32): {}", commit_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Patch event for commit {}: {}", commit_id, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_pull_request_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    commit_id: &str,
    clone_url: &str,
    title: Option<&str>,
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for pull request {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["commit", commit_id]).expect("Failed to create commit tag"),
        Tag::parse(["clone", clone_url]).expect("Failed to create clone tag"),
    ];

    if let Some(t) = title {
        tags.push(Tag::parse(["title", t]).expect("Failed to create title tag"));
    }

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(1618), // NIP-34 Pull Request kind
        "gnostr patch", // Content can be empty or a description for the PR
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Pull Request event for commit {}. Event ID (raw): {:?}, Event ID (bech32): {}", commit_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Pull Request event for commit {}: {}", commit_id, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_pr_update_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    pr_event_id: &EventId,
    updated_commit_id: &str,
    updated_clone_url: &str,
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for PR update {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["p", pr_event_id.to_string().as_str()]).expect("Failed to create PR event ID tag"),
        Tag::parse(["commit", updated_commit_id]).expect("Failed to create updated commit ID tag"),
        Tag::parse(["clone", updated_clone_url]).expect("Failed to create updated clone URL tag"),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(1619), // NIP-34 PR Update kind
        "", // Content is empty for PR update
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 PR Update event for PR {} (raw: {:?}). Event ID (raw): {:?}, Event ID (bech32): {}", pr_event_id.to_bech32().unwrap(), pr_event_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 PR Update event for PR {}: {}", pr_event_id.to_string(), e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_repository_state_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    branch_name: &str,
    commit_id: &str,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for repository state {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let event_builder = EventBuilder::new(
        Kind::Custom(30618), // NIP-34 Repository State kind
        "", // Content is empty for repository state
    ).tags(vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["name", branch_name]).expect("Failed to create branch name tag"),
        Tag::parse(["commit", commit_id]).expect("Failed to create commit ID tag"),
    ]);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Repository State event for branch {} (commit {}). Event ID (raw): {:?}, Event ID (bech32): {}", branch_name, commit_id, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Repository State event for branch {} (commit {}): {}", branch_name, commit_id, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub async fn publish_issue_event(
    keys: &Keys,
    relay_urls: &[String],
    d_tag_value: &str,
    issue_id: &str, // Unique identifier for the issue
    title: &str,
    content: &str,
    build_manifest_event_id: Option<&EventId>,
) {
    let client = nostr_sdk::Client::new(keys.clone());

    for relay_url in relay_urls {
        if let Err(e) = client.add_relay(relay_url).await {
            println!("cargo:warning=Failed to add relay for issue {}: {}", relay_url, e);
        }
    }
    client.connect().await;

    let mut tags = vec![
        Tag::custom("d".into(), vec![d_tag_value.to_string()]), // Repository d-tag
        Tag::parse(["i", issue_id]).expect("Failed to create issue ID tag"),
        Tag::parse(["title", title]).expect("Failed to create title tag"),
    ];

    if let Some(event_id) = build_manifest_event_id {
        tags.push(Tag::event(*event_id));
    }

    let event_builder = EventBuilder::new(
        Kind::Custom(1621), // NIP-34 Issue kind
        content,
    ).tags(tags);

    match client.send_event_builder(event_builder).await {
        Ok(event_id) => {
            println!("cargo:warning=Published NIP-34 Issue event for issue {} ({}). Event ID (raw): {:?}, Event ID (bech32): {}", issue_id, title, event_id, event_id.to_bech32().unwrap());
        }
        Err(e) => {
            println!("cargo:warning=Failed to publish NIP-34 Issue event for issue {} ({}): {}", issue_id, title, e);
        }
    }
}

#[cfg(feature = "nostr")]
pub fn generate_frost_keys(
    max_signers: u16,
    min_signers: u16,
) -> Result<(BTreeMap<frost::Identifier, SecretShare>, PublicKeyPackage), Box<dyn std::error::Error>> {                let mut rng = thread_rng();
    let (shares, pubkey_package) = frost::keys::generate_with_dealer(
        max_signers,
        min_signers,
        frost::keys::IdentifierList::Default,
        &mut rng,
    )?;
    Ok((shares, pubkey_package))
    }

    #[cfg(feature = "nostr")]
    pub fn create_frost_commitment(
    secret_share: &SecretShare,
    ) -> (SigningNonces, SigningCommitments) {
    let mut rng = thread_rng();
    frost::round1::commit(secret_share.signing_share(), &mut rng)
    }

    #[cfg(feature = "nostr")]
    pub fn create_signing_package(
    commitments: BTreeMap<frost::Identifier, SigningCommitments>,
    message: &[u8],
    ) -> SigningPackage {
    frost::SigningPackage::new(commitments, message)
    }

    #[cfg(feature = "nostr")]
    pub fn generate_signature_share(
    signing_package: &SigningPackage,
    nonces: &SigningNonces,
    secret_share: &SecretShare,
    ) -> Result<SignatureShare, Box<dyn std::error::Error>> {
    let key_package: KeyPackage = secret_share.clone().try_into()?;
    Ok(frost::round2::sign(signing_package, nonces, &key_package)?)
    }

    #[cfg(feature = "nostr")]
    pub fn aggregate_signature_shares(
    signing_package: &SigningPackage,
    signature_shares: &BTreeMap<frost::Identifier, SignatureShare>,
    pubkey_package: &PublicKeyPackage,
    ) -> Result<frost_secp256k1_tr::Signature, Box<dyn std::error::Error>> {
    Ok(frost::aggregate(signing_package, signature_shares, pubkey_package)?)
    }

    #[cfg(feature = "nostr")]
    pub fn verify_frost_signature(
    group_public_key: &frost_secp256k1_tr::VerifyingKey,
    message: &[u8],
    signature: &frost_secp256k1_tr::Signature,
    ) -> Result<(), Box<dyn std::error::Error>> {
    Ok(group_public_key.verify(message, signature)?)
    }
#[cfg(test)]
mod tests {
    #[cfg(feature = "nostr")]
    use serial_test::serial;
    #[cfg(feature = "nostr")]
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::io::Write;
    use sha2::{Digest, Sha256};
    use tempfile;
    use super::get_git_tracked_files;
    #[cfg(feature = "nostr")]
    use super::frost;
        use std::process::Command;
    #[cfg(feature = "nostr")]
    use nostr_sdk::EventId;
    #[cfg(feature = "nostr")]
    use std::str::FromStr;

    // Test for get_file_hash! macro
    #[test]
    fn test_get_file_hash() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let content = "Hello, world!";
        File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

        // The macro expects a string literal, so we need to construct the path at compile time.
        // This is a limitation for testing, normally you'd use it with a known file.
        // For testing, we'll manually verify a file known to be in the project.
        // Let's test `lib.rs` itself for a more realistic scenario.
        let macro_hash = get_file_hash!("lib.rs");

        // We will assert on a known file within the crate.
        let bytes = include_bytes!("lib.rs");
        let mut hasher_manual = Sha256::new();
        hasher_manual.update(bytes);
        let expected_hash_lib_rs = hasher_manual.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        assert_eq!(macro_hash, expected_hash_lib_rs);

        // Test with another known file, e.g., Cargo.toml of the core crate
        let cargo_toml_hash = get_file_hash!("../Cargo.toml");
        let cargo_toml_bytes = include_bytes!("../Cargo.toml");
        let mut cargo_toml_hasher = Sha256::new();
        cargo_toml_hasher.update(cargo_toml_bytes);
        let expected_cargo_toml_hash = cargo_toml_hasher.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        assert_eq!(cargo_toml_hash, expected_cargo_toml_hash);
    }

    #[test]
    fn test_get_git_tracked_files() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Initialize a git repository
        let _ = Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to initialize git repo");

        // Create some files
        let file1_path = repo_path.join("file1.txt");
        File::create(&file1_path).unwrap().write_all(b"content1").unwrap();
        let file2_path = repo_path.join("file2.txt");
        File::create(&file2_path).unwrap().write_all(b"content2").unwrap();

        // Add and commit files
        let _ = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to git add files");
        let _ = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to git commit");

        let tracked_files = get_git_tracked_files(&repo_path.to_path_buf());
        assert_eq!(tracked_files.len(), 2);
        assert!(tracked_files.contains(&"file1.txt".to_string()));
        assert!(tracked_files.contains(&"file2.txt".to_string()));
    }

    // #[cfg(feature = "nostr")]
    // #[test]
    // fn test_file_hash_as_nostr_private_key() {
    //     use super::file_hash_as_nostr_private_key;
    //     // use std::fs::{File, remove_file};
    //     // use std::io::Write;
    //     // use tempfile::tempdir; // Not needed as we're using a literal path
    //     use nostr_sdk::prelude::ToBech32;

    //     let file_path = PathBuf::from("test_nostr_file_for_macro.txt");
    //     let content = "Nostr test content!";
    //     File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

    //     let keys = file_hash_as_nostr_private_key!("test_nostr_file_for_macro.txt");

    //     assert!(!keys.public_key().to_bech32().unwrap().is_empty());

    //     remove_file(&file_path).unwrap();
    // }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_metadata_event_tr() {
        use super::publish_metadata_event;
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let picture_url = super::DEFAULT_PICTURE_URL;
        let banner_url = super::DEFAULT_BANNER_URL;
        let file_path_str = "test_file.txt";

        // This test primarily checks that the function doesn't panic
        // and goes through its execution path.
        // Actual publishing success depends on external network conditions.
        let relay_urls = super::get_relay_urls();
        publish_metadata_event(
            &keys,
            &relay_urls,
            picture_url,
            banner_url,
            file_path_str,
        ).await;
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    #[serial]
    async fn test_repository_announcement_event_tr() {
        use super::get_relay_urls;
        use nostr_sdk::{Keys, EventId};
        use std::str::FromStr;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let project_name = "test-nip34-repo";
        let description = "A test repository for NIP-34 announcements.";
        let clone_url = "git@example.com:test/test-nip34-repo.git";
        let _dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();
        let _file_for_euc = "Cargo.toml"; // Use a known file in the project, as required by include_bytes!

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_repo_announcement_picture.jpg",
            "https://example.com/test_repo_announcement_banner.jpg",
            "test_repository_announcement_event_metadata",
        ).await;

        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        repository_announcement!(
            &keys,
            &relay_urls,
            project_name,
            description,
            clone_url,
            "../Cargo.toml", // Pass the string literal directly, correcting path for include_bytes!
            Some(&dummy_build_manifest_id)
            );
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_patch_event_tr() {
        use super::{get_relay_urls, DEFAULT_PICTURE_URL, DEFAULT_BANNER_URL};
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-patch";
        let commit_id = "fedcba9876543210fedcba9876543210fedcba";

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            DEFAULT_PICTURE_URL,
            DEFAULT_BANNER_URL,
            "test_publish_patch_event_metadata",
        ).await;

        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();
        publish_patch!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            "lib.rs", // Use an existing file for the patch content
            Some(&dummy_build_manifest_id)
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_pull_request_event_tr() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-pr";
        let commit_id = "0123456789abcdef0123456789abcdef01234567";
        let clone_url = "git@example.com:test/pr-branch.git";
        let title = Some("Feat: Implement NIP-34 PR");
        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_pr_picture.jpg",
            "https://example.com/test_pr_banner.jpg",
            "test_publish_pull_request_event_metadata",
        ).await;

        // Test with a title
        publish_pull_request!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            clone_url,
            Some(title.unwrap()),
            Some(&dummy_build_manifest_id)
            );
        // Test without a title
        publish_pull_request!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            clone_url
        );
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_pr_update_event_tr() {
        use super::get_relay_urls;
        use nostr_sdk::{Keys, EventId};
        use std::str::FromStr;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-pr-update";
        let pr_event_id = EventId::from_str("f6e4d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9").unwrap(); // Placeholder EventId
        let updated_commit_id = "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0";
        let updated_clone_url = "git@example.com:test/pr-branch-updated.git";
        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_pr_update_picture.jpg",
            "https://example.com/test_pr_update_banner.jpg",
            "test_publish_pr_update_event_metadata",
        ).await;

        publish_pr_update!(
            &keys,
            &relay_urls,
            d_tag,
            &pr_event_id, // Pass a reference to pr_event_id
            updated_commit_id,
            updated_clone_url,
            Some(&dummy_build_manifest_id)
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_repository_state_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-state";
        let branch_name = "main";
        let commit_id = "abcde12345abcde12345abcde12345abcde12345";
        use nostr_sdk::EventId;
        use std::str::FromStr;
        let _dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_repo_state_picture.jpg",
            "https://example.com/test_repo_state_banner.jpg",
            "test_publish_repository_state_event_metadata",
        ).await;

        publish_repository_state!(
            &keys,
            &relay_urls,
            d_tag,
            branch_name,
            commit_id
        );    }




    // Test for get_file_hash! macro
    #[test]
    fn test_get_file_hash_tr() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        let content = "Hello, world!";
        File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

        // The macro expects a string literal, so we need to construct the path at compile time.
        // This is a limitation for testing, normally you'd use it with a known file.
        // For testing, we'll manually verify a file known to be in the project.
        // Let's test `lib.rs` itself for a more realistic scenario.
        let macro_hash = get_file_hash!("lib.rs");

        // We will assert on a known file within the crate.
        let bytes = include_bytes!("lib.rs");
        let mut hasher_manual = Sha256::new();
        hasher_manual.update(bytes);
        let expected_hash_lib_rs = hasher_manual.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        assert_eq!(macro_hash, expected_hash_lib_rs);

        // Test with another known file, e.g., Cargo.toml of the core crate
        let cargo_toml_hash = get_file_hash!("../Cargo.toml");
        let cargo_toml_bytes = include_bytes!("../Cargo.toml");
        let mut cargo_toml_hasher = Sha256::new();
        cargo_toml_hasher.update(cargo_toml_bytes);
        let expected_cargo_toml_hash = cargo_toml_hasher.finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        assert_eq!(cargo_toml_hash, expected_cargo_toml_hash);
    }

    #[test]
    fn test_get_git_tracked_files_tr() {
        let dir = tempfile::tempdir().unwrap();
        let repo_path = dir.path();

        // Initialize a git repository
        let _ = Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to initialize git repo");

        // Create some files
        let file1_path = repo_path.join("file1.txt");
        File::create(&file1_path).unwrap().write_all(b"content1").unwrap();
        let file2_path = repo_path.join("file2.txt");
        File::create(&file2_path).unwrap().write_all(b"content2").unwrap();

        // Add and commit files
        let _ = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to git add files");
        let _ = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to git commit");

        let tracked_files = get_git_tracked_files(&repo_path.to_path_buf());
        assert_eq!(tracked_files.len(), 2);
        assert!(tracked_files.contains(&"file1.txt".to_string()));
        assert!(tracked_files.contains(&"file2.txt".to_string()));
    }

    // #[cfg(feature = "nostr")]
    // #[test]
    // fn test_file_hash_as_nostr_private_key() {
    //     use super::file_hash_as_nostr_private_key;
    //     // use std::fs::{File, remove_file};
    //     // use std::io::Write;
    //     // use tempfile::tempdir; // Not needed as we're using a literal path
    //     use nostr_sdk::prelude::ToBech32;

    //     let file_path = PathBuf::from("test_nostr_file_for_macro.txt");
    //     let content = "Nostr test content!";
    //     File::create(&file_path).unwrap().write_all(content.as_bytes()).unwrap();

    //     let keys = file_hash_as_nostr_private_key!("test_nostr_file_for_macro.txt");

    //     assert!(!keys.public_key().to_bech32().unwrap().is_empty());

    //     remove_file(&file_path).unwrap();
    // }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_metadata_event() {
        use super::publish_metadata_event;
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let picture_url = super::DEFAULT_PICTURE_URL;
        let banner_url = super::DEFAULT_BANNER_URL;
        let file_path_str = "test_file.txt";

        // This test primarily checks that the function doesn't panic
        // and goes through its execution path.
        // Actual publishing success depends on external network conditions.
        let relay_urls = super::get_relay_urls();
        publish_metadata_event(
            &keys,
            &relay_urls,
            picture_url,
            banner_url,
            file_path_str,
        ).await;
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    #[serial]
    async fn test_repository_announcement_event() {
        use super::get_relay_urls;
        use nostr_sdk::{Keys, EventId};
        use std::str::FromStr;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let project_name = "test-nip34-repo";
        let description = "A test repository for NIP-34 announcements.";
        let clone_url = "git@example.com:test/test-nip34-repo.git";
        let _dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();
        let _file_for_euc = "Cargo.toml"; // Use a known file in your project, as required by include_bytes!

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_repo_announcement_picture.jpg",
            "https://example.com/test_repo_announcement_banner.jpg",
            "test_repository_announcement_event_metadata",
        ).await;

        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        repository_announcement!(
            &keys,
            &relay_urls,
            project_name,
            description,
            clone_url,
            "../Cargo.toml", // Pass the string literal directly, correcting path for include_bytes!
            Some(&dummy_build_manifest_id)
            );
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_patch_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-patch";
        let commit_id = "fedcba9876543210fedcba9876543210fedcba";

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_patch_picture.jpg",
            "https://example.com/test_patch_banner.jpg",
            "test_publish_patch_event_metadata",
        ).await;

        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();
        publish_patch!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            "lib.rs", // Use an existing file for the patch content
            Some(&dummy_build_manifest_id)
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_pull_request_event() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-pr";
        let commit_id = "0123456789abcdef0123456789abcdef01234567";
        let clone_url = "git@example.com:test/pr-branch.git";
        let title = Some("Feat: Implement NIP-34 PR");
        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_pr_picture.jpg",
            "https://example.com/test_pr_banner.jpg",
            "test_publish_pull_request_event_metadata",
        ).await;

        // Test with a title
        publish_pull_request!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            clone_url,
            Some(title.unwrap()),
            Some(&dummy_build_manifest_id)
            );
        // Test without a title
        publish_pull_request!(
            &keys,
            &relay_urls,
            d_tag,
            commit_id,
            clone_url
        );
    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_pr_update_event() {
        use super::get_relay_urls;
        use nostr_sdk::{Keys, EventId};
        use std::str::FromStr;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-pr-update";
        let pr_event_id = EventId::from_str("f6e4d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9").unwrap(); // Placeholder EventId
        let updated_commit_id = "z9y8x7w6v5u4t3s2r1q0p9o8n7m6l5k4j3i2h1g0";
        let updated_clone_url = "git@example.com:test/pr-branch-updated.git";
        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_pr_update_picture.jpg",
            "https://example.com/test_pr_update_banner.jpg",
            "test_publish_pr_update_event_metadata",
        ).await;

        publish_pr_update!(
            &keys,
            &relay_urls,
            d_tag,
            &pr_event_id, // Pass a reference to pr_event_id
            updated_commit_id,
            updated_clone_url,
            Some(&dummy_build_manifest_id)
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    #[serial]
    async fn test_publish_repository_state_event_tr() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;
        use nostr_sdk::secp256k1::SecretKey as NostrSecretKey;
        
        // 1. Generate FROST keys (1-of-1 for this test to derive a single Nostr key)
        let (shares, _pubkey_package) = super::generate_frost_keys(2, 2).unwrap();
        let signer_id = frost::Identifier::try_from(1 as u16).unwrap();
        let secret_share = shares.get(&signer_id).unwrap();

        // Convert FROST secret share's scalar to a Nostr SecretKey
        let frost_secp_secret_key = secret_share.signing_share().to_scalar();
        let nostr_secret_key = NostrSecretKey::from_slice(&frost_secp_secret_key.to_bytes()).unwrap();
        let keys = Keys::new(nostr_secret_key.into());
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-state";
        let branch_name = "main";
        let commit_id = "abcde12345abcde12345abcde12345abcde12345";
        use nostr_sdk::EventId;
        use std::str::FromStr;
        let _dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_repo_state_picture.jpg",
            "https://example.com/test_repo_state_banner.jpg",
            "test_publish_repository_state_event_metadata",
        ).await;

        publish_repository_state!(
            &keys,
            &relay_urls,
            d_tag,
            branch_name,
            commit_id
        );    }

    #[cfg(feature = "nostr")]
    #[tokio::test]
    async fn test_publish_issue_event_tr() {
        use super::get_relay_urls;
        use nostr_sdk::Keys;
        use nostr_sdk::EventId;
        use std::str::FromStr;

        let keys = Keys::parse(super::DEFAULT_GNOSTR_KEY).expect("Failed to create Nostr Keys from DEFAULT_GNOSTR_KEY");
        let relay_urls = get_relay_urls();
        let d_tag = "test-repo-for-issue";
        let issue_id = "456";
        let title = "Feature: Implement NIP-34 Issues";
        let content = "This is a test issue to verify the NIP-34 issue macro implementation.";
        let dummy_build_manifest_id = EventId::from_str(super::DUMMY_BUILD_MANIFEST_ID_STR).unwrap();

        // This test primarily checks that the macro and function compile and execute without panicking.
        // Actual publishing success depends on external network conditions.
        super::publish_metadata_event(
            &keys,
            &relay_urls,
            "https://example.com/test_issue_picture.jpg",
            "https://example.com/test_issue_banner.jpg",
            "test_publish_issue_event_metadata",
        ).await;

        publish_issue!(
            &keys,
            &relay_urls,
            d_tag,
            issue_id,
            title,
            content,
            Some(&dummy_build_manifest_id)
        );
    }

    #[cfg(feature = "nostr")]
    #[test]
    fn test_frost_signature_flow_tr() {
        let max_signers = 3;
        let min_signers = 2;
        let message = b"This is a test message for FROST signing";

        // 1. Key Generation
        let (shares, pubkey_package) = super::generate_frost_keys(max_signers, min_signers).unwrap();

        let mut commitments = BTreeMap::new();
        let mut nonces_map = BTreeMap::new();
        let mut signature_shares_map = BTreeMap::new();

        // 2. Commitment Phase (simulated for two signers)
        let signer1_id = frost::Identifier::try_from(1 as u16).unwrap();
        let (nonces1, comms1) = super::create_frost_commitment(&shares[&signer1_id]);
        commitments.insert(signer1_id, comms1);
        nonces_map.insert(signer1_id, nonces1);

        let signer2_id = frost::Identifier::try_from(2 as u16).unwrap();
        let (nonces2, comms2) = super::create_frost_commitment(&shares[&signer2_id]);
        commitments.insert(signer2_id, comms2);
        nonces_map.insert(signer2_id, nonces2);

        // 3. Signing Package Creation
        let signing_package = super::create_signing_package(commitments, message);

        // 4. Signature Share Generation
        let share1 = super::generate_signature_share(&signing_package, &nonces_map[&signer1_id], &shares[&signer1_id]).unwrap();
        signature_shares_map.insert(signer1_id, share1);

        let share2 = super::generate_signature_share(&signing_package, &nonces_map[&signer2_id], &shares[&signer2_id]).unwrap();
        signature_shares_map.insert(signer2_id, share2);

        // 5. Aggregation
        let group_signature = super::aggregate_signature_shares(&signing_package, &signature_shares_map, &pubkey_package).unwrap();

        // 6. Verification
        let group_public_key = pubkey_package.verifying_key();
        super::verify_frost_signature(&group_public_key, message, &group_signature).unwrap();
    }
}
