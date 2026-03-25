//! NIP-94: File Metadata
//!
//! This NIP defines how to represent file metadata using event kind 1063.
//! These events allow users to share and describe files on Nostr.
//!
//! https://github.com/nostr-protocol/nips/blob/master/94.md

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Event, EventKind, Id, PreEvent, PublicKey, Signature, Tag, Unixtime};

/// NIP-94 File Metadata Event Kind (Regular Event)
pub const FILE_METADATA_KIND: u32 = 1063;

/// Represents the content of a NIP-94 File Metadata Event (Kind 1063).
/// The content typically holds a description of the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadataContent {
    pub description: String,
}

/// Helper trait for NIP-94 events.
pub trait NIP94Event {
    /// Extracts the main URL of the file from the 'url' tag.
    fn file_url(&self) -> Option<&str>;
    /// Extracts the MIME type from the 'm' tag.
    fn mime_type(&self) -> Option<&str>;
    /// Extracts the SHA-256 hash of the transformed file from the 'x' tag.
    fn sha256_hash(&self) -> Option<&str>;
    /// Extracts the SHA-256 hash of the original file from the 'ox' tag.
    fn original_sha256_hash(&self) -> Option<&str>;
    /// Extracts the file size from the 'size' tag.
    fn file_size(&self) -> Option<usize>;
    /// Extracts dimensions from the 'dim' tag (e.g., "1920x1080").
    fn dimensions(&self) -> Option<&str>;
    /// Extracts blurhash from the 'blurhash' tag.
    fn blurhash(&self) -> Option<&str>;

    /// Creates a NIP-94 File Metadata event builder.
    fn new_file_metadata(
        public_key: PublicKey,
        description: String,
        url: String,
        mime_type: String,
        sha256_hash: String,
        original_sha256_hash: Option<String>,
        file_size: Option<usize>,
        dimensions: Option<String>,
        blurhash: Option<String>,
        // Add other relevant NIP-94 tags as needed
    ) -> Result<Event>;
}

impl NIP94Event for Event {
    fn file_url(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "url" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn mime_type(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "m" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn sha256_hash(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "x" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn original_sha256_hash(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "ox" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn file_size(&self) -> Option<usize> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "size" {
                tag.0[1].parse::<usize>().ok()
            } else {
                None
            }
        })
    }

    fn dimensions(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "dim" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn blurhash(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "blurhash" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn new_file_metadata(
        public_key: PublicKey,
        description: String,
        url: String,
        mime_type: String,
        sha256_hash: String,
        original_sha256_hash: Option<String>,
        file_size: Option<usize>,
        dimensions: Option<String>,
        blurhash: Option<String>,
    ) -> Result<Event> {
        let content = FileMetadataContent { description }.description;
        let mut tags: Vec<Tag> = vec![
            Tag::new(&["url", &url]),
            Tag::new(&["m", &mime_type]),
            Tag::new(&["x", &sha256_hash]),
        ];

        if let Some(ox) = original_sha256_hash {
            tags.push(Tag::new(&["ox", &ox]));
        }
        if let Some(size) = file_size {
            tags.push(Tag::new(&["size", &size.to_string()]));
        }
        if let Some(dim) = dimensions {
            tags.push(Tag::new(&["dim", &dim]));
        }
        if let Some(bh) = blurhash {
            tags.push(Tag::new(&["blurhash", &bh]));
        }

        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::FileMetadata, // NIP-94 event is directly FileMetadata enum variant
            tags: tags.clone(),
            content: content.clone(),
        };

        let id = pre_event.hash().unwrap();

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::FileMetadata,
            tags,
            content,
            sig: Signature::zeroes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Id, PublicKey, Signature};

    // Helper to create a dummy event for testing
    fn create_dummy_event(
        url: &str,
        mime: &str,
        x_hash: &str,
        ox_hash: Option<&str>,
        size: Option<usize>,
        dim: Option<&str>,
        blur: Option<&str>,
        description: &str,
    ) -> Event {
        let public_key = PublicKey::mock();
        Event::new_file_metadata(
            public_key,
            description.to_string(),
            url.to_string(),
            mime.to_string(),
            x_hash.to_string(),
            ox_hash.map(|s| s.to_string()),
            size,
            dim.map(|s| s.to_string()),
            blur.map(|s| s.to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_file_url() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.file_url(), Some("http://example.com/file.jpg"));
    }

    #[test]
    fn test_mime_type() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.mime_type(), Some("image/jpeg"));
    }

    #[test]
    fn test_sha256_hash() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.sha256_hash(), Some("hash_x"));
    }

    #[test]
    fn test_original_sha256_hash() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            Some("hash_ox"),
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.original_sha256_hash(), Some("hash_ox"));
    }

    #[test]
    fn test_file_size() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            Some(1024),
            None,
            None,
            "A file",
        );
        assert_eq!(event.file_size(), Some(1024));
    }

    #[test]
    fn test_dimensions() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            Some("1920x1080"),
            None,
            "A file",
        );
        assert_eq!(event.dimensions(), Some("1920x1080"));
    }

    #[test]
    fn test_blurhash() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            Some("LGE.g9of~qof_3jYRPofM_jsfjeY"),
            "A file",
        );
        assert_eq!(event.blurhash(), Some("LGE.g9of~qof_3jYRPofM_jsfjeY"));
    }

    #[test]
    fn test_new_file_metadata_event() {
        let event = Event::new_file_metadata(
            PublicKey::mock(),
            "My cool image".to_string(),
            "http://example.com/cool.png".to_string(),
            "image/png".to_string(),
            "hash_of_transformed_file".to_string(),
            Some("hash_of_original_file".to_string()),
            Some(2048),
            Some("800x600".to_string()),
            Some("blurhash_string".to_string()),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::FileMetadata);
        assert_eq!(event.content, "My cool image");
        assert_eq!(event.file_url(), Some("http://example.com/cool.png"));
        assert_eq!(event.mime_type(), Some("image/png"));
        assert_eq!(event.sha256_hash(), Some("hash_of_transformed_file"));
        assert_eq!(event.original_sha256_hash(), Some("hash_of_original_file"));
        assert_eq!(event.file_size(), Some(2048));
        assert_eq!(event.dimensions(), Some("800x600"));
        assert_eq!(event.blurhash(), Some("blurhash_string"));
    }
}
