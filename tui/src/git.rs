//! Git repository detection and operation primitives.

use std::path::PathBuf;

// ── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRepoKind {
    /// Contains a `.git` file or directory.
    Repo,
    /// No `.git`, but has `HEAD` + `objects/` + `refs/` at root — a bare clone.
    Bare,
}

/// In-progress git operation detected from sentinel files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitRepoState {
    Merging,
    Rebasing,
    CherryPicking,
    Reverting,
    Bisecting,
}

impl GitRepoState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Merging       => "MERGING",
            Self::Rebasing      => "REBASING",
            Self::CherryPicking => "CHERRY-PICK",
            Self::Reverting     => "REVERTING",
            Self::Bisecting     => "BISECTING",
        }
    }
}

/// Rich git metadata read from `.git/HEAD`, `.git/config`, and sentinel files.
/// Populated cheaply with pure `fs::read_to_string` — no subprocess.
#[derive(Debug, Clone)]
pub struct GitRepoInfo {
    pub kind:        GitRepoKind,
    /// Current branch name, or `"detached:<sha7>"` for detached HEAD.
    pub branch:      Option<String>,
    /// Name of the first remote found in `.git/config` (usually `"origin"`).
    pub remote_name: Option<String>,
    /// URL of `remote_name`.
    pub remote_url:  Option<String>,
    /// Non-`None` when a merge / rebase / cherry-pick / etc. is in progress.
    pub state:       Option<GitRepoState>,
}

/// Git operations available from the file browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitAction {
    Status,
    Pull,
    Push,
    Fetch,
    Log,
    Diff,
    Add,
    Commit,
}

// ── Detection ──────────────────────────────────────────────────────────────

/// Detect git repo kind and read metadata from the git directory.
/// Uses only `fs::read_to_string` / `is_file` / `is_dir` — no subprocess.
pub fn detect_git_info(dir: &std::path::Path) -> Option<GitRepoInfo> {
    let git_dir = if dir.join(".git").is_dir() {
        dir.join(".git")
    } else if dir.join(".git").is_file() {
        // worktree / submodule — parsing the gitfile is complex; mark as Repo
        return Some(GitRepoInfo {
            kind:        GitRepoKind::Repo,
            branch:      None,
            remote_name: None,
            remote_url:  None,
            state:       None,
        });
    } else if dir.join("HEAD").is_file()
        && dir.join("objects").is_dir()
        && dir.join("refs").is_dir()
    {
        dir.to_path_buf() // bare repo — config lives at root
    } else {
        return None;
    };

    let kind = if git_dir == dir {
        GitRepoKind::Bare
    } else {
        GitRepoKind::Repo
    };

    let branch = git_read_branch(&git_dir);
    let (remote_name, remote_url) = git_read_first_remote(&git_dir);
    let state = git_detect_state(&git_dir);

    Some(GitRepoInfo { kind, branch, remote_name, remote_url, state })
}

/// Read current branch from `<git_dir>/HEAD`.
fn git_read_branch(git_dir: &PathBuf) -> Option<String> {
    let raw = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let raw = raw.trim();
    if let Some(branch) = raw.strip_prefix("ref: refs/heads/") {
        Some(branch.to_owned())
    } else if raw.len() >= 7 {
        Some(format!("detached:{}", &raw[..7]))
    } else {
        None
    }
}

/// Parse the first `[remote "…"]` section from `<git_dir>/config`.
fn git_read_first_remote(git_dir: &PathBuf) -> (Option<String>, Option<String>) {
    let config =
        std::fs::read_to_string(git_dir.join("config")).unwrap_or_default();
    let mut name: Option<String> = None;
    let mut url: Option<String> = None;
    let mut in_remote = false;
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with("[remote \"") {
            if let Some(n) = line
                .strip_prefix("[remote \"")
                .and_then(|s| s.strip_suffix("\"]"))
            {
                name = Some(n.to_owned());
                in_remote = true;
            }
        } else if line.starts_with('[') {
            in_remote = false;
        } else if in_remote {
            if let Some(u) = line.strip_prefix("url = ") {
                url = Some(u.to_owned());
                break;
            }
        }
    }
    (name, url)
}

/// Check for sentinel files indicating an in-progress git operation.
fn git_detect_state(git_dir: &PathBuf) -> Option<GitRepoState> {
    if git_dir.join("MERGE_HEAD").is_file() {
        return Some(GitRepoState::Merging);
    }
    if git_dir.join("CHERRY_PICK_HEAD").is_file() {
        return Some(GitRepoState::CherryPicking);
    }
    if git_dir.join("REVERT_HEAD").is_file() {
        return Some(GitRepoState::Reverting);
    }
    if git_dir.join("rebase-merge").is_dir()
        || git_dir.join("rebase-apply").is_dir()
    {
        return Some(GitRepoState::Rebasing);
    }
    if git_dir.join("BISECT_LOG").is_file() {
        return Some(GitRepoState::Bisecting);
    }
    None
}

/// Compat shim — kept so any external callers still compile.
#[deprecated(note = "use detect_git_info instead")]
pub fn detect_git_repo(dir: &std::path::Path) -> Option<GitRepoKind> {
    detect_git_info(dir).map(|i| i.kind)
}

/// Walk `path` and its ancestors until a git root is found.
/// Returns `(root_path, GitRepoInfo)` for the nearest enclosing repo, or
/// `None` if `path` is not inside any git repository.
pub fn find_git_root(path: &std::path::Path) -> Option<(PathBuf, GitRepoInfo)> {
    let mut dir = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };
    loop {
        if let Some(info) = detect_git_info(&dir) {
            return Some((dir, info));
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => return None,
        }
    }
}

// ── FileBrowserEntry ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FileBrowserEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    /// Set when the entry is a directory that is (or contains) a git repo.
    pub git: Option<GitRepoInfo>,
}

impl FileBrowserEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let is_dir = path.is_dir();
        let git = if is_dir { detect_git_info(&path) } else { None };
        Self { name, path, is_dir, git }
    }
}

// ── Runner ─────────────────────────────────────────────────────────────────
/// Run a git sub-command inside `repo_dir` and return combined stdout+stderr.
pub async fn run_git_command(
    repo_dir: &std::path::Path,
    action: GitAction,
    commit_msg: &str,
) -> Result<String, String> {
    let args: &[&str] = match action {
        GitAction::Status => &["status"],
        GitAction::Pull => &["pull"],
        GitAction::Push => &["push"],
        GitAction::Fetch => &["fetch", "--all"],
        GitAction::Log => &["log", "--oneline", "--graph", "--decorate", "-20"],
        GitAction::Diff => &["diff"],
        GitAction::Add => &["add", "-A"],
        GitAction::Commit => &["commit", "-m", commit_msg],
    };

    let out = tokio::process::Command::new("git")
        .args(args)
        .current_dir(repo_dir)
        .output()
        .await
        .map_err(|e| format!("spawn git: {e}"))?;

    let mut combined = String::from_utf8_lossy(&out.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&out.stderr);
    if !stderr.is_empty() {
        if !combined.is_empty() {
            combined.push('\n');
        }
        combined.push_str(&stderr);
    }
    if combined.is_empty() {
        combined = "(no output)".into();
    }
    Ok(combined)
}
