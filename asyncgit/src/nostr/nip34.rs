/// NIP-34 Git Stuff — types, builders and parsers.
///
/// Implements the event kinds defined in NIP-34:
/// - 30617  Repository Announcement
/// - 30618  Repository State
/// - 1617   Patch
/// - 1618   Pull Request
/// - 1621   Issue
/// - 1630–1633  Status (Open / Applied / Closed / Draft)
use std::time::{SystemTime, UNIX_EPOCH};

use secp256k1::{Keypair, XOnlyPublicKey, SECP256K1};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

use super::client::NostrEvent;

// ── event-kind constants ─────────────────────────────────────────────────────

/// Kind 30617 – Repository Announcement.
pub const KIND_REPO_ANNOUNCEMENT: u64 = 30617;
/// Kind 30618 – Repository State (branch/tag refs).
pub const KIND_REPO_STATE: u64 = 30618;
/// Kind 1617 – Patch.
pub const KIND_PATCH: u64 = 1617;
/// Kind 1618 – Pull Request.
pub const KIND_PULL_REQUEST: u64 = 1618;
/// Kind 1621 – Issue.
pub const KIND_ISSUE: u64 = 1621;
/// Kind 1622 – Reply (comment on patch/issue).
pub const KIND_REPLY: u64 = 1622;
/// Kind 1630 – Status: Open.
pub const KIND_STATUS_OPEN: u64 = 1630;
/// Kind 1631 – Status: Applied/Merged for patches; Resolved for issues.
pub const KIND_STATUS_APPLIED: u64 = 1631;
/// Kind 1632 – Status: Closed.
pub const KIND_STATUS_CLOSED: u64 = 1632;
/// Kind 1633 – Status: Draft.
pub const KIND_STATUS_DRAFT: u64 = 1633;

// ── parsed structs ────────────────────────────────────────────────────────────

/// A NIP-34 patch (kind 1617).
#[derive(Clone, Debug)]
pub struct GitPatch {
	/// Nostr event id (hex).
	pub id: String,
	/// Author pubkey (hex).
	pub pubkey: String,
	/// Unix timestamp.
	pub created_at: u64,
	/// `git format-patch` content.
	pub content: String,
	/// `a` tag pointing to the repository (30617:<pubkey>:<repo-id>).
	pub repo_a_tag: String,
	/// Optional subject extracted from the patch content or `subject` tag.
	pub subject: String,
	/// Commit id referenced by the patch.
	pub commit: Option<String>,
	/// Current status (default Open).
	pub status: PatchStatus,
}

/// A NIP-34 issue (kind 1621).
#[derive(Clone, Debug)]
pub struct GitIssue {
	/// Nostr event id (hex).
	pub id: String,
	/// Author pubkey (hex).
	pub pubkey: String,
	/// Unix timestamp.
	pub created_at: u64,
	/// Markdown body.
	pub content: String,
	/// `a` tag pointing to the repository.
	pub repo_a_tag: String,
	/// Issue subject / title.
	pub subject: String,
	/// Labels from `t` tags.
	pub labels: Vec<String>,
	/// Current status (default Open).
	pub status: PatchStatus,
}

/// A NIP-34 repository announcement (kind 30617).
#[derive(Clone, Debug)]
pub struct GitRepoAnnouncement {
	/// Nostr event id (hex).
	pub id: String,
	/// Author pubkey (hex).
	pub pubkey: String,
	/// Short repository identifier (kebab-case).
	pub repo_id: String,
	/// Human-readable name.
	pub name: String,
	/// Brief description.
	pub description: String,
	/// Clone URLs.
	pub clone_urls: Vec<String>,
	/// Web URLs.
	pub web_urls: Vec<String>,
	/// Relay URLs this repo monitors.
	pub relays: Vec<String>,
	/// Earliest unique commit id.
	pub earliest_unique_commit: Option<String>,
}

/// Status of a patch or issue.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum PatchStatus {
	/// Open (default).
	#[default]
	Open,
	/// Applied / Merged / Resolved.
	Applied,
	/// Closed.
	Closed,
	/// Draft.
	Draft,
}

