use std::{
    env, fs,
    io,
    path::{Path, PathBuf},
    process::Command,
};

pub const UPSTREAM_URL: &str = "git@github.com:nostr-protocol/nips.git";

pub fn checkout_dir() -> PathBuf {
    if let Ok(path) = env::var("NIPS_REPO_DIR") {
        return PathBuf::from(path);
    }

    if let Ok(cache_home) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(cache_home).join("gnostr").join("nips");
    }

    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("gnostr").join("nips");
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".cache")
        .join("gnostr")
        .join("nips")
}

pub fn ensure_checkout() -> io::Result<PathBuf> {
    let dir = checkout_dir();

    if dir.join(".git").is_dir() {
        sync_checkout(&dir)?;
    } else if dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "checkout directory '{}' exists but is not a git repository",
                dir.display()
            ),
        ));
    } else {
        clone_checkout(&dir)?;
    }

    Ok(dir)
}

pub fn worktree_dirty(dir: &Path) -> io::Result<bool> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git status failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(!output.stdout.is_empty())
}

fn clone_checkout(dir: &Path) -> io::Result<()> {
    if let Some(parent) = dir.parent() {
        fs::create_dir_all(parent)?;
    }

    run_git(&["clone", "--quiet", "--depth", "1", UPSTREAM_URL], Some(dir.parent().unwrap_or(dir)), Some(dir))
}

fn sync_checkout(dir: &Path) -> io::Result<()> {
    run_git(
        &["remote", "set-url", "origin", UPSTREAM_URL],
        Some(dir),
        None,
    )?;
    run_git(&["fetch", "--prune", "origin"], Some(dir), None)?;
    if worktree_dirty(dir)? {
        return Ok(());
    }

    run_git(&["pull", "--ff-only", "--prune"], Some(dir), None)
}

fn run_git(args: &[&str], current_dir: Option<&Path>, clone_target: Option<&Path>) -> io::Result<()> {
    let mut command = Command::new("git");
    command.args(args);

    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }

    if let Some(target) = clone_target {
        command.arg(target);
    }

    let output = command.output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "git {:?} failed with exit code {:?}\nstdout: {}\nstderr: {}",
            args,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}
