//! Shared types for Nostr tab and components

#[cfg(feature = "nostr")]
use asyncgit::nostr::{GitIssue, GitPatch, GitRepoAnnouncement};
#[cfg(feature = "nostr")]
#[derive(Clone, Debug)]
pub struct IndexedNostrItem {
    pub idx: usize,
    pub item: NostrItem,
}

#[derive(Clone, Debug)]
pub enum NostrItem {
	Patch(GitPatch),
	Issue(GitIssue),
	Announcement(GitRepoAnnouncement),
}

#[cfg(feature = "nostr")]
impl NostrItem {
	pub fn subject(&self) -> &str {
		match self {
			Self::Patch(p) => &p.subject,
			Self::Issue(i) => &i.subject,
			Self::Announcement(a) => &a.name,
		}
	}
	pub fn pubkey_short(&self) -> String {
		let pk = match self {
			Self::Patch(p) => &p.pubkey,
			Self::Issue(i) => &i.pubkey,
			Self::Announcement(a) => &a.pubkey,
		};
		if pk.len() >= 8 {
			format!("{}…{}", &pk[..4], &pk[pk.len() - 4..])
		} else {
			pk.clone()
		}
	}
	pub fn kind_label(&self) -> &'static str {
		match self {
			Self::Patch(_) => "patch",
			Self::Issue(_) => "issue",
			Self::Announcement(_) => "repo",
		}
	}
	pub fn status_label(&self) -> &'static str {
		match self {
			Self::Patch(p) => p.status.label(),
			Self::Issue(i) => i.status.label(),
			Self::Announcement(_) => "",
		}
	}
	pub fn id(&self) -> &str {
		match self {
			Self::Patch(p) => &p.id,
			Self::Issue(i) => &i.id,
			Self::Announcement(a) => &a.id,
		}
	}
	pub fn content(&self) -> &str {
		match self {
			Self::Patch(p) => &p.content,
			Self::Issue(i) => &i.content,
			Self::Announcement(a) => &a.description,
		}
	}
	pub fn created_at(&self) -> u64 {
		match self {
			Self::Patch(p) => p.created_at,
			Self::Issue(i) => i.created_at,
			Self::Announcement(_) => 0,
		}
	}
}