impl PatchStatus {
	/// Display label for the status.
	pub fn label(&self) -> &'static str {
		match self {
			Self::Open => "open",
			Self::Applied => "merged",
			Self::Closed => "closed",
			Self::Draft => "draft",
		}
	}

	/// Event kind number for this status.
	pub fn kind(&self) -> u64 {
		match self {
			Self::Open => KIND_STATUS_OPEN,
			Self::Applied => KIND_STATUS_APPLIED,
			Self::Closed => KIND_STATUS_CLOSED,
			Self::Draft => KIND_STATUS_DRAFT,
		}
	}

	/// Parse from a kind number.
	pub fn from_kind(kind: u64) -> Option<Self> {
		match kind {
			KIND_STATUS_OPEN => Some(Self::Open),
			KIND_STATUS_APPLIED => Some(Self::Applied),
			KIND_STATUS_CLOSED => Some(Self::Closed),
			KIND_STATUS_DRAFT => Some(Self::Draft),
			_ => None,
		}
	}
}

// ── builders ──────────────────────────────────────────────────────────────────

/// Build a NIP-34 repository announcement event (kind 30617).
pub fn build_repo_announcement(
	repo_id: &str,
	name: &str,
	description: &str,
	clone_urls: &[String],
	web_urls: &[String],
	relay_urls: &[String],
	earliest_commit: Option<&str>,
	kp: &Keypair,
) -> Result<NostrEvent> {
	let mut tags: Vec<Vec<String>> = vec![
		vec!["d".into(), repo_id.into()],
		vec!["name".into(), name.into()],
		vec!["description".into(), description.into()],
	];

	if !clone_urls.is_empty() {
		let mut t = vec!["clone".into()];
		t.extend(clone_urls.iter().cloned());
		tags.push(t);
	}

	if !web_urls.is_empty() {
		let mut t = vec!["web".into()];
		t.extend(web_urls.iter().cloned());
		tags.push(t);
	}

	if !relay_urls.is_empty() {
		let mut t = vec!["relays".into()];
		t.extend(relay_urls.iter().cloned());
		tags.push(t);
	}

	if let Some(euc) = earliest_commit {
		tags.push(vec!["r".into(), euc.into(), "euc".into()]);
	}

	sign_event(KIND_REPO_ANNOUNCEMENT, String::new(), tags, kp)
}

/// Build a NIP-34 patch event (kind 1617) from raw `git format-patch` output.
///
/// `repo_a_tag` is the full `a`-tag value: `30617:<maintainer-pubkey>:<repo-id>`.
/// `commit_id` and `parent_commit_id` are optional but help clients track commits.
pub fn build_patch(
	patch_content: &str,
	repo_a_tag: &str,
	maintainer_pubkey_hex: &str,
	commit_id: Option<&str>,
	parent_commit_id: Option<&str>,
	is_root: bool,
	kp: &Keypair,
) -> Result<NostrEvent> {
	let mut tags: Vec<Vec<String>> = vec![
		vec!["a".into(), repo_a_tag.into()],
		vec!["p".into(), maintainer_pubkey_hex.into()],
	];

	if is_root {
		tags.push(vec!["t".into(), "root".into()]);
	}

	if let Some(cid) = commit_id {
		tags.push(vec!["commit".into(), cid.into()]);
		tags.push(vec!["r".into(), cid.into()]);
	}

	if let Some(pid) = parent_commit_id {
		tags.push(vec!["parent-commit".into(), pid.into()]);
	}

	sign_event(KIND_PATCH, patch_content.to_owned(), tags, kp)
}

/// Build a NIP-34 issue event (kind 1621).
///
/// `repo_a_tag` is the full `a`-tag value: `30617:<maintainer-pubkey>:<repo-id>`.
pub fn build_issue(
	subject: &str,
	body: &str,
	repo_a_tag: &str,
	maintainer_pubkey_hex: &str,
	labels: &[String],
	kp: &Keypair,
) -> Result<NostrEvent> {
	let mut tags: Vec<Vec<String>> = vec![
		vec!["a".into(), repo_a_tag.into()],
		vec!["p".into(), maintainer_pubkey_hex.into()],
		vec!["subject".into(), subject.into()],
	];

	for label in labels {
		tags.push(vec!["t".into(), label.clone()]);
	}

	sign_event(KIND_ISSUE, body.to_owned(), tags, kp)
}

