//! CLI for the v2 `perfect_ip` example binary (`fractal_swarm_perfect_ip_v2`).
//!
//! Mirrors v1's `fractal_ip.rs` flow on the v2 data plane and adds the
//! `p2p/TODO.md` gap-fill surface:
//!
//! - `--json` selects the JSON repair protocol for wire traffic; the default
//!   is the binary postcard codec on `/fractal/repair/1.1.0/postcard`.
//! - `--fetch <MULTIADDR>` requests every slice from a repair peer by id
//!   (repair-by-id is the natural loss path for a git transport) and
//!   reconstructs from the wire-received slices only.
//! - Verification runs `git fsck --strict` on the reconstructed repository
//!   before the clone step (TODO.md plan 6).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use log::log_enabled;
use log::Level;
use log::{debug, info};

use crate::fractal::fractal_ip_core_v2::{git_fsck_strict, request_slices_v2, WireCodecV2};
use gnostr_types::perfect_ip::sha256_hex;
use gnostr_types::perfect_ip_v2::{
    packetize_v2, reconstruct_directory_v2, run_directory_walk_v2, DirectoryWalkV2, FileEntryV2,
    IntegrityManagerV2,
};

#[derive(Debug, Parser)]
#[command(name = "fractal_swarm_perfect_ip_v2", arg_required_else_help = true)]
struct CliV2 {
    /// Walk the INPUT path as a directory tree and preserve relative paths.
    /// Requires --clone <INPUT> <OUTPUT>.
    #[arg(long, requires = "clone")]
    recursive: bool,

    /// Walk INPUT recursively, reconstruct to OUTPUT, and git clone OUTPUT to
    /// OUTPUT_repo.
    ///
    /// Requires exactly two paths: <INPUT> <OUTPUT>. If --recursive is given,
    /// the directory algorithm is used. Otherwise, if INPUT is a git
    /// repository the recursive algorithm is used; if it is a regular file the
    /// single-file algorithm is used automatically.
    #[arg(long, num_args = 2, value_name = "PATH")]
    clone: Option<Vec<PathBuf>>,

    /// Limit recursive directory walking to N levels.
    #[arg(long)]
    depth: Option<usize>,

    /// Enable verbose logging (optional level, defaults to "info").
    #[arg(long, num_args = 0..=1, default_missing_value = "info")]
    logging: Option<String>,

    /// Use the JSON repair protocol (/fractal/repair/2.0.0/json) for wire
    /// traffic instead of the default binary postcard codec
    /// (/fractal/repair/1.1.0/postcard).
    #[arg(long)]
    json: bool,

    /// Fetch every slice from a repair peer at <MULTIADDR> before
    /// reconstructing (repair-by-id). The manifest still comes from walking
    /// INPUT locally; the payload comes from the peer.
    #[arg(long, value_name = "MULTIADDR", requires = "clone")]
    fetch: Option<String>,

    /// Remove a previous OUTPUT and OUTPUT_repo before reconstructing, so a
    /// re-run does not fail on 'destination path already exists'.
    #[arg(long, requires = "clone")]
    force: bool,
}

