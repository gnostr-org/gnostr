/// Nostr key management for gnostr-tui.
///
/// Handles generating, parsing, loading and storing nostr keys,
/// following the same pattern as asyncgit's sync layer.
/// Ported from nostr/src/infrastructure/config.rs and nostr/src/main.rs.
use std::path::Path;

use nostr_sdk::prelude::*;
use secrecy::{ExposeSecret, SecretString};

use crate::error::{Error, Result};

/// Nostr identity: a full keypair or a read-only public key (npub).
#[derive(Clone)]
pub enum NostrIdentity {
	/// Full keypair — can sign and publish events.
	Keypair(Keys),
	/// Read-only — public key only (npub / hex pubkey).
	ReadOnly(PublicKey),
}

impl NostrIdentity {
	pub fn public_key(&self) -> PublicKey {
		match self {
			Self::Keypair(keys) => keys.public_key(),
			Self::ReadOnly(pk) => *pk,
		}
	}

	pub fn can_sign(&self) -> bool {
		matches!(self, Self::Keypair(_))
	}

	/// bech32-encoded public key (npub1…)
	pub fn npub(&self) -> String {
		self.public_key()
			.to_bech32()
			.unwrap_or_else(|_| "<invalid>".to_owned())
	}

	/// Shortened display: first 8 + last 8 chars of the npub.
	pub fn short_npub(&self) -> String {
		let npub = self.npub();
		if npub.len() > 20 {
			format!("{}…{}", &npub[..8], &npub[npub.len() - 8..])
		} else {
			npub
		}
	}
}

impl std::fmt::Debug for NostrIdentity {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Keypair(_) => write!(f, "NostrIdentity::Keypair(<redacted>)"),
			Self::ReadOnly(pk) => write!(f, "NostrIdentity::ReadOnly({pk})"),
		}
	}
}

/// Parse a key string — nsec, hex private key, npub, or hex public key —
/// into a `NostrIdentity`.  Ported from nostr/src/main.rs.
pub fn parse_key(key_str: &str) -> Result<NostrIdentity> {
	let s = key_str.trim();
	if s.starts_with("npub") {
		let pk = PublicKey::parse(s)
			.map_err(|e| Error::Generic(format!("invalid npub: {e}")))?;
		Ok(NostrIdentity::ReadOnly(pk))
	} else {
		let keys = Keys::parse(s).map_err(|e| {
			Error::Generic(format!(
				"invalid key (expected nsec or hex private key): {e}"
			))
		})?;
		Ok(NostrIdentity::Keypair(keys))
	}
}

/// Generate a fresh nostr keypair.
pub fn generate_keys() -> Keys {
	Keys::generate()
}

/// Load the raw key string from git config (`nostr.key`) or the
/// `NOSTR_KEY` environment variable.
pub fn load_key_from_git_config(repo_path: &Path) -> Result<SecretString> {
	for config_key in &["nostr.key", "nostr.privatekey"] {
		let out = std::process::Command::new("git")
			.args([
				"-C",
				repo_path.to_str().unwrap_or("."),
				"config",
				config_key,
			])
			.output()
			.map_err(|e| {
				Error::Generic(format!("git config failed: {e}"))
			})?;

		if out.status.success() {
			let s = String::from_utf8_lossy(&out.stdout)
				.trim()
				.to_string();
			if !s.is_empty() {
				return Ok(SecretString::from(s));
			}
		}
	}

	if let Ok(val) = std::env::var("NOSTR_KEY") {
		if !val.is_empty() {
			return Ok(SecretString::from(val));
		}
	}

	Err(Error::Generic(
		"no nostr key found in git config (nostr.key) or NOSTR_KEY env var"
			.to_owned(),
	))
}

/// Persist a key to the local git repo config (`nostr.key`).
pub fn save_key_to_git_config(
	repo_path: &Path,
	key: &SecretString,
) -> Result<()> {
	let status = std::process::Command::new("git")
		.args([
			"-C",
			repo_path.to_str().unwrap_or("."),
			"config",
			"--local",
			"nostr.key",
			key.expose_secret(),
		])
		.status()
		.map_err(|e| Error::Generic(format!("git config failed: {e}")))?;

	if !status.success() {
		return Err(Error::Generic(
			"git config exited with non-zero status".to_owned(),
		));
	}
	Ok(())
}

/// Attempt to load and parse a `NostrIdentity` from git config.
/// Returns `None` if no key is configured.
pub fn load_identity(repo_path: &Path) -> Option<NostrIdentity> {
	let secret = load_key_from_git_config(repo_path).ok()?;
	parse_key(secret.expose_secret()).ok()
}
