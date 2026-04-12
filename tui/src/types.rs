//! Plain data types shared across the blossom-tui crate.

use std::path::PathBuf;

use blossom_rs::BlobDescriptor;

// ── Sort/Filter ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortField {
    #[default]
    Date,
    Size,
    Hash,
    ContentType,
}

impl SortField {
    pub fn next(self) -> Self {
        match self {
            Self::Date => Self::Size,
            Self::Size => Self::Hash,
            Self::Hash => Self::ContentType,
            Self::ContentType => Self::Date,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Date => "Date",
            Self::Size => "Size",
            Self::Hash => "Hash",
            Self::ContentType => "Type",
        }
    }
}

// ── Modal ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    /// Prompt for local save path to download selected blob.
    Download { sha256: String },
    /// Prompt for remote URL to mirror onto the server.
    Mirror,
}

// ── Async messages ─────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AppMsg {
    BlobsLoaded(Vec<BlobDescriptor>),
    BlobsError(String),
    UploadDone(BlobDescriptor),
    UploadError(String),
    StatusLoaded(serde_json::Value),
    StatusError(String),
    DeleteDone(String),
    DeleteError(String),
    DownloadDone(PathBuf),
    DownloadError(String),
    MirrorDone(BlobDescriptor),
    MirrorError(String),
    BatchItemDone(usize, BlobDescriptor),
    BatchItemError(usize, String),
    AdminStatsLoaded(serde_json::Value),
    AdminStatsError(String),
    AdminUsersLoaded(serde_json::Value),
    AdminUsersError(String),
    RelayPolicyLoaded(serde_json::Value),
    RelayPolicyError(String),
    Nip96InfoLoaded(serde_json::Value),
    Nip96InfoError(String),
    Nip96FilesLoaded(serde_json::Value),
    Nip96FilesError(String),
    Nip94Published(String),
    Nip94PublishError(String),
    Nip34EventReceived(Nip34EventItem),
    Nip34Connected(String),
    Nip34Error(String),
    GitDone(String),
    GitError(String),
}

// ── NIP-34 ─────────────────────────────────────────────────────────────────

/// A single NIP-34 event received from a relay.
#[derive(Debug, Clone)]
pub struct Nip34EventItem {
    pub kind: u64,
    pub id: String,
    pub pubkey: String,
    pub created_at: u64,
    /// First 80 chars of content or d-tag.
    pub content_preview: String,
}

impl Nip34EventItem {
    pub fn kind_name(&self) -> &'static str {
        match self.kind {
            30617 => "RepoAnnounce",
            30618 => "RepoState",
            1617 => "Patch",
            1618 => "PullRequest",
            1619 => "PRUpdate",
            1621 => "Issue",
            1630 => "Status:Open",
            1631 => "Status:Applied",
            1632 => "Status:Closed",
            1633 => "Status:Draft",
            10317 => "GraspList",
            _ => "Unknown",
        }
    }
}

// ── Batch upload ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum BatchStatus {
    Pending,
    Running,
    Done(BlobDescriptor),
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct BatchItem {
    pub path: String,
    pub status: BatchStatus,
}

// ── Keygen ─────────────────────────────────────────────────────────────────

pub struct KeygenResult {
    pub hex_secret: String,
    pub nsec: String,
    pub pubkey: String,
    pub npub: String,
}
