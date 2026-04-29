use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub(crate) struct ClonedRepo {
    pub(crate) path: PathBuf,
    pub(crate) short_hash: String,
}

pub(crate) struct FileInfo {
    pub(crate) rel: String,
    pub(crate) size: u64,
    pub(crate) content: Option<String>,
}

pub(crate) fn clone_repo(tmp_dir: &Path, repo_url: &str) -> Result<ClonedRepo> {
    let repo_url = normalize_repo_url(repo_url);
    ensure_helper_support(&repo_url)?;

    println!("Cloning repository...");
    let status = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env(
            "GIT_SSH_COMMAND",
            "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new",
        )
        .args(["clone", "--depth", "1", "--single-branch", "--no-tags", &repo_url, "repo"])
        .current_dir(tmp_dir)
        .status()
        .context("Git command failed")?;

    if !status.success() {
        anyhow::bail!("Failed to clone repository.");
    }

    let path = tmp_dir.join("repo");
    let short_hash = git_short_hash(&path).unwrap_or_else(|_| "unknown".to_string());

    Ok(ClonedRepo { path, short_hash })
}

pub(crate) fn checkout_ref(repo_path: &Path, git_ref: &str) -> Result<()> {
    let status = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args(["-C", repo_path.to_string_lossy().as_ref(), "checkout", "--force", git_ref])
        .status()
        .context("Git checkout command failed")?;

    if status.success() {
        return Ok(());
    }

    anyhow::bail!("Failed to checkout ref '{git_ref}'");
}

pub(crate) fn probe_repo(repo_url: &str, git_ref: Option<&str>) -> Result<()> {
    let repo_url = normalize_repo_url(repo_url);
    ensure_helper_support(&repo_url)?;

    let mut args = vec!["ls-remote", "--exit-code", &repo_url];
    if let Some(git_ref) = git_ref.filter(|value| !value.is_empty()) {
        args.push(git_ref);
    }

    let output = Command::new("git")
        .env("GIT_TERMINAL_PROMPT", "0")
        .env(
            "GIT_SSH_COMMAND",
            "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new",
        )
        .args(args)
        .output()
        .context("Git ls-remote command failed")?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = match (stderr.is_empty(), stdout.is_empty()) {
        (true, true) => "repository is not reachable".to_string(),
        (false, true) => stderr,
        (true, false) => stdout,
        (false, false) => format!("{stderr}: {stdout}"),
    };
    anyhow::bail!("Flat precheck failed: {detail}");
}

fn normalize_repo_url(repo_url: &str) -> String {
    let repo_url = repo_url.strip_prefix("git+").unwrap_or(repo_url);
    if let Some(normalized) = normalize_scp_style_url(repo_url) {
        return normalized;
    }
    repo_url.to_string()
}

fn normalize_scp_style_url(repo_url: &str) -> Option<String> {
    if repo_url.contains("://") {
        return None;
    }

    let (left, right) = repo_url.split_once(':')?;
    if left.is_empty()
        || right.is_empty()
        || left.contains('/')
        || right.starts_with('/')
        || right.starts_with('\\')
    {
        return None;
    }

    Some(format!("ssh://{left}/{right}"))
}

fn ensure_helper_support(repo_url: &str) -> Result<()> {
    if let Some(helper) = helper_binary_for_url(repo_url) {
        if !command_in_path(helper) {
            anyhow::bail!(
                "missing git helper '{helper}' for '{repo_url}'. install the helper or use a plain https/git clone url"
            );
        }
    }
    Ok(())
}

fn helper_binary_for_url(repo_url: &str) -> Option<&'static str> {
    const MAP: [(&str, &str); 9] = [
        ("nostr+wss://", "git-remote-nostr"),
        ("nostr+ws://", "git-remote-nostr"),
        ("nostr://", "git-remote-nostr"),
        ("pkarr://", "git-remote-pkarr"),
        ("ipfs://", "git-remote-ipfs"),
        ("blossom+onion://", "git-remote-tor"),
        ("tor://", "git-remote-tor"),
        ("blossom://", "git-remote-blossom"),
        ("blossom+https://", "git-remote-blossom"),
    ];

    MAP.iter()
        .find_map(|(prefix, helper)| repo_url.starts_with(prefix).then_some(*helper))
}

fn command_in_path(command: &str) -> bool {
    let Some(path) = env::var_os("PATH") else {
        return false;
    };

    for dir in env::split_paths(&path) {
        let candidate = dir.join(command);
        if candidate.is_file() {
            return true;
        }
        #[cfg(windows)]
        {
            let candidate = dir.join(format!("{command}.exe"));
            if candidate.is_file() {
                return true;
            }
        }
    }

    false
}

fn git_short_hash(repo_path: &Path) -> Result<String> {
    let out = Command::new("git")
        .args(["-C", repo_path.to_string_lossy().as_ref(), "rev-parse", "--short", "HEAD"])
        .output()
        .context("git rev-parse --short HEAD")?;

    if !out.status.success() {
        anyhow::bail!("git rev-parse --short HEAD failed");
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub(crate) fn collect_files(repo_path: &Path, max_bytes: u64) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(repo_path).sort_by_file_name() {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let rel = path
            .strip_prefix(repo_path)?
            .to_string_lossy()
            .replace('\\', "/");

        if rel.starts_with(".git/") {
            continue;
        }

        let size = entry.metadata()?.len();
        let mut content = None;

        if size <= max_bytes && !is_binary(path) {
            content = fs::read_to_string(path).ok();
        }

        files.push(FileInfo { rel, size, content });
    }

    Ok(files)
}

fn is_binary(path: &Path) -> bool {
    fs::read(path)
        .map(|b| b.iter().take(4096).any(|&x| x == 0))
        .unwrap_or(true)
}
