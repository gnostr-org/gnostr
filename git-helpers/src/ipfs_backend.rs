//! IPFS/Kubo MFS backend for `git-remote-ipfs`.
//!
//! Uses the Kubo HTTP RPC API (`/api/v0/files/*`) to store git repositories
//! in the Mutable File System (MFS) — a local, mutable namespace on the IPFS
//! node.  Bundles and the refs manifest are stored under `/git/<repo>/`.
//!
//! ## URL format
//! ```text
//! ipfs://<repo-name>                  → API at $IPFS_API (default http://127.0.0.1:5001)
//! ipfs+api://<host>:<port>/<repo>     → API at http://<host>:<port>
//! ```
//!
//! ## Environment variables
//! | Variable   | Default                    | Description            |
//! |------------|----------------------------|------------------------|
//! | `IPFS_API` | `http://127.0.0.1:5001`    | Kubo RPC API base URL  |
//!
//! ## MFS layout
//! ```text
//! /git/<repo>/refs.json            ← refs manifest (same schema as blossom)
//! /git/<repo>/bundles/<sha256>     ← git bundle blobs (content-addressed)
//! ```
//!
//! ## Sharing
//! After pushing, run `ipfs files stat --hash /git/<repo>` to get the root CID.
//! Share that CID for read-only access, or use `ipfs name publish` to publish
//! the CID under an IPNS key.

use std::collections::HashMap;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::protocol::{FetchCmd, GitRef, PushResult, PushSpec, RemoteHelper};

// ── Manifest (same schema as blossom_backend::RefsManifest) ───────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RefsManifest {
    pub repo: String,
    pub refs: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    /// SHA-256 hashes of bundle blobs (oldest first).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bundles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_manifest: Option<String>,
}

// ── Backend ────────────────────────────────────────────────────────────────

pub struct IpfsRemote {
    /// Kubo API base URL, e.g. `http://127.0.0.1:5001`.
    api: String,
    /// Repository name (used as the MFS directory name under `/git/`).
    repo: String,
    client: reqwest::blocking::Client,
}

impl IpfsRemote {
    pub fn new(api: &str, repo: &str) -> Self {
        Self {
            api: api.trim_end_matches('/').to_string(),
            repo: repo.to_string(),
            client: reqwest::blocking::Client::builder()
                .user_agent("git-remote-ipfs/0.1")
                .build()
                .expect("build HTTP client"),
        }
    }

    // ── MFS helpers ───────────────────────────────────────────────────────

    fn mfs_refs_path(&self) -> String {
        format!("/git/{}/refs.json", self.repo)
    }

    fn mfs_bundle_path(&self, sha256: &str) -> String {
        format!("/git/{}/bundles/{}", self.repo, sha256)
    }

    /// Read raw bytes from an MFS path. Returns `None` if not found (404).
    fn mfs_read(&self, path: &str) -> Result<Option<Vec<u8>>> {
        let url = format!(
            "{}/api/v0/files/read?arg={}",
            self.api,
            urlencoding::encode(path)
        );
        let resp = self
            .client
            .post(&url)
            .send()
            .with_context(|| format!("POST {url}"))?;

        if resp.status().as_u16() == 500 {
            // Kubo returns 500 with JSON error for missing files
            let body: serde_json::Value = resp.json().unwrap_or_default();
            let msg = body["Message"].as_str().unwrap_or("");
            if msg.contains("does not exist") || msg.contains("file does not exist") {
                return Ok(None);
            }
            bail!("mfs read {path}: {msg}");
        }

        if !resp.status().is_success() {
            bail!("mfs read {path}: HTTP {}", resp.status());
        }

        Ok(Some(resp.bytes().context("read MFS body")?.to_vec()))
    }

    /// Write bytes to an MFS path (create + truncate + parents).
    fn mfs_write(&self, path: &str, data: Vec<u8>) -> Result<()> {
        let url = format!(
            "{}/api/v0/files/write?arg={}&create=true&truncate=true&parents=true",
            self.api,
            urlencoding::encode(path)
        );
        let part = reqwest::blocking::multipart::Part::bytes(data)
            .file_name("data")
            .mime_str("application/octet-stream")
            .context("build multipart part")?;
        let form = reqwest::blocking::multipart::Form::new().part("file", part);

        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .with_context(|| format!("POST {url}"))?;

        if !resp.status().is_success() {
            let body: serde_json::Value = resp.json().unwrap_or_default();
            let msg = body["Message"].as_str().unwrap_or("unknown error");
            bail!("mfs write {path}: {msg}");
        }
        Ok(())
    }

