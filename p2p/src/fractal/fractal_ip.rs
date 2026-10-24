use std::{fs, path::Path, path::PathBuf, process::Command};

use clap::Parser;
use log::{Level, debug, info, log_enabled, trace, warn};
// pub use crate::perfect_ip_core::*;
use crate::fractal::fractal_ip_core::{
    generate_manifest, packetize, reconstruct_directory, reconstruct_payload, run_directory_walk,
    sha256_hex, summarize_packets, IntegrityManager,
};

#[derive(Debug, Parser)]
#[command(name = "fractal_swarm_perfect_ip", arg_required_else_help = true)]
struct Cli {
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
}

fn clone_and_verify_reconstructed_repo(
    reconstructed: &Path,
    clone_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        "verified git clone: {} -> {} (HEAD {})",
        reconstructed.display(),
        clone_path.display(),
        clone_head
    );
    Ok(())
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

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();

    if let Some(ref level) = cli.logging {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(level)).init();
    }

    let depth = cli.depth.unwrap_or(3);
    let depth_set = cli.depth.is_some();

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

    if depth_set && !use_recursive {
        return Err("--depth is only valid with --recursive".into());
    }

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

        info!("recursive walk root: {}", input.display());

         // TODO debug logging
         if log_enabled!(Level::Debug) && !log_enabled!(Level::Trace) {
            // debug!("{line}");
         }
         // TODO else if log_enabled!(Level::Trace) {
         // TODO     //TODO info!("{line}");
         // TODO }
         else if log_enabled!(Level::Info) {
            // info!("{line}");
         }

        info!("recursive depth: {}", depth);
        info!("logging flag is: {:?}", cli.logging);
        let walk = run_directory_walk(input, depth, cli.logging.is_some())?;
        reconstruct_directory(walk, &out)?;
        println!("recursively reconstructed directory to {}", out.display());
        clone_and_verify_reconstructed_repo(&out, &clone_dest)?;
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
    let batch = packetize("ROOT".to_string(), original_bytes.clone());
    let manifest = generate_manifest("ROOT".to_string(), original_bytes.len());

    println!("sender wrote {}", input.display());
    println!("sender sha256: {original_sha256}");
    if cli.logging.is_some() {
        info!("perfect_ip packet batch: {} packets", batch.total_packets);
        for line in summarize_packets(&batch.packets) {
            debug!("{line}");
        }
    }

    let mut integrity = IntegrityManager::new(manifest.clone());
    for slice in batch.packets.clone() {
        integrity.record_slice(slice);
    }
    println!("missing nodes: {:?}", integrity.get_missing_nodes());
    println!("integrity verified: {}", integrity.verify_integrity());

    let reconstructed = reconstruct_payload(&batch.packets);
    let reconstructed_sha256 = sha256_hex(&reconstructed);
    fs::write(&out, &reconstructed)?;

    println!("peer reconstructed {}", out.display());
    println!("peer sha256: {reconstructed_sha256}");
    println!("sha256 match: {}", reconstructed_sha256 == original_sha256);

    println!("file reconstruction complete; exiting");
    Ok(())
}
