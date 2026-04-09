use git2::{Email, EmailCreateOptions};

use super::{repository::repo, CommitId, RepoPath};
use crate::error::{Error, Result};

/// Generate `git format-patch`-style email content for a single commit.
///
/// Returns the raw mbox-formatted patch string suitable for embedding in a
/// NIP-34 patch event (kind 1617).
pub fn commit_to_format_patch(
	repo_path: &RepoPath,
	commit_id: CommitId,
) -> Result<String> {
	let r = repo(repo_path)?;
	let oid: git2::Oid = commit_id.into();
	let commit = r.find_commit(oid)?;
	let mut opts = EmailCreateOptions::new();
	let email = Email::from_commit(&commit, &mut opts)
		.map_err(|e| Error::Generic(format!("format-patch: {e}")))?;
	String::from_utf8(email.as_slice().to_vec())
		.map_err(|e| Error::Generic(format!("format-patch utf8: {e}")))
}