    // ── Manifest helpers ──────────────────────────────────────────────────

    fn fetch_manifest(&self) -> Result<Option<RefsManifest>> {
        let path = self.mfs_refs_path();
        let Some(data) = self.mfs_read(&path)? else {
            return Ok(None);
        };
        let manifest: RefsManifest = serde_json::from_slice(&data).context("parse refs.json")?;
        eprintln!("[ipfs] found manifest for repo '{}'", self.repo);
        Ok(Some(manifest))
    }

    fn save_manifest(&self, manifest: &RefsManifest) -> Result<()> {
        let path = self.mfs_refs_path();
        let data = serde_json::to_vec_pretty(manifest)?;
        self.mfs_write(&path, data)?;
        eprintln!("[ipfs] manifest saved to {path}");
        Ok(())
    }

    fn sha256_of(data: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(data);
        hex::encode(h.finalize())
    }

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

impl RemoteHelper for IpfsRemote {
    fn capabilities(&self) -> &[&'static str] {
        &["fetch", "push", "option"]
    }

    fn list(&mut self, _for_push: bool) -> Result<Vec<GitRef>> {
        let Some(manifest) = self.fetch_manifest()? else {
            eprintln!("[ipfs] empty repo — no refs");
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
            eprintln!("[ipfs] fetching bundle {}", &sha256[..8]);
            let path = self.mfs_bundle_path(sha256);
            let data = self
                .mfs_read(&path)?
                .with_context(|| format!("bundle {} not found in MFS", &sha256[..8]))?;

            let tmp = std::env::temp_dir().join(format!("ipfs-{sha256}.bundle"));
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

        let push_refs: Vec<&PushSpec> = specs.iter().filter(|s| !s.src.is_empty()).collect();

        let bundle_sha256 = if !push_refs.is_empty() {
            let mut cmd = Command::new("git");
            cmd.args(["bundle", "create", "-"]);

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

            let sha256 = Self::sha256_of(&bundle.stdout);
            let path = self.mfs_bundle_path(&sha256);
            eprintln!("[ipfs] uploading bundle ({} bytes) → {}", bundle.stdout.len(), &sha256[..8]);
            self.mfs_write(&path, bundle.stdout)?;
            Some(sha256)
        } else {
            None
        };

        let mut new_refs = prev_manifest
            .as_ref()
            .map(|m| m.refs.clone())
            .unwrap_or_default();

        let mut results = Vec::new();

        for spec in &specs {
            if spec.src.is_empty() {
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

        self.save_manifest(&manifest)?;
        Ok(results)
    }
}

// ── URL parser ─────────────────────────────────────────────────────────────

/// Parse an `ipfs://` URL into `(api_url, repo_name)`.
///
/// Accepted formats:
/// - `ipfs://<repo>`                → API from `$IPFS_API` or `http://127.0.0.1:5001`
/// - `ipfs+api://<host>:<port>/<repo>` → API at `http://<host>:<port>`
pub fn parse_ipfs_url(url: &str) -> Result<(String, String)> {
    let default_api =
        std::env::var("IPFS_API").unwrap_or_else(|_| "http://127.0.0.1:5001".into());

    if let Some(rest) = url.strip_prefix("ipfs+api://") {
        // ipfs+api://<host>:<port>/<repo>
        let (host_port, repo) = rest
            .split_once('/')
            .with_context(|| format!("ipfs+api:// URL must be ipfs+api://<host>:<port>/<repo>, got: {url}"))?;
        let api = format!("http://{host_port}");
        Ok((api, repo.trim_end_matches(".git").to_string()))
    } else if let Some(rest) = url.strip_prefix("ipfs://") {
        // ipfs://<repo>  — api from env
        let repo = rest.trim_end_matches(".git");
        if repo.is_empty() {
            bail!("ipfs:// URL must include a repo name, got: {url}");
        }
        Ok((default_api, repo.to_string()))
    } else {
        bail!("not an ipfs:// URL: {url}");
    }
}

// ── Internal URL encoder (avoids adding a dep for a tiny helper) ──────────

mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 8);
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
                | b'-' | b'_' | b'.' | b'~' | b'/' => out.push(b as char),
                _ => {
                    out.push('%');
                    out.push_str(&format!("{b:02X}"));
                }
            }
        }
        out
    }
}
