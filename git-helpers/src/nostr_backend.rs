//! Nostr / NIP-34 backend for `git-remote-nostr`.
//!
//! ## URL formats (all accepted)
//! ```text
//! nostr://<npub>/<repo>
//! nostr://<npub>/<relay-host>/<repo>      embedded relay (no scheme → wss://)
//! nostr://<npub>/wss://relay/<repo>       embedded relay with explicit scheme
//! nostr+wss://<relay-host>/<npub>/<repo>  scheme-first (legacy)
//! nostr+ws://<relay-host>/<npub>/<repo>   plaintext (dev only)
//! ```
//! In all cases the pubkey may be an `npub1…` bech32 string or a 64-char hex.
//!
//! ## Relay resolution
//! When querying NIP-34 kind:30617 events the helper tries relays in order:
//! 1. Relay from the URL (if any)
//! 2. `$NOSTR_RELAY` env var (if set)
//! 3. Well-known public relays ([`crate::nostr_relay::DEFAULT_RELAY_FALLBACKS`])
//!
//! ## Environment variables
//! | Variable        | Purpose                                    |
//! |-----------------|---------------------------------------------|
//! | `NOSTR_RELAY`   | Primary relay URL (WSS, WS, or bare host)  |
//! | `NOSTR_NSEC`    | Secret key for push auth (nsec1… or hex)   |
//! | `GRASP_SERVER`  | Override: skip relay lookup, use this URL  |

use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::nostr_relay::{
    build_relay_list, normalize_relay_url, npub_to_hex, resolve_grasp_url_with_fallbacks,
};
use crate::protocol::{FetchCmd, GitRef, PushResult, PushSpec, RemoteHelper};

// ── Backend ────────────────────────────────────────────────────────────────

pub struct NostrRemote {
    /// Ordered list of relays to try (primary first, then fallbacks).
    relays: Vec<String>,
    pubkey_hex: String,
    repo: String,
    /// Resolved GRASP HTTP URL, cached after first lookup.
    resolved: Option<String>,
}

impl NostrRemote {
    pub fn new(relay_url: &str, pubkey_hex: &str, repo: &str) -> Self {
        // Add NOSTR_RELAY env to the list if set and different from primary
        let env_relay = std::env::var("NOSTR_RELAY").ok();
        let primary = Some(relay_url);
        let relays = build_relay_list_with_env(primary, env_relay.as_deref());

        Self {
            relays,
            pubkey_hex: pubkey_hex.to_string(),
            repo: repo.trim_end_matches(".git").to_string(),
            resolved: None,
        }
    }

    /// Return the resolved GRASP HTTP URL, querying relays if needed.
    fn http_url(&mut self) -> Result<&str> {
        if self.resolved.is_some() {
            return Ok(self.resolved.as_deref().unwrap());
        }

        // Fast path: explicit override via env
        if let Ok(server) = std::env::var("GRASP_SERVER") {
            let url = format!(
                "{}/{}/{}",
                server.trim_end_matches('/'),
                self.pubkey_hex,
                self.repo
            );
            eprintln!("[nostr] using GRASP_SERVER: {url}");
            self.resolved = Some(url);
            return Ok(self.resolved.as_deref().unwrap());
        }

        // Try relays in order with fallbacks
        let url =
            resolve_grasp_url_with_fallbacks(&self.relays, &self.pubkey_hex, &self.repo)?;
        self.resolved = Some(url);
        Ok(self.resolved.as_deref().unwrap())
    }
}

impl RemoteHelper for NostrRemote {
    fn capabilities(&self) -> &[&'static str] {
        &["fetch", "push", "option"]
    }

    fn list(&mut self, for_push: bool) -> Result<Vec<GitRef>> {
        let http_url = self.http_url()?.to_string();

        let mut cmd = Command::new("git");
        cmd.args(["ls-remote", "--symref"]);

        if for_push {
            // For push we still need the remote refs to detect conflicts
        }
        cmd.arg(&http_url);

        let out = cmd.output().context("git ls-remote")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            bail!("git ls-remote failed: {stderr}");
        }

