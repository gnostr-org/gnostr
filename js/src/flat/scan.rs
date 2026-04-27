use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub(crate) struct FileInfo {
    pub(crate) rel: String,
    pub(crate) size: u64,
    pub(crate) content: Option<String>,
}

pub(crate) fn clone_repo(tmp_dir: &Path, repo_url: &str) -> Result<PathBuf> {
    let repo_url = normalize_repo_url(repo_url);
    ensure_helper_support(&repo_url)?;

    println!("Cloning repository...");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", "--single-branch", "--no-tags", &repo_url, "repo"])
        .current_dir(tmp_dir)
        .status()
        .context("Git command failed")?;

    if !status.success() {
        anyhow::bail!("Failed to clone repository.");
    }

    Ok(tmp_dir.join("repo"))
}

fn normalize_repo_url(repo_url: &str) -> String {
    repo_url.strip_prefix("git+").unwrap_or(repo_url).to_string()
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
