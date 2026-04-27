use anyhow::{Context, Result};
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
    println!("Cloning repository...");
    let status = Command::new("git")
        .args(["clone", "--depth", "1", repo_url, "repo"])
        .current_dir(tmp_dir)
        .status()
        .context("Git command failed")?;

    if !status.success() {
        anyhow::bail!("Failed to clone repository.");
    }

    Ok(tmp_dir.join("repo"))
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