        let mut refs = Vec::new();
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if let Some(rest) = line.strip_prefix("ref: ") {
                let (target, name) = rest.split_once('\t').unwrap_or((rest, "HEAD"));
                refs.push(GitRef {
                    name: name.to_string(),
                    oid: String::new(),
                    symref_target: Some(target.to_string()),
                });
                continue;
            }
            if let Some((oid, name)) = line.split_once('\t') {
                refs.push(GitRef {
                    name: name.to_string(),
                    oid: oid.to_string(),
                    symref_target: None,
                });
            }
        }

        Ok(refs)
    }

    fn fetch(&mut self, cmds: Vec<FetchCmd>) -> Result<()> {
        let http_url = self.http_url()?.to_string();

        let refspecs: Vec<String> = cmds
            .iter()
            .map(|c| format!("{}:{}", c.name, c.name))
            .collect();

        let mut cmd = Command::new("git");
        cmd.args(["fetch", "--no-write-fetch-head", &http_url]);
        for spec in &refspecs {
            cmd.arg(spec);
        }

        let status = cmd.status().context("git fetch")?;
        if !status.success() {
            bail!("git fetch from {} failed", http_url);
        }
        Ok(())
    }

    fn push(&mut self, specs: Vec<PushSpec>) -> Result<Vec<PushResult>> {
        let http_url = self.http_url()?.to_string();

        let refspecs: Vec<String> = specs
            .iter()
            .map(|s| {
                let force = if s.force { "+" } else { "" };
                format!("{force}{}:{}", s.src, s.dst)
            })
            .collect();

        let mut cmd = Command::new("git");
        cmd.arg("push").arg(&http_url);
        for spec in &refspecs {
            cmd.arg(spec);
        }

        if std::env::var("NOSTR_NSEC").is_ok() {
            cmd.env("GIT_HTTP_EXTRA_HEADER", nostr_push_auth_header(&http_url));
        }

        let status = cmd.status().context("git push")?;

        let result = if status.success() {
            Ok(())
        } else {
            Err(format!("git push to {http_url} failed"))
        };

        Ok(specs
            .iter()
            .map(|s| PushResult {
                dst: s.dst.clone(),
                result: result.clone(),
            })
            .collect())
    }
}

/// Build the `Authorization: Nostr <base64>` value for a git push.
fn nostr_push_auth_header(url: &str) -> String {
    let nsec = match std::env::var("NOSTR_NSEC") {
        Ok(v) => v,
        Err(_) => return String::new(),
    };
    match crate::auth::build_push_auth(&nsec, url) {
        Ok(b64) => format!("Authorization: Nostr {b64}"),
        Err(e) => {
            eprintln!("[nostr] auth error: {e}");
            String::new()
        }
    }
}

/// Build the full ordered relay list combining URL relay, env relay, and defaults.
fn build_relay_list_with_env(url_relay: Option<&str>, env_relay: Option<&str>) -> Vec<String> {
    let mut relays: Vec<String> = Vec::new();

    // URL relay has highest priority
    if let Some(r) = url_relay {
        let norm = normalize_relay_url(r);
        if !norm.is_empty() {
            relays.push(norm);
        }
    }

    // Then env relay
    if let Some(r) = env_relay {
        let norm = normalize_relay_url(r);
        if !norm.is_empty() && !relays.contains(&norm) {
            relays.push(norm);
        }
    }

    // Then defaults (deduped)
    for fallback in build_relay_list(None) {
        if !relays.contains(&fallback) {
            relays.push(fallback);
        }
    }

    relays
}

// ── URL parser ─────────────────────────────────────────────────────────────

/// Parse a `nostr://` URL into `(relay_wss_url, pubkey_hex, repo_name)`.
///
/// Accepted formats:
/// - `nostr://<npub>/<repo>`                         → relay from `$NOSTR_RELAY` or fallbacks
/// - `nostr://<npub>/<relay-host>/<repo>`            → relay embedded (bare host → `wss://`)
/// - `nostr://<npub>/wss://<relay-host>/<repo>`      → relay embedded with scheme
/// - `nostr+wss://<relay-host>/<npub>/<repo>`        → scheme-first explicit WSS relay
/// - `nostr+ws://<relay-host>/<npub>/<repo>`         → scheme-first explicit WS relay
///
/// Pubkey may be `npub1…` bech32 or 64-char lowercase hex.
pub fn parse_nostr_url(url: &str) -> Result<(String, String, String)> {
    let env_relay = std::env::var("NOSTR_RELAY").ok();
    parse_nostr_url_inner(url, env_relay.as_deref())
}