/// Build a NIP-34 status event.
///
/// `status` determines the event kind (1630–1633).
/// `target_event_id` is the hex id of the root patch or issue being updated.
pub fn build_status(
	status: PatchStatus,
	target_event_id: &str,
	target_author_pubkey: &str,
	repo_a_tag: &str,
	comment: &str,
	kp: &Keypair,
) -> Result<NostrEvent> {
	let kind = status.kind();
	let tags: Vec<Vec<String>> = vec![
		vec![
			"e".into(),
			target_event_id.into(),
			String::new(),
			"root".into(),
		],
		vec!["p".into(), target_author_pubkey.into()],
		vec!["a".into(), repo_a_tag.into()],
	];

	sign_event(kind, comment.to_owned(), tags, kp)
}

/// Build a subscription filter for NIP-34 events referencing a repository.
///
/// `repo_a_tag` is `30617:<maintainer-pubkey>:<repo-id>`.
/// Returns the JSON filter object to embed in a `["REQ", sub_id, <filter>]` message.
pub fn repo_filter(repo_a_tag: &str) -> serde_json::Value {
	json!({
		"kinds": [
			KIND_PATCH,
			KIND_PULL_REQUEST,
			KIND_ISSUE,
			KIND_REPLY,
			KIND_STATUS_OPEN,
			KIND_STATUS_APPLIED,
			KIND_STATUS_CLOSED,
			KIND_STATUS_DRAFT,
		],
		"#a": [repo_a_tag],
		"limit": 200
	})
}

/// Build a subscription filter for repository announcements by a given pubkey.
pub fn announcement_filter(pubkey_hex: &str) -> serde_json::Value {
	json!({
		"kinds": [KIND_REPO_ANNOUNCEMENT],
		"authors": [pubkey_hex],
		"limit": 50
	})
}

/// Build a broad subscription filter for ALL NIP-34 event kinds.
///
/// Used to populate the Nostr tab on first connect without knowing which
/// specific repository the user wants to watch.
pub fn all_nip34_filter() -> serde_json::Value {
	json!({
		"kinds": [
			KIND_REPO_ANNOUNCEMENT,
			KIND_REPO_STATE,
			KIND_PATCH,
			KIND_PULL_REQUEST,
			KIND_ISSUE,
			KIND_REPLY,
			KIND_STATUS_OPEN,
			KIND_STATUS_APPLIED,
			KIND_STATUS_CLOSED,
			KIND_STATUS_DRAFT,
		],
		"limit": 200
	})
}

// ── parsers ───────────────────────────────────────────────────────────────────

/// Try to parse a [`NostrEvent`] as a [`GitPatch`].
pub fn parse_patch(ev: &NostrEvent) -> Option<GitPatch> {
	if ev.kind != KIND_PATCH {
		return None;
	}
	let repo_a_tag = first_tag_value(&ev.tags, "a")?;
	let subject = first_tag_value(&ev.tags, "subject")
		.unwrap_or_else(|| subject_from_patch(&ev.content));
	let commit = first_tag_value(&ev.tags, "commit");

	Some(GitPatch {
		id: ev.id.clone(),
		pubkey: ev.pubkey.clone(),
		created_at: ev.created_at,
		content: ev.content.clone(),
		repo_a_tag,
		subject,
		commit,
		status: PatchStatus::Open,
	})
}

/// Try to parse a [`NostrEvent`] as a [`GitIssue`].
pub fn parse_issue(ev: &NostrEvent) -> Option<GitIssue> {
	if ev.kind != KIND_ISSUE {
		return None;
	}
	let repo_a_tag = first_tag_value(&ev.tags, "a")?;
	let subject = first_tag_value(&ev.tags, "subject")
		.unwrap_or_else(|| "(no subject)".into());
	let labels: Vec<String> = ev
		.tags
		.iter()
		.filter(|t| t.first().map(|s| s == "t").unwrap_or(false))
		.filter_map(|t| t.get(1).cloned())
		.collect();

	Some(GitIssue {
		id: ev.id.clone(),
		pubkey: ev.pubkey.clone(),
		created_at: ev.created_at,
		content: ev.content.clone(),
		repo_a_tag,
		subject,
		labels,
		status: PatchStatus::Open,
	})
}