fn git_output(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("spawn git");
    assert!(
        output.status.success(),
        "git -C {} {} failed",
        repo.display(),
        args.join(" ")
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_bare_git_repo(path: &Path) -> bool {
    Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--is-bare-repository"])
        .output()
        .map(|output| {
            output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "true"
        })
        .unwrap_or(false)
}

/// True only when `path` is itself a git repository the reconstructed tree
/// can be cloned from — its own `.git` (non-bare) or its own bare layout.
///
/// Unlike `is_git_repo`, this never matches a directory that merely lives
/// inside some parent repository: `git rev-parse --git-dir` succeeds for
/// those too, which would send a plain reconstructed directory into the
/// clone step to fail with `fatal: repository ... does not exist`.
fn is_self_contained_git_repo(path: &Path) -> bool {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let rev_parse = |arg: &str| {
        Command::new("git")
            .arg("-C")
            .arg(path)
            .args(["rev-parse", arg])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
    };
    if let Some(toplevel) = rev_parse("--show-toplevel") {
        // Non-bare: the work-tree root must be exactly `path`, not a parent.
        return Path::new(&toplevel)
            .canonicalize()
            .map(|resolved| resolved == canonical)
            .unwrap_or(false);
    }
    // Bare (e.g. a reconstructed `*.git` fixture): the git dir must be
    // `path` itself.
    rev_parse("--git-dir")
        .map(|git_dir| {
            let git_dir = Path::new(&git_dir);
            let resolved = if git_dir.is_absolute() {
                git_dir.to_path_buf()
            } else {
                path.join(git_dir)
            };
            resolved
                .canonicalize()
                .map(|resolved| resolved == canonical)
                .unwrap_or(false)
        })
        .unwrap_or(false)
}

fn with_git_extension(path: &Path) -> PathBuf {
    let s = path.as_os_str().to_string_lossy();
    if s.ends_with(".git") {
        path.to_path_buf()
    } else {
        let mut s = path.as_os_str().to_os_string();
        s.push(".git");
        PathBuf::from(s)
    }
}

/// Verify the reconstructed repository: `git fsck --strict` first (TODO.md
/// plan 6 — catches object-store corruption a clone might tolerate), then the
/// clone → HEAD-match → clean-tree flow.
fn fsck_clone_and_verify_reconstructed_repo(
    reconstructed: &Path,
    clone_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    git_fsck_strict(reconstructed)
        .map_err(|message| -> Box<dyn std::error::Error + Send + Sync> { message.into() })?;

    let status = Command::new("git")
        .arg("clone")
        .arg(reconstructed)
        .arg(clone_path)
        .status()?;
    if !status.success() {
        return Err(format!(
            "git clone from {} to {} failed",
            reconstructed.display(),
            clone_path.display()
        )
        .into());
    }

    let reconstructed_head = git_output(reconstructed, &["rev-parse", "HEAD"]);
    let clone_head = git_output(clone_path, &["rev-parse", "HEAD"]);
    if reconstructed_head != clone_head {
        return Err(format!(
            "clone HEAD mismatch: reconstructed={reconstructed_head}, clone={clone_head}"
        )
        .into());
    }

    let status_output = git_output(clone_path, &["status", "--porcelain"]);
    if !status_output.is_empty() {
        return Err(format!("cloned working tree is not clean:\n{status_output}").into());
    }

    println!(
        "verified git fsck --strict + clone: {} -> {} (HEAD {})",
        reconstructed.display(),
        clone_path.display(),
        clone_head
    );
    Ok(())
}

/// Build a [`DirectoryWalkV2`] whose packets come from `received` instead of
/// the original walk, dropping any file that has no slices in `received`.
fn walk_from_received(
    walk: &DirectoryWalkV2,
    received: &std::collections::HashMap<String, gnostr_types::perfect_ip_v2::ProtocolSliceV2>,
) -> DirectoryWalkV2 {
    let mut files = std::collections::BTreeMap::new();
    for (rel_path, entry) in &walk.files {
        let mut packets = Vec::new();
        let mut manifest = Vec::new();
        for manifest_entry in &entry.manifest {
            if let Some(slice) = received.get(&manifest_entry.id) {
                packets.push(slice.clone());
                manifest.push(manifest_entry.clone());
            }
        }
        if !packets.is_empty() {
            files.insert(rel_path.clone(), FileEntryV2 { packets, manifest });
        }
    }
    DirectoryWalkV2 {
        files,
        empty_dirs: walk.empty_dirs.clone(),
    }
}

pub async fn run_v2() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = CliV2::parse();

    if let Some(ref level) = cli.logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    }

    let depth = cli.depth.unwrap_or(3);

    let Some(ref clone) = cli.clone else {
        return Err("nothing to do: provide --clone <INPUT> <OUTPUT>".into());
    };
    if clone.len() != 2 {
        return Err("--clone requires exactly two paths: <INPUT> <OUTPUT>".into());
    }
    let input = &clone[0];
    let mut out = clone[1].clone();
    if is_bare_git_repo(input) {
        out = with_git_extension(&out);
    }

    let use_recursive = cli.recursive || is_git_repo(input);

    if cli.depth.is_some() && !use_recursive {
        return Err("--depth is only valid with --recursive".into());
    }

    let wire = if cli.json {
        WireCodecV2::Json
    } else {
        WireCodecV2::Binary
    };
    info!("v2 repair wire protocol: {wire:?}");

    if use_recursive {
        if !input.is_dir() {
            return Err(
                format!("--recursive input is not a directory: {}", input.display()).into(),
            );
        }

        let clone_dest = out.with_file_name(format!(
            "{}_repo",
            out.file_name().unwrap_or_default().to_string_lossy()
        ));

        if cli.force {
            // Clear previous runs: a stale OUTPUT/OUTPUT_repo makes git
            // clone fail with 'destination path already exists', and a
            // stale OUTPUT inside a walked tree would be re-ingested.
            for path in [&out, &clone_dest] {
                if !path.exists() {
                    continue;
                }
                if path == input {
                    return Err(format!(
                        "--force refuses to remove the input path {}",
                        path.display()
                    )
                    .into());
                }
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
                info!("--force: removed previous {}", path.display());
            }
        }

        info!("recursive walk root: {}", input.display());
        info!("recursive depth: {depth}");
        let walk = run_directory_walk_v2(input, depth, cli.logging.is_some())?;

        if let Some(ref peer) = cli.fetch {
            let addr: libp2p::Multiaddr = peer
                .parse()
                .map_err(|error| format!("invalid --fetch multiaddr {peer}: {error}"))?;
            let all_ids: Vec<String> = walk
                .manifest()
                .iter()
                .map(|entry| entry.id.clone())
                .collect();
            println!("fetching {} slices from {addr} ({wire:?})", all_ids.len());
            let fetch = request_slices_v2(addr, "fractal_v2_fetcher", all_ids, wire).await;
            println!(
                "fetched {} slices ({} explicit absences)",
                fetch.received.len(),
                fetch.errors.len()
            );
            for message in &fetch.errors {
                println!("absence: {message}");
            }

            let wire_walk = walk_from_received(&walk, &fetch.received);
            reconstruct_directory_v2(&wire_walk, &out)?;
            println!(
                "reconstructed directory from wire slices to {}",
                out.display()
            );
        } else {
            reconstruct_directory_v2(&walk, &out)?;
            println!("recursively reconstructed directory to {}", out.display());
        }

        if is_self_contained_git_repo(&out) {
            fsck_clone_and_verify_reconstructed_repo(&out, &clone_dest)?;
        } else {
            println!("reconstructed path is not a git repo; skipping fsck/clone verification");
        }
        return Ok(());
    }

    let original_bytes = if input.exists() && !input.is_dir() {
        fs::read(input)?
    } else {
        return Err(format!(
            "file input does not exist or is a directory: {}",
            input.display()
        )
        .into());
    };

    let original_sha256 = sha256_hex(&original_bytes);
    let result = packetize_v2("ROOT".to_string(), original_bytes.clone());

    println!("sender wrote {}", input.display());
    println!("sender sha256: {original_sha256}");
    if cli.logging.is_some() {
        info!(
            "perfect_ip v2 packet batch: {} packets",
            result.batch.total_packets
        );
    }
    if log_enabled!(Level::Debug) && !log_enabled!(Level::Trace) {
        debug!("manifest entries: {}", result.manifest.len());
    }

    let mut integrity = IntegrityManagerV2::new(result.manifest.clone());
    for slice in result.batch.packets.clone() {
        integrity.record_slice(slice);
    }
    assert!(integrity.rejected.is_empty(), "own slices must verify");
    println!("missing nodes: {:?}", integrity.get_missing_nodes());
    println!("integrity verified: {}", integrity.verify_integrity());

    let reconstructed = integrity
        .reconstruct_payload()
        .map_err(|error| format!("v2 reconstruction failed: {error}"))?;
    if cli.force && out.is_dir() {
        fs::remove_dir_all(&out)?;
    }
    let reconstructed_sha256 = sha256_hex(&reconstructed);
    fs::write(&out, &reconstructed)?;

    println!("peer reconstructed {}", out.display());
    println!("peer sha256: {reconstructed_sha256}");
    println!("sha256 match: {}", reconstructed_sha256 == original_sha256);

    println!("file reconstruction complete; exiting");
    Ok(())
}
