//! Git remote helper protocol — stdio command dispatcher.
//!
//! Implements the "dumb" side of the git remote helper protocol as described
//! in `gitremote-helpers(7)`. Helpers communicate with git over stdin/stdout
//! using a simple line-oriented text protocol.
//!
//! # Capabilities supported
//! - `fetch`  — receive objects via git bundles
//! - `push`   — send objects via git bundles  
//! - `option` — set transport options (verbosity, progress)
//!
//! # Wire protocol summary
//! ```text
//! git → helper: capabilities
//! git ← helper: fetch\npush\noption\n\n
//!
//! git → helper: list [for-push]
//! git ← helper: <oid> <refname>\n … \n\n
//!
//! git → helper: fetch <oid> <refname>\n … \n\n
//! git ← helper: \n  (blank = done)
//!
//! git → helper: push [+]<src>:<dst>\n … \n\n
//! git ← helper: ok <dst>\nerror <dst> <msg>\n … \n\n
//! ```

use std::io::{self, BufRead, Write};

use anyhow::{Context, Result};

// ── Public types ───────────────────────────────────────────────────────────

/// A git ref as advertised by the remote.
#[derive(Debug, Clone)]
pub struct GitRef {
    /// Full refname, e.g. `refs/heads/main` or `HEAD`.
    pub name: String,
    /// Object ID (SHA1 hex).  Empty for symbolic refs.
    pub oid: String,
    /// If set, this ref is a symref pointing to `symref_target`.
    /// git expects the line format `@<target> <name>`.
    pub symref_target: Option<String>,
}

/// A single fetch request from git.
#[derive(Debug, Clone)]
pub struct FetchCmd {
    pub oid: String,
    pub name: String,
}

/// A single push refspec from git.
#[derive(Debug, Clone)]
pub struct PushSpec {
    pub force: bool,
    pub src: String, // empty = delete
    pub dst: String,
}

/// Result of a single push refspec.
#[derive(Debug, Clone)]
pub struct PushResult {
    pub dst: String,
    pub result: Result<(), String>,
}

// ── Trait ──────────────────────────────────────────────────────────────────

/// Implement this trait to build a git remote helper backend.
pub trait RemoteHelper {
    /// Advertised capabilities (e.g. `["fetch", "push", "option"]`).
    fn capabilities(&self) -> &[&'static str];

    /// Return the current ref list.  `for_push` is true when git called
    /// `list for-push` (preparing a push rather than a fetch).
    fn list(&mut self, for_push: bool) -> Result<Vec<GitRef>>;

    /// Fetch the requested refs.  Implementations should make the objects
    /// available in the local repository (e.g. via `git bundle unbundle`).
    fn fetch(&mut self, cmds: Vec<FetchCmd>) -> Result<()>;

    /// Push the given refspecs.  Return one `PushResult` per spec.
    fn push(&mut self, specs: Vec<PushSpec>) -> Result<Vec<PushResult>>;
}

// ── Wire helpers ───────────────────────────────────────────────────────────

fn parse_fetch_line(line: &str) -> Option<FetchCmd> {
    let rest = line.strip_prefix("fetch ")?;
    let (oid, name) = rest.split_once(' ')?;
    Some(FetchCmd {
        oid: oid.to_string(),
        name: name.to_string(),
    })
}

fn parse_push_line(line: &str) -> Option<PushSpec> {
    let rest = line.strip_prefix("push ")?;
    let (force, rest) = match rest.strip_prefix('+') {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let (src, dst) = rest.split_once(':')?;
    Some(PushSpec {
        force,
        src: src.to_string(),
        dst: dst.to_string(),
    })
}

/// Read lines from `stdin` until a blank line or EOF, collecting those that
/// start with `prefix` via `parser`.
fn read_batch<T, F>(
    stdin: &mut impl BufRead,
    first: &str,
    prefix: &str,
    parser: F,
) -> Vec<T>
where
    F: Fn(&str) -> Option<T>,
{
    let mut items = Vec::new();
    if let Some(v) = parser(first) {
        items.push(v);
    }
    loop {
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let t = line.trim();
        if t.is_empty() {
            break;
        }
        if t.starts_with(prefix) {
            if let Some(v) = parser(t) {
                items.push(v);
            }
        }
    }
    items
}

// ── Protocol loop ──────────────────────────────────────────────────────────

/// Drive the git remote helper protocol until git closes stdin.
///
/// Reads commands from `stdin`, dispatches to `helper`, and writes responses
/// to `stdout`.  Diagnostics go to `stderr` (stdout must stay clean for git).
pub fn run_helper<H: RemoteHelper>(mut helper: H) -> Result<()> {
    let stdin_raw = io::stdin();
    let stdout_raw = io::stdout();

    let mut stdin = io::BufReader::new(stdin_raw.lock());
    let mut out = io::BufWriter::new(stdout_raw.lock());

    loop {
        let mut line = String::new();
        if stdin.read_line(&mut line).context("reading stdin")? == 0 {
            break; // EOF
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break; // blank line = done
        }

        // ── capabilities ─────────────────────────────────────────────────
        if trimmed == "capabilities" {
            for cap in helper.capabilities() {
                writeln!(out, "{cap}")?;
            }
            writeln!(out)?;
            out.flush()?;

        // ── list ──────────────────────────────────────────────────────────
        } else if trimmed == "list" || trimmed == "list for-push" {
            let for_push = trimmed.ends_with("for-push");
            match helper.list(for_push) {
                Ok(refs) => {
                    for r in refs {
                        if let Some(target) = r.symref_target {
                            writeln!(out, "@{target} {}", r.name)?;
                        } else {
                            writeln!(out, "{} {}", r.oid, r.name)?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[git-remote] list error: {e:#}");
                }
            }
            writeln!(out)?;
            out.flush()?;

        // ── fetch ─────────────────────────────────────────────────────────
        } else if trimmed.starts_with("fetch ") {
            let cmds = read_batch(&mut stdin, trimmed, "fetch ", parse_fetch_line);
            if let Err(e) = helper.fetch(cmds) {
                eprintln!("[git-remote] fetch error: {e:#}");
            }
            writeln!(out)?;
            out.flush()?;

        // ── push ──────────────────────────────────────────────────────────
        } else if trimmed.starts_with("push ") {
            let specs = read_batch(&mut stdin, trimmed, "push ", parse_push_line);
            match helper.push(specs) {
                Ok(results) => {
                    for r in results {
                        match r.result {
                            Ok(()) => writeln!(out, "ok {}", r.dst)?,
                            Err(e) => writeln!(out, "error {} {e}", r.dst)?,
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[git-remote] push error: {e:#}");
                }
            }
            writeln!(out)?;
            out.flush()?;

        // ── option ────────────────────────────────────────────────────────
        } else if trimmed.starts_with("option ") {
            // We don't act on any options currently; just acknowledge.
            writeln!(out, "unsupported")?;
            out.flush()?;
        }
    }

    Ok(())
}
