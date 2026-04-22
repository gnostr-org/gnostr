//! Blossom blob storage backend for `git-remote-blossom`.
//!
//! ## URL format
//! ```text
//! blossom://<server-host>/<pubkey-hex>/<repo-name>
//! blossom+https://<server-host>/<pubkey-hex>/<repo-name>
//! ```
//!
//! ## How it works
//!
//! ### Refs manifest
//! A JSON blob stored on the Blossom server with
//! `Content-Type: application/x-git-refs+json`.  The server's BUD-02 list
//! endpoint (`GET /<pubkey>`) is queried to find the newest manifest for
//! this repo (by `created_at` timestamp).
//!
//! Manifest schema:
//! ```json
//! {
//!   "repo":           "my-repo",
//!   "refs":           { "refs/heads/main": "<sha1>", ... },
//!   "head":           "refs/heads/main",
//!   "bundles":        ["<sha256>", ...],
//!   "prev_manifest":  "<sha256>"
//! }
//! ```
//!
//! ### Object storage
//! Objects are stored as git bundles uploaded to Blossom with
//! `Content-Type: application/x-git-bundle`.
//!
//! On push:
//! 1. `git bundle create -` produces a thin bundle of new objects.
//! 2. The bundle is uploaded → returns a sha256.
//! 3. A new manifest is built and uploaded.
//!
//! On fetch:
//! 1. Download each bundle sha256 from the manifest.
//! 2. Pipe each through `git bundle unbundle` to inject objects locally.
//!
//! ### Authentication
//! Set `NOSTR_NSEC` (nsec1… bech32 or hex) to sign upload requests with a
//! BUD-01 `Authorization: Nostr <base64-event>` header.
//! If unset, uploads are attempted without auth (works on open servers).

use std::collections::HashMap;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::auth::build_upload_auth;
use crate::protocol::{FetchCmd, GitRef, PushResult, PushSpec, RemoteHelper};

// ── Manifest ───────────────────────────────────────────────────────────────

/// JSON refs manifest stored as a Blossom blob.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefsManifest {
    pub repo: String,
    /// refname → SHA-1 OID
    pub refs: HashMap<String, String>,
    /// Symref target for HEAD (e.g. `"refs/heads/main"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// SHA-256 hashes of bundle blobs (oldest first).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bundles: Vec<String>,
    /// SHA-256 of the previous manifest blob (forms an immutable chain).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_manifest: Option<String>,
}

// ── BlobDescriptor (minimal, matches BUD-01 response) ─────────────────────

#[derive(Debug, Deserialize)]
struct BlobDescriptor {
    pub sha256: String,
    pub url: String,
    #[serde(rename = "type")]
    pub content_type: Option<String>,
    pub created: Option<u64>,
}

// ── Backend ────────────────────────────────────────────────────────────────

pub struct BlossomRemote {
    /// Base URL of the Blossom server, e.g. `https://blossom.example.com`.
    server: String,
    /// Hex-encoded public key of the repo owner.
    pubkey: String,
    /// Repository name (no `.git` suffix).
    repo: String,
    /// Optional nsec for signing upload requests.
    nsec: Option<String>,
    client: reqwest::blocking::Client,
}

impl BlossomRemote {
    pub fn new(server: &str, pubkey: &str, repo: &str) -> Self {
        let nsec = std::env::var("NOSTR_NSEC").ok();
        if nsec.is_none() {
            eprintln!("[blossom] NOSTR_NSEC not set — uploads will be unauthenticated");
        }
        Self {
            server: server.trim_end_matches('/').to_string(),
            pubkey: pubkey.to_string(),
            repo: repo.to_string(),
            nsec,
            client: reqwest::blocking::Client::builder()
                .user_agent("git-remote-blossom/0.1")
                .build()
                .expect("build HTTP client"),
        }
    }

    /// Create with a pre-built reqwest client (e.g. one configured with a SOCKS5 proxy).
    pub fn with_client(
        client: reqwest::blocking::Client,
        nsec: Option<String>,
        server: &str,
        pubkey: &str,
        repo: &str,
    ) -> Self {
        Self {
            server: server.trim_end_matches('/').to_string(),
            pubkey: pubkey.to_string(),
            repo: repo.to_string(),
            nsec,
            client,
        }
    }

    // ── Private helpers ───────────────────────────────────────────────────

