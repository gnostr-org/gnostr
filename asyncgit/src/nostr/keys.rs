/// Nostr key management for gnostr-tui.
///
/// Handles generating, parsing, loading and storing nostr keys using
/// secp256k1 + bech32 directly, with no dependency on nostr-sdk.
use std::path::Path;

use bech32::{Bech32, Hrp};
use secp256k1::{Keypair, Secp256k1, SecretKey, XOnlyPublicKey, SECP256K1};

use crate::error::{Error, Result};

const HRP_PUBLIC_KEY: Hrp = Hrp::parse_unchecked("npub");

// ── public types ─────────────────────────────────────────────────────────────

/// Nostr identity: a full keypair or a read-only public key (npub).
#[derive(Clone)]
pub enum NostrIdentity {
	/// Full keypair — can sign and publish events.
	Keypair(Keypair),
	/// Read-only — x-only public key (npub / hex pubkey).
	ReadOnly(XOnlyPublicKey),
}

impl NostrIdentity {
	/// Returns the x-only public key for this identity.
	pub fn public_key(&self) -> XOnlyPublicKey {
		match self {
			Self::Keypair(kp) => XOnlyPublicKey::from_keypair(kp).0,
			Self::ReadOnly(pk) => *pk,
		}
	}

	/// Returns `true` if this identity holds a private key and can sign events.
	pub fn can_sign(&self) -> bool {
		matches!(self, Self::Keypair(_))
	}

	/// bech32-encoded public key (npub1…)
	pub fn npub(&self) -> String {
		to_bech32(HRP_PUBLIC_KEY, &self.public_key().serialize())
			.unwrap_or_else(|_| "<invalid>".to_owned())
	}

	/// Shortened display: first 8 chars + "…" + last 8 chars.
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
			Self::Keypair(_) => {
				write!(f, "NostrIdentity::Keypair(<redacted>)")
			}
			Self::ReadOnly(pk) => {
				write!(f, "NostrIdentity::ReadOnly({})", hex::encode(pk.serialize()))
			}
		}
	}
}

// ── parsing ───────────────────────────────────────────────────────────────────

/// Parse a key string — nsec, hex private key, npub, or hex public key —
/// into a `NostrIdentity`.
pub fn parse_key(key_str: &str) -> Result<NostrIdentity> {
	let s = key_str.trim();

	if s.starts_with("nsec") {
		let bytes = from_bech32(s)?;
		let sk = SecretKey::from_slice(&bytes)
			.map_err(|e| Error::Generic(format!("invalid nsec: {e}")))?;
		let kp = Keypair::from_secret_key(SECP256K1, &sk);
		return Ok(NostrIdentity::Keypair(kp));
	}

	if s.starts_with("npub") {
		let bytes = from_bech32(s)?;
		let pk = XOnlyPublicKey::from_slice(&bytes)
			.map_err(|e| Error::Generic(format!("invalid npub: {e}")))?;
		return Ok(NostrIdentity::ReadOnly(pk));
	}

	// Try hex private key (64 chars = 32 bytes)
	if s.len() == 64 {
		if let Ok(bytes) = hex::decode(s) {
			if let Ok(sk) = SecretKey::from_slice(&bytes) {
				let kp = Keypair::from_secret_key(SECP256K1, &sk);
				return Ok(NostrIdentity::Keypair(kp));
			}
		}
		// May be a hex public key
		if let Ok(bytes) = hex::decode(s) {
			if let Ok(pk) = XOnlyPublicKey::from_slice(&bytes) {
				return Ok(NostrIdentity::ReadOnly(pk));
			}
		}
	}

	Err(Error::Generic(format!(
		"unrecognised key format (expected nsec/npub or 64-char hex): {s}"
	)))
}

/// Generate a fresh nostr keypair.
pub fn generate_keys() -> Keypair {
	let secp = Secp256k1::new();
	let (sk, _) = secp.generate_keypair(&mut secp256k1::rand::thread_rng());
	Keypair::from_secret_key(&secp, &sk)
}

/// Generate a fresh keypair and return `(nsec, npub)` bech32-encoded strings.
pub fn generate_keypair_strings() -> (String, String) {
	const HRP_SECRET_KEY: Hrp = Hrp::parse_unchecked("nsec");
	let kp = generate_keys();
	let id = NostrIdentity::Keypair(kp.clone());
	let nsec = {
		let bytes = kp.secret_bytes();
		to_bech32(HRP_SECRET_KEY, &bytes)
			.unwrap_or_else(|_| hex::encode(bytes))
	};
	let npub = id.npub();
	(nsec, npub)
}

// ── git config I/O ───────────────────────────────────────────────────────────

/// Load the raw key string from git config (`nostr.key`) or `NOSTR_KEY` env.
pub fn load_key_from_git_config(repo_path: &Path) -> Result<String> {
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
			let s =
				String::from_utf8_lossy(&out.stdout).trim().to_string();
			if !s.is_empty() {
				return Ok(s);
			}
		}
	}

	if let Ok(val) = std::env::var("NOSTR_KEY") {
		if !val.is_empty() {
			return Ok(val);
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
	key: &str,
) -> Result<()> {
	let status = std::process::Command::new("git")
		.args([
			"-C",
			repo_path.to_str().unwrap_or("."),
			"config",
			"--local",
			"nostr.key",
			key,
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
	let raw = load_key_from_git_config(repo_path).ok()?;
	parse_key(&raw).ok()
}

// ── bech32 helpers ────────────────────────────────────────────────────────────

fn to_bech32(hrp: Hrp, data: &[u8]) -> Result<String> {
	bech32::encode::<Bech32>(hrp, data)
		.map_err(|e| Error::Generic(format!("bech32 encode: {e}")))
}

fn from_bech32(s: &str) -> Result<Vec<u8>> {
	let (_, bytes) = bech32::decode(s)
		.map_err(|e| Error::Generic(format!("bech32 decode: {e}")))?;
	Ok(bytes)
}