/// Try to parse a [`NostrEvent`] as a [`GitRepoAnnouncement`].
pub fn parse_repo_announcement(ev: &NostrEvent) -> Option<GitRepoAnnouncement> {
	if ev.kind != KIND_REPO_ANNOUNCEMENT {
		return None;
	}
	let repo_id = first_tag_value(&ev.tags, "d")?;
	let name =
		first_tag_value(&ev.tags, "name").unwrap_or_else(|| repo_id.clone());
	let description =
		first_tag_value(&ev.tags, "description").unwrap_or_default();
	let clone_urls = multi_tag_values(&ev.tags, "clone");
	let web_urls = multi_tag_values(&ev.tags, "web");
	let relays = multi_tag_values(&ev.tags, "relays");
	let earliest_unique_commit = ev
		.tags
		.iter()
		.find(|t| {
			t.first().map(|s| s == "r").unwrap_or(false)
				&& t.get(2).map(|s| s == "euc").unwrap_or(false)
		})
		.and_then(|t| t.get(1).cloned());

	Some(GitRepoAnnouncement {
		id: ev.id.clone(),
		pubkey: ev.pubkey.clone(),
		repo_id,
		name,
		description,
		clone_urls,
		web_urls,
		relays,
		earliest_unique_commit,
	})
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Return the first value of a tag with the given name.
fn first_tag_value(tags: &[Vec<String>], name: &str) -> Option<String> {
	tags.iter()
		.find(|t| t.first().map(|s| s == name).unwrap_or(false))
		.and_then(|t| t.get(1).cloned())
}

/// Return all values (position 1+) of all tags with the given name.
fn multi_tag_values(tags: &[Vec<String>], name: &str) -> Vec<String> {
	tags.iter()
		.filter(|t| t.first().map(|s| s == name).unwrap_or(false))
		.flat_map(|t| t.iter().skip(1).cloned())
		.collect()
}

/// Extract a subject from the first line of `git format-patch` output.
///
/// Format-patch subjects look like: `Subject: [PATCH n/N] <message>`
fn subject_from_patch(content: &str) -> String {
	for line in content.lines() {
		if let Some(rest) = line.strip_prefix("Subject: ") {
			// Strip [PATCH ...] prefix if present
			if let Some(bracket_end) = rest.find("] ") {
				return rest[bracket_end + 2..].trim().to_owned();
			}
			return rest.trim().to_owned();
		}
	}
	"(no subject)".into()
}

/// Sign an event using our secp256k1 keypair and return the complete `NostrEvent`.
fn sign_event(
	kind: u64,
	content: String,
	tags: Vec<Vec<String>>,
	kp: &Keypair,
) -> Result<NostrEvent> {
	let (xonly, _) = XOnlyPublicKey::from_keypair(kp);
	let pubkey = hex::encode(xonly.serialize());
	let created_at = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap_or_default()
		.as_secs();

	let id = compute_id(&pubkey, created_at, kind, &tags, &content);

	let msg_bytes: [u8; 32] = {
		let mut a = [0u8; 32];
		let decoded = hex::decode(&id)
			.map_err(|e| Error::Generic(format!("hex decode id: {e}")))?;
		let len = decoded.len().min(32);
		a[..len].copy_from_slice(&decoded[..len]);
		a
	};
	let msg = secp256k1::Message::from_digest(msg_bytes);
	let sig = SECP256K1.sign_schnorr_no_aux_rand(&msg, kp);

	Ok(NostrEvent {
		id,
		pubkey,
		created_at,
		kind,
		tags,
		content,
		sig: hex::encode(sig.serialize()),
	})
}

fn compute_id(
	pubkey: &str,
	created_at: u64,
	kind: u64,
	tags: &[Vec<String>],
	content: &str,
) -> String {
	let serialised =
		json!([0, pubkey, created_at, kind, tags, content]).to_string();
	let mut h = Sha256::new();
	h.update(serialised.as_bytes());
	hex::encode(h.finalize())
}