    /// Query BUD-02 (`GET /<pubkey>`) and return the newest refs manifest
    /// for `self.repo`, or `None` if the repo has no history yet.
    fn fetch_manifest(&self) -> Result<Option<RefsManifest>> {
        let url = format!("{}/{}", self.server, self.pubkey);
        let resp = self
            .client
            .get(&url)
            .send()
            .with_context(|| format!("GET {url}"))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let blobs: Vec<BlobDescriptor> = resp
            .json()
            .context("parse BUD-02 blob list")?;

        // Filter manifest blobs for this repo
        let ct_manifest = "application/x-git-refs+json";
        let mut candidates: Vec<&BlobDescriptor> = blobs
            .iter()
            .filter(|b| b.content_type.as_deref() == Some(ct_manifest))
            .collect();

        // Sort newest-first by `created` timestamp
        candidates.sort_by(|a, b| {
            b.created.unwrap_or(0).cmp(&a.created.unwrap_or(0))
        });

        for blob in candidates {
            let manifest: RefsManifest = self
                .client
                .get(&blob.url)
                .send()
                .with_context(|| format!("GET {}", blob.url))?
                .json()
                .context("parse manifest JSON")?;

            if manifest.repo == self.repo {
                eprintln!(
                    "[blossom] found manifest {} for repo '{}'",
                    &blob.sha256[..8],
                    self.repo
                );
                return Ok(Some(manifest));
            }
        }

        Ok(None)
    }

    /// Upload `data` to the Blossom server.  Returns the blob sha256.
    fn upload_blob(&self, data: Vec<u8>, content_type: &str) -> Result<String> {
        let url = format!("{}/upload", self.server);

        // Compute SHA-256 for the auth header
        let sha256 = {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(&data);
            hex::encode(h.finalize())
        };

        let mut req = self
            .client
            .put(&url)
            .header("Content-Type", content_type)
            .body(data);

        if let Some(nsec) = &self.nsec {
            match build_upload_auth(nsec, &sha256, content_type) {
                Ok(auth) => {
                    req = req.header("Authorization", format!("Nostr {auth}"));
                }
                Err(e) => {
                    eprintln!("[blossom] auth error (continuing without auth): {e}");
                }
            }
        }

        let resp: serde_json::Value = req
            .send()
            .with_context(|| format!("PUT {url}"))?
            .error_for_status()
            .context("upload failed")?
            .json()
            .context("parse upload response")?;

        let sha = resp["sha256"]
            .as_str()
            .context("missing sha256 in upload response")?
            .to_string();

        eprintln!("[blossom] uploaded {} bytes → {}", resp["size"].as_u64().unwrap_or(0), &sha[..8]);
        Ok(sha)
    }

    /// Download a blob by its SHA-256.
    fn download_blob(&self, sha256: &str) -> Result<Vec<u8>> {
        let url = format!("{}/{}", self.server, sha256);
        let bytes = self
            .client
            .get(&url)
            .send()
            .with_context(|| format!("GET {url}"))?
            .error_for_status()
            .with_context(|| format!("download {}", &sha256[..8]))?
            .bytes()
            .context("read blob body")?;
        Ok(bytes.to_vec())
    }