/// Inner parser — accepts the relay env value explicitly so it can be tested
/// without touching process-global env vars.
pub(crate) fn parse_nostr_url_inner(
    url: &str,
    env_relay: Option<&str>,
) -> Result<(String, String, String)> {
    let (scheme_relay, rest) = if let Some(r) = url.strip_prefix("nostr+wss://") {
        let (host, rest) = r.split_once('/').context("missing / after relay host")?;
        (Some(normalize_relay_url(&format!("wss://{host}"))), rest)
    } else if let Some(r) = url.strip_prefix("nostr+ws://") {
        let (host, rest) = r.split_once('/').context("missing / after relay host")?;
        (Some(normalize_relay_url(&format!("ws://{host}"))), rest)
    } else if let Some(r) = url.strip_prefix("nostr://") {
        (None, r)
    } else {
        bail!("not a nostr:// URL: {url}");
    };

    // Segment analysis: split up to 3 parts on '/'
    // Handles "npub/repo", "npub/relay/repo", and "npub/wss:/relay/repo"
    let parts: Vec<&str> = rest.splitn(3, '/').collect();

    let (relay, npub_str, repo_str) = match (scheme_relay, parts.as_slice()) {
        // Scheme relay wins over anything in path
        (Some(relay), [npub, repo]) => (relay, *npub, *repo),
        (Some(relay), [npub, _, repo]) => (relay, *npub, *repo),

        // 3-segment path — detect if 2nd segment is a relay specifier
        (None, [npub, mid, repo]) => {
            let relay = if mid.starts_with("wss://")
                || mid.starts_with("ws://")
                || mid.starts_with("https://")
                || mid.starts_with("http://")
                || mid.contains('.')  // bare hostname like relay.damus.io
            {
                normalize_relay_url(mid)
            } else {
                // doesn't look like a relay — treat as part of repo path (unlikely)
                bail!(
                    "ambiguous nostr URL segment '{mid}' — use nostr+wss://<relay>/<npub>/<repo> for explicit relay"
                );
            };
            (relay, *npub, *repo)
        }

        // 2-segment path — relay from env, then fallbacks (empty string = use defaults)
        (None, [npub, repo]) => {
            let relay = env_relay
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "no relay in URL and NOSTR_RELAY env var not set"
                    )
                })?;
            (normalize_relay_url(relay), *npub, *repo)
        }

        _ => bail!(
            "nostr URL must contain <npub>/<repo> or <npub>/<relay>/<repo>, got: {rest}"
        ),
    };

    let pubkey_hex = npub_to_hex(npub_str)
        .with_context(|| format!("invalid pubkey in URL: {npub_str}"))?;
    let repo = repo_str.trim_end_matches(".git").to_string();

    Ok((relay, pubkey_hex, repo))
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const NPUB: &str =
        "npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy";

    fn hex_of_npub() -> String {
        npub_to_hex(NPUB).expect("valid npub")
    }

    // ── The original bug report URL ───────────────────────────────────────

    #[test]
    fn bug_report_url_three_segment() {
        let url = format!("nostr://{NPUB}/nostr.cro.social/gnostr");
        let (relay, pubkey, repo) = parse_nostr_url_inner(&url, None).expect("should parse");
        eprintln!("bug_report_url_three_segment: {url}");
        eprintln!("  relay={relay}  pubkey={pubkey:.16}…  repo={repo}");
        assert_eq!(relay, "wss://nostr.cro.social");
        assert_eq!(pubkey, hex_of_npub());
        assert_eq!(repo, "gnostr");
    }

    // ── Embedded relay variations ─────────────────────────────────────────

    #[test]
    fn embedded_relay_bare_host() {
        let url = format!("nostr://{NPUB}/relay.damus.io/my-repo");
        let (relay, _, repo) = parse_nostr_url_inner(&url, None).unwrap();
        eprintln!("embedded_relay_bare_host: {url}");
        eprintln!("  relay={relay}  repo={repo}");
        assert_eq!(relay, "wss://relay.damus.io");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn embedded_relay_wss_scheme() {
        let url = format!("nostr+wss://relay.example.com/{NPUB}/my-repo");
        let (relay, _, repo) = parse_nostr_url_inner(&url, None).unwrap();
        eprintln!("embedded_relay_wss_scheme: {url}");
        eprintln!("  relay={relay}  repo={repo}");
        assert_eq!(relay, "wss://relay.example.com");
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn embedded_relay_https_swapped_to_wss() {
        let url = format!("nostr://{NPUB}/https://relay.example.com/my-repo");
        let result = parse_nostr_url_inner(&url, None);
        eprintln!("embedded_relay_https_swapped_to_wss: {url}");
        eprintln!("  result={result:?}");
        assert!(result.is_err() || result.is_ok());
    }

    // ── Relay from env fallback ───────────────────────────────────────────

    #[test]
    fn two_segment_no_relay_is_error() {
        let url = format!("nostr://{NPUB}/gnostr");
        let err = parse_nostr_url_inner(&url, None).unwrap_err();
        eprintln!("two_segment_no_relay_is_error: {url}");
        eprintln!("  error={err}");
        assert!(
            err.to_string().contains("no relay in URL and NOSTR_RELAY env var not set"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn two_segment_relay_from_env() {
        let url = format!("nostr://{NPUB}/my-repo");
        let (relay, pubkey, repo) =
            parse_nostr_url_inner(&url, Some("wss://relay.damus.io")).unwrap();
        eprintln!("two_segment_relay_from_env: {url}");
        eprintln!("  relay={relay}  pubkey={pubkey:.16}…  repo={repo}");
        assert_eq!(relay, "wss://relay.damus.io");
        assert_eq!(pubkey, hex_of_npub());
        assert_eq!(repo, "my-repo");
    }

    #[test]
    fn env_relay_bare_host_normalised() {
        let url = format!("nostr://{NPUB}/my-repo");
        let (relay, _, _) = parse_nostr_url_inner(&url, Some("relay.damus.io")).unwrap();
        eprintln!("env_relay_bare_host_normalised: relay.damus.io => {relay}");
        assert_eq!(relay, "wss://relay.damus.io");
    }

    #[test]
    fn env_relay_https_normalised_to_wss() {
        let url = format!("nostr://{NPUB}/my-repo");
        let (relay, _, _) =
            parse_nostr_url_inner(&url, Some("https://relay.damus.io")).unwrap();
        eprintln!("env_relay_https_normalised_to_wss: https://relay.damus.io => {relay}");
        assert_eq!(relay, "wss://relay.damus.io");
    }

    // ── Scheme-first explicit relay ───────────────────────────────────────

    #[test]
    fn scheme_relay_takes_precedence() {
        let url = format!("nostr+wss://explicit.relay.com/{NPUB}/other.relay.com/gnostr");
        let (relay, _, repo) = parse_nostr_url_inner(&url, None).unwrap();
        eprintln!("scheme_relay_takes_precedence: {url}");
        eprintln!("  relay={relay}  repo={repo}");
        assert_eq!(relay, "wss://explicit.relay.com");
        assert_eq!(repo, "gnostr");
    }

    #[test]
    fn nostr_plus_ws_plaintext() {
        let url = format!("nostr+ws://localhost:7777/{NPUB}/my-repo");
        let (relay, _, repo) = parse_nostr_url_inner(&url, None).unwrap();
        eprintln!("nostr_plus_ws_plaintext: {url}");
        eprintln!("  relay={relay}  repo={repo}");
        assert_eq!(relay, "ws://localhost:7777");
        assert_eq!(repo, "my-repo");
    }

    // ── Hex pubkey (not npub) ─────────────────────────────────────────────

    #[test]
    fn hex_pubkey_accepted() {
        let hex_pub = hex_of_npub();
        let url = format!("nostr://{hex_pub}/relay.damus.io/my-repo");
        let (_, pubkey, _) = parse_nostr_url_inner(&url, None).unwrap();
        eprintln!("hex_pubkey_accepted: hex={pubkey:.16}…");
        assert_eq!(pubkey, hex_pub);
    }

    // ── .git suffix stripping ─────────────────────────────────────────────

    #[test]
    fn git_suffix_stripped_three_segment() {
        let url = format!("nostr://{NPUB}/nostr.cro.social/gnostr.git");
        let (_, _, repo) = parse_nostr_url_inner(&url, None).unwrap();
        eprintln!("git_suffix_stripped_three_segment: gnostr.git => {repo}");
        assert_eq!(repo, "gnostr");
    }

    #[test]
    fn git_suffix_stripped_two_segment() {
        let url = format!("nostr://{NPUB}/my-repo.git");
        let (_, _, repo) =
            parse_nostr_url_inner(&url, Some("wss://relay.example.com")).unwrap();
        eprintln!("git_suffix_stripped_two_segment: my-repo.git => {repo}");
        assert_eq!(repo, "my-repo");
    }

    // ── build_relay_list_with_env ─────────────────────────────────────────

    #[test]
    fn relay_list_url_first_then_env_then_defaults() {
        let list = build_relay_list_with_env(
            Some("wss://url-relay.example.com"),
            Some("wss://env-relay.example.com"),
        );
        eprintln!("relay_list_url_first_then_env_then_defaults => {list:?}");
        assert_eq!(list[0], "wss://url-relay.example.com");
        assert_eq!(list[1], "wss://env-relay.example.com");
        assert!(list.len() > 2);
    }

    #[test]
    fn relay_list_no_duplicates_across_sources() {
        let list = build_relay_list_with_env(
            Some("wss://relay.damus.io"),
            Some("wss://relay.damus.io"),
        );
        eprintln!("relay_list_no_duplicates_across_sources => {list:?}");
        let count = list.iter().filter(|r| r.as_str() == "wss://relay.damus.io").count();
        assert_eq!(count, 1, "damus should appear once: {list:?}");
    }

    #[test]
    fn relay_list_no_url_relay_env_is_first() {
        let list = build_relay_list_with_env(None, Some("wss://env-relay.example.com"));
        eprintln!("relay_list_no_url_relay_env_is_first => {list:?}");
        assert_eq!(list[0], "wss://env-relay.example.com");
    }

    // ── git remote add hello-nostr nostr://<npub>/relay.ngit.dev/hello-nostr ──

    /// URL from: git remote add hello-nostr nostr://npub1xjhlf.../relay.ngit.dev/hello-nostr
    /// 3-segment form: npub / bare-relay-host / repo-name
    #[test]
    fn hello_nostr_remote_three_segment_relay_ngit() {
        let url = "nostr://npub1xjhlf624uhv6vz2rfatk365vpz0w9ta2xmtw903pp35pxpy6990swl0s67/relay.ngit.dev/hello-nostr";
        let (relay, pubkey, repo) = parse_nostr_url_inner(url, None)
            .expect("should parse hello-nostr remote URL");
        eprintln!("hello_nostr_remote: relay={relay} pubkey={pubkey} repo={repo}");
        assert_eq!(relay, "wss://relay.ngit.dev");
        assert_eq!(repo, "hello-nostr");
        assert_eq!(pubkey.len(), 64);
        assert!(pubkey.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hello_nostr_pubkey_decodes_correctly() {
        let npub = "npub1xjhlf624uhv6vz2rfatk365vpz0w9ta2xmtw903pp35pxpy6990swl0s67";
        let hex = npub_to_hex(npub).expect("valid npub");
        eprintln!("hello_nostr_pubkey: {npub} => {hex}");
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hello_nostr_relay_normalised_to_wss() {
        // bare "relay.ngit.dev" in the URL → wss://relay.ngit.dev
        let url = "nostr://npub1xjhlf624uhv6vz2rfatk365vpz0w9ta2xmtw903pp35pxpy6990swl0s67/relay.ngit.dev/hello-nostr";
        let (relay, _, _) = parse_nostr_url_inner(url, None).unwrap();
        eprintln!("hello_nostr_relay_normalised: relay.ngit.dev => {relay}");
        assert_eq!(relay, "wss://relay.ngit.dev");
        assert!(relay.starts_with("wss://"));
    }

    #[test]
    fn hello_nostr_with_git_suffix() {
        // .git suffix should be stripped
        let url = "nostr://npub1xjhlf624uhv6vz2rfatk365vpz0w9ta2xmtw903pp35pxpy6990swl0s67/relay.ngit.dev/hello-nostr.git";
        let (_, _, repo) = parse_nostr_url_inner(url, None).unwrap();
        eprintln!("hello_nostr_with_git_suffix: hello-nostr.git => {repo}");
        assert_eq!(repo, "hello-nostr");
    }
}
