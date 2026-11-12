//! Async Git integration helpers (labtui-asyncgit).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, unbounded};
// TODO avoid the circular dep
use gnostr_asyncgit::{
    AsyncDiff, AsyncGitNotification, AsyncStatus, DiffParams, DiffType, FileDiff, StatusParams,
    sync::RepoPath,
    sync::diff::DiffOptions,
    sync::status::{StatusItem, StatusItemType, StatusType},
};

/// Container for async git state.
pub struct GitState {
    pub repo_path: RepoPath,
    pub async_status: AsyncStatus,
    pub async_diff: AsyncDiff,
    pub rx: Receiver<AsyncGitNotification>,
    pub tx: Sender<AsyncGitNotification>,
    pub status_items: Vec<StatusItem>,
    pub diff: Option<(DiffParams, FileDiff)>,
    pub diff_path: Option<PathBuf>,
    pub blame: Option<String>,
    pub blame_path: Option<PathBuf>,
    last_status: Instant,
}

impl GitState {
    /// Try to create a git state for the current working directory.
    pub fn new() -> Option<Self> {
        let inside = std::process::Command::new("git")
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !inside {
            return None;
        }

        let repo_path = RepoPath::Path(PathBuf::from("."));
        let (tx, rx) = unbounded();
        let async_status = AsyncStatus::new(repo_path.clone(), tx.clone());
        let async_diff = AsyncDiff::new(repo_path.clone(), &tx);

        let mut state = Self {
            repo_path,
            async_status,
            async_diff,
            rx,
            tx,
            status_items: Vec::new(),
            diff: None,
            diff_path: None,
            blame: None,
            blame_path: None,
            last_status: Instant::now(),
        };

        state.fetch_status();
        Some(state)
    }

    pub fn poll(&mut self) {
        while let Ok(notif) = self.rx.try_recv() {
            match notif {
                AsyncGitNotification::Status => {
                    if let Ok(status) = self.async_status.last() {
                        self.status_items = status.items;
                    }
                }
                AsyncGitNotification::Diff => {
                    if let Ok(Some(diff)) = self.async_diff.last() {
                        self.diff = Some(diff);
                    }
                }
                _ => {}
            }
        }

        // Periodically refresh status.
        if self.last_status.elapsed() >= Duration::from_secs(5) {
            self.fetch_status();
            self.last_status = Instant::now();
        }
    }

    pub fn fetch_status(&mut self) {
        let params = StatusParams::new(StatusType::WorkingDir, None);
        let _ = self.async_status.fetch(&params);
    }

    pub fn fetch_diff(&mut self, path: &std::path::Path) {
        self.diff_path = Some(path.to_path_buf());
        let params = DiffParams {
            path: path.to_string_lossy().to_string(),
            diff_type: DiffType::WorkDir,
            options: DiffOptions::default(),
        };
        let _ = self.async_diff.request(params);
    }

    /// Synchronously fetch `git blame` for the given file.
    pub fn fetch_blame(&mut self, path: &std::path::Path) {
        self.blame_path = Some(path.to_path_buf());
        let output = std::process::Command::new("git")
            .args(["blame", "--", &path.to_string_lossy()])
            .output();
        self.blame = output.ok().and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        });
    }

    pub fn status_for(&self, path: &std::path::Path) -> Option<StatusItemType> {
        let needle = path.to_string_lossy().to_string();
        self.status_items
            .iter()
            .find(|item| item.path == needle || path.ends_with(&item.path))
            .map(|item| item.status)
    }
}