    /// Resolve a local ref to its SHA-1 OID using `git rev-parse`.
    fn rev_parse(&self, refspec: &str) -> Result<String> {
        let out = Command::new("git")
            .args(["rev-parse", refspec])
            .output()
            .context("git rev-parse")?;
        if !out.status.success() {
            bail!("git rev-parse {refspec} failed");
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Return the symbolic ref target for HEAD.
    fn head_symref(&self) -> Option<String> {
        let out = Command::new("git")
            .args(["symbolic-ref", "HEAD"])
            .output()
            .ok()?;
        if out.status.success() {
            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            None
        }
    }
}

impl RemoteHelper for BlossomRemote {
    fn capabilities(&self) -> &[&'static str] {
        &["fetch", "push", "option"]
    }

    fn list(&mut self, _for_push: bool) -> Result<Vec<GitRef>> {
        let Some(manifest) = self.fetch_manifest()? else {
            eprintln!("[blossom] empty repo — no refs");
            return Ok(vec![]);
        };

        let mut refs: Vec<GitRef> = manifest
            .refs
            .iter()
            .map(|(name, oid)| GitRef {
                name: name.clone(),
                oid: oid.clone(),
                symref_target: None,
            })
            .collect();

        if let Some(target) = &manifest.head {
            refs.push(GitRef {
                name: "HEAD".into(),
                oid: String::new(),
                symref_target: Some(target.clone()),
            });
        }

        Ok(refs)
    }

    fn fetch(&mut self, _cmds: Vec<FetchCmd>) -> Result<()> {
        let Some(manifest) = self.fetch_manifest()? else {
            return Ok(());
        };

        for sha256 in &manifest.bundles {
            eprintln!("[blossom] fetching bundle {}", &sha256[..8]);
            let data = self.download_blob(sha256)?;

            // Write bundle to a temp file then unbundle
            let tmp = std::env::temp_dir().join(format!("blossom-{sha256}.bundle"));
            std::fs::write(&tmp, &data)
                .with_context(|| format!("write bundle {}", tmp.display()))?;

            let status = Command::new("git")
                .args(["bundle", "unbundle"])
                .arg(&tmp)
                .status()
                .context("git bundle unbundle")?;

            let _ = std::fs::remove_file(&tmp);

            if !status.success() {
                bail!("git bundle unbundle failed for {}", &sha256[..8]);
            }
        }

        Ok(())
    }

    fn push(&mut self, specs: Vec<PushSpec>) -> Result<Vec<PushResult>> {
        let prev_manifest = self.fetch_manifest()?;

        // Collect non-delete refspecs for bundle creation
        let push_refs: Vec<&PushSpec> = specs.iter().filter(|s| !s.src.is_empty()).collect();

        let bundle_sha256 = if !push_refs.is_empty() {
            let mut cmd = Command::new("git");
            cmd.args(["bundle", "create", "-"]);

            // Add prerequisites so the bundle is thin (incremental)
            if let Some(prev) = &prev_manifest {
                for oid in prev.refs.values() {
                    cmd.arg(format!("^{oid}"));
                }
            }
            for s in &push_refs {
                cmd.arg(&s.src);
            }

            let bundle = cmd.output().context("git bundle create")?;
            if !bundle.status.success() {
                let err = String::from_utf8_lossy(&bundle.stderr);
                bail!("git bundle create failed: {err}");
            }

            eprintln!("[blossom] uploading bundle ({} bytes)…", bundle.stdout.len());
            let sha = self.upload_blob(bundle.stdout, "application/x-git-bundle")?;
            Some(sha)
        } else {
            None
        };

        // Resolve new ref OIDs and build results
        let mut new_refs = prev_manifest
            .as_ref()
            .map(|m| m.refs.clone())
            .unwrap_or_default();

        let mut results = Vec::new();

        for spec in &specs {
            if spec.src.is_empty() {
                // Deletion
                new_refs.remove(&spec.dst);
                results.push(PushResult { dst: spec.dst.clone(), result: Ok(()) });
                continue;
            }
            match self.rev_parse(&spec.src) {
                Ok(oid) => {
                    new_refs.insert(spec.dst.clone(), oid);
                    results.push(PushResult { dst: spec.dst.clone(), result: Ok(()) });
                }
                Err(e) => {
                    results.push(PushResult {
                        dst: spec.dst.clone(),
                        result: Err(format!("rev-parse failed: {e}")),
                    });
                }
            }
        }

        // Build bundle list (append new bundle)
        let mut bundles = prev_manifest
            .as_ref()
            .map(|m| m.bundles.clone())
            .unwrap_or_default();
        let prev_manifest_sha = prev_manifest
            .as_ref()
            .and_then(|m| m.bundles.last().cloned());

        if let Some(sha) = bundle_sha256 {
            bundles.push(sha);
        }

        let manifest = RefsManifest {
            repo: self.repo.clone(),
            refs: new_refs,
            head: self.head_symref(),
            bundles,
            prev_manifest: prev_manifest_sha,
        };

        let manifest_json = serde_json::to_vec_pretty(&manifest)?;
        let msha = self.upload_blob(manifest_json, "application/x-git-refs+json")?;
        eprintln!("[blossom] manifest updated → {}", &msha[..8]);

        Ok(results)
    }
}

// ── URL parser ─────────────────────────────────────────────────────────────

/// Parse a `blossom://` or `blossom+https://` URL into components.
///
/// Accepted formats:
/// - `blossom://<host>/<pubkey>/<repo>`          → HTTPS to host
/// - `blossom+https://<host>/<pubkey>/<repo>`    → HTTPS
/// - `blossom+http://<host>/<pubkey>/<repo>`     → HTTP (dev only)
///
/// Returns `(server_url, pubkey_hex, repo_name)`.
pub fn parse_blossom_url(url: &str) -> Result<(String, String, String)> {
    // Strip the blossom[+scheme]:// prefix
    let (scheme_hint, rest) = if let Some(r) = url.strip_prefix("blossom+https://") {
        ("https", r)
    } else if let Some(r) = url.strip_prefix("blossom+http://") {
        ("http", r)
    } else if let Some(r) = url.strip_prefix("blossom://") {
        ("https", r)
    } else {
        bail!("not a blossom:// URL: {url}");
    };

    // rest = "<host>/<pubkey>/<repo>"
    let parts: Vec<&str> = rest.splitn(3, '/').collect();
    if parts.len() != 3 {
        bail!(
            "blossom URL must be blossom://<host>/<pubkey>/<repo>, got: {url}"
        );
    }

    let server = format!("{scheme_hint}://{}", parts[0]);
    let pubkey = parts[1].to_string();
    let repo = parts[2].trim_end_matches(".git").to_string();

    if pubkey.len() != 64 || !pubkey.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("pubkey must be a 64-char hex string, got: {pubkey}");
    }

    Ok((server, pubkey, repo))
}
