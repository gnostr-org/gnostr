use std::{
    collections::HashSet,
    io,
    path::PathBuf,
    time::{Duration, Instant},
};

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use gnostr_asyncgit::git2;
use gnostr_asyncgit::types::{
    get_leading_zero_bits,
    nip34::{Nip34Event, Nip34UnsignedEvent, RepoRef, RepoState},
    EventKind, PrivateKey, PublicKey, TagV3, Unixtime, UncheckedUrl,
};
use gnostr_asyncgit::sync::{AccumulatedPowSummary, accumulated_pow, RepoPath};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
enum Nip34Error {
    #[error("no secret key provided; use --secret-key, set GNOSTR_SECRET_KEY, or provide a config file")]
    MissingSecretKey,
    #[error("invalid secret key: {0}")]
    InvalidSecretKey(String),
    #[error("invalid public key '{key}': {error}")]
    InvalidPublicKey { key: String, error: String },
    #[error("failed to read config file {path}: {source}")]
    ConfigRead { path: PathBuf, source: io::Error },
    #[error("failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("failed to build NIP-34 event: {0}")]
    EventBuild(String),
    #[error("terminal error: {0}")]
    Terminal(#[from] io::Error),
    #[error("patch creation failed: {0}")]
    PatchCreation(String),
    #[error("invalid maintainer key '{0}'")]
    InvalidMaintainer(String),
    #[error("publish failed: {0}")]
    Publish(String),
}

type Result<T> = std::result::Result<T, Nip34Error>;

#[derive(Parser, Debug)]
#[command(name = "gnostr-nip34", about = "NIP-34 git navigator")]
struct Cli {
    /// Secret key as hex or nsec1... bech32 string.
    #[arg(long, short = 'k', env = "GNOSTR_SECRET_KEY")]
    secret_key: Option<String>,

    /// Path to a TOML config file.
    #[arg(long, short = 'c')]
    config: Option<PathBuf>,

    /// Repository identifier.
    #[arg(long)]
    repo_identifier: Option<String>,

    /// Repository reference slug, e.g. "owner/name" used in NIP-34 repository tags.
    #[arg(long)]
    repo_reference: Option<String>,

    /// Repository name.
    #[arg(long)]
    repo_name: Option<String>,

    /// Repository description.
    #[arg(long)]
    repo_description: Option<String>,

    /// Maintainer public keys (hex or npub), comma-separated.
    #[arg(long, value_delimiter = ',')]
    maintainers: Vec<String>,

    /// Default relays, comma-separated.
    #[arg(long, value_delimiter = ',')]
    relays: Option<Vec<String>>,

    /// Git server URLs, comma-separated.
    #[arg(long, value_delimiter = ',')]
    git_server: Option<Vec<String>>,

    /// Web URLs, comma-separated.
    #[arg(long, value_delimiter = ',')]
    web: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct AppConfig {
    secret_key: Option<String>,
    #[serde(default)]
    repo: RepoConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct RepoConfig {
    identifier: Option<String>,
    reference: Option<String>,
    name: Option<String>,
    description: Option<String>,
    maintainers: Option<Vec<String>>,
    relays: Option<Vec<String>>,
    git_server: Option<Vec<String>>,
    web: Option<Vec<String>>,
}

fn default_config_path() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| dirs.config_dir().join("gnostr").join("nip34.toml"))
}

fn load_config(path: Option<&PathBuf>) -> Result<AppConfig> {
    match path {
        Some(path) => {
            let contents = std::fs::read_to_string(path).map_err(|e| Nip34Error::ConfigRead {
                path: path.clone(),
                source: e,
            })?;
            Ok(toml::from_str(&contents)?)
        }
        None => Ok(AppConfig::default()),
    }
}

fn resolve_private_key(cli: &Cli, config: &AppConfig) -> Result<PrivateKey> {
    let key_str = cli
        .secret_key
        .clone()
        .or_else(|| config.secret_key.clone())
        .ok_or(Nip34Error::MissingSecretKey)?;

    PrivateKey::try_from_bech32_string(&key_str)
        .or_else(|_| PrivateKey::try_from_hex_string(&key_str))
        .map_err(|e| Nip34Error::InvalidSecretKey(e.to_string()))
}

fn parse_public_key(s: &str) -> Result<PublicKey> {
    PublicKey::try_from_bech32_string(s, false)
        .or_else(|_| PublicKey::try_from_hex_string(s, false))
        .map_err(|e| Nip34Error::InvalidPublicKey {
            key: s.to_string(),
            error: e.to_string(),
        })
}

fn resolve_maintainers(
    cli: &[String],
    config: &Option<Vec<String>>,
) -> Result<Vec<PublicKey>> {
    let sources: Vec<&String> = if !cli.is_empty() {
        cli.iter().collect()
    } else {
        config.as_ref().map(|v| v.iter().collect()).unwrap_or_default()
    };
    sources
        .into_iter()
        .map(|s| {
            parse_public_key(s).map_err(|e| {
                Nip34Error::InvalidMaintainer(format!("{s}: {e}"))
            })
        })
        .collect()
}

fn resolve_repo_config(cli: &Cli, config: &AppConfig, public_key: PublicKey) -> Result<RepoRef> {
    let repo = &config.repo;

    let identifier = cli
        .repo_identifier
        .clone()
        .or_else(|| repo.identifier.clone())
        .unwrap_or_else(|| "gnostr".to_string());
    let name = cli
        .repo_name
        .clone()
        .or_else(|| repo.name.clone())
        .unwrap_or_else(|| "gnostr".to_string());
    let description = cli
        .repo_description
        .clone()
        .or_else(|| repo.description.clone())
        .unwrap_or_else(|| "A git implementation on nostr".to_string());
    let git_server = cli
        .git_server
        .clone()
        .or_else(|| repo.git_server.clone())
        .unwrap_or_else(|| vec!["https://github.com/gnostr-org/gnostr.git".to_string()]);
    let web = cli
        .web
        .clone()
        .or_else(|| repo.web.clone())
        .unwrap_or_else(|| vec!["https://github.com/gnostr-org/gnostr".to_string()]);
    let relays: Vec<_> = cli
        .relays
        .clone()
        .or_else(|| repo.relays.clone())
        .unwrap_or_else(|| vec!["wss://relay.damus.io".to_string()])
        .iter()
        .map(|r| UncheckedUrl::from_str(r))
        .collect();

    let mut maintainers = resolve_maintainers(&cli.maintainers, &repo.maintainers)?;
    if maintainers.is_empty() {
        maintainers.push(public_key);
    }

    let trusted_maintainer = maintainers.first().copied().unwrap_or(public_key);

    Ok(RepoRef {
        name,
        description,
        identifier: identifier.clone(),
        root_commit: String::new(),
        git_server,
        web,
        relays,
        hashtags: vec![identifier],
        maintainers,
        trusted_maintainer,
        events: std::collections::HashMap::new(),
    })
}

/// Represents a relevant subset of a Git commit's data.
#[derive(Debug, Clone)]
struct Commit {
    hash: String,
    full_hash: String,
    author: String,
    summary: String,
    committer_date: String,
}

/// Represents a Git branch's data.
#[derive(Debug, Clone)]
struct Branch {
    name: String,
    commit_hash: String,
    commit_message: String,
    author: String,
    is_current: bool,
    is_remote: bool,
}

/// Navigation modes for the application.
#[derive(Debug, Clone, PartialEq)]
enum NavigatorMode {
    Commits,
    Branches,
    Nip34Events,
}

fn kind_label(kind: EventKind) -> &'static str {
    match kind {
        EventKind::RepositoryAnnouncement => "Repo Announcement",
        EventKind::GitRepoAnnouncement => "Repo State",
        EventKind::Patches => "Patch",
        EventKind::GitIssue => "Issue",
        EventKind::GitReply => "Reply",
        EventKind::GitStatusOpen => "Status Open",
        EventKind::GitStatusApplied => "Status Applied",
        EventKind::GitStatusClosed => "Status Closed",
        EventKind::GitStatusDraft => "Status Draft",
        _ => "Unknown",
    }
}

fn build_sample_event(
    kind: EventKind,
    tags: Vec<TagV3>,
    content: String,
    private_key: &PrivateKey,
) -> Result<Nip34Event> {
    let preevent = Nip34UnsignedEvent {
        pubkey: private_key.public_key(),
        created_at: Unixtime::now(),
        kind,
        tags,
        content,
    };
    Nip34Event::sign_with_private_key(preevent, private_key)
        .map_err(|e| Nip34Error::EventBuild(e.to_string()))
}

/// Publish a signed NIP-34 event to the provided relays over WebSocket.
async fn publish_event_to_relays(event: &Nip34Event, relays: &[String]) -> Vec<String> {
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let payload = serde_json::json!(["EVENT", event]).to_string();
    let mut results = Vec::new();

    for url in relays {
        let outcome = match tokio_tungstenite::connect_async(url).await {
            Ok((mut ws, _)) => {
                if let Err(e) = ws.send(Message::Text(payload.clone().into())).await {
                    format!("{url}: send failed: {e}")
                } else {
                    match tokio::time::timeout(Duration::from_secs(3), ws.next()).await {
                        Ok(Some(Ok(Message::Text(resp)))) => format!("{url}: {resp}"),
                        Ok(Some(Ok(msg))) => format!("{url}: {msg:?}"),
                        Ok(Some(Err(e))) => format!("{url}: read error: {e}"),
                        Ok(None) => format!("{url}: closed without response"),
                        Err(_) => format!("{url}: timeout waiting for OK"),
                    }
                }
            }
            Err(e) => format!("{url}: connect failed: {e}"),
        };
        results.push(outcome);
    }

    results
}

/// Collect extra syndication relays from the crawler bootstrap list and the
/// p2p crawler relay buckets. Available only when the `nip34-syndicate`
/// feature is enabled.
#[cfg(feature = "nip34-syndicate")]
fn collect_syndication_relays() -> Vec<String> {
    use std::collections::HashSet;

    let mut relays: HashSet<String> = gnostr_crawler::relays::bootstrap_relays()
        .into_iter()
        .collect();

    match gnostr_p2p::crawler_broadcast::load_crawler_relay_buckets() {
        Ok(buckets) => {
            for bucket in buckets {
                for relay in bucket.relays {
                    let _ = relays.insert(relay);
                }
            }
        }
        Err(err) => {
            tracing::warn!("failed to load p2p crawler relay buckets: {err}");
        }
    }

    relays.into_iter().collect()
}

/// The main application state.
struct App {
    commits: Vec<Commit>,
    branches: Vec<Branch>,
    nip34_events: Vec<Nip34Event>,
    commit_state: ListState,
    branch_state: ListState,
    nip34_state: ListState,
    current_mode: NavigatorMode,
    repo: git2::Repository,
    selected_commits: HashSet<usize>,
    selected_nip34_events: HashSet<usize>,
    full_commit_details: Option<String>,
    show_full_commit: bool,
    private_key: PrivateKey,
    _public_key: PublicKey,
    repo_reference: String,
    relays: Vec<String>,
    pow_summary: AccumulatedPowSummary,
    status_message: Option<String>,
    error_message: Option<String>,
    show_help: bool,
    popup_message: Option<String>,
}

impl App {
    /// Constructs a new App with git data and NIP-34 support.
    fn new(cli: &Cli, config: &AppConfig, private_key: PrivateKey) -> Result<Self> {
        let repo = git2::Repository::open_from_env()?;

        // Load commits (same as original)
        let mut revwalk = repo.revwalk()?;
        revwalk.push_head()?;

        let commits: Vec<Commit> = revwalk
            .filter_map(|id| id.ok())
            .filter_map(|oid| repo.find_commit(oid).ok())
            .take(100)
            .map(|commit| {
                let author = commit.author();
                let time = commit.committer().when();
                let date = time.seconds();
                let datetime = chrono::DateTime::from_timestamp(date, 0)
                    .map(|dt| dt.naive_local())
                    .unwrap_or_default();
                let committer_date = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                let full_hash = commit.id().to_string();
                let hash = full_hash.chars().take(8).collect::<String>();
                let summary = commit
                    .summary()
                    .ok()
                    .flatten()
                    .unwrap_or_default()
                    .to_string();

                Commit {
                    hash,
                    full_hash,
                    author: author.name().unwrap_or("Unknown").to_string(),
                    summary,
                    committer_date,
                }
            })
            .collect();

        // Load branches (same as original)
        let mut branches = Vec::new();

        for branch_ref in repo.branches(Some(git2::BranchType::Local))? {
            let (branch, _) = branch_ref?;
            if let Some(branch_name) = branch.name()? {
                let branch_ref_name = format!("refs/heads/{}", branch_name);
                if let Ok(reference) = repo.find_reference(&branch_ref_name) {
                    if let Some(commit) = reference.peel_to_commit().ok() {
                        let is_current = if let Ok(head) = repo.head() {
                            head.name().ok() == Some(branch_ref_name.as_str())
                        } else {
                            false
                        };

                        let author = commit.author();
                        branches.push(Branch {
                            name: branch_name.to_string(),
                            commit_hash: commit.id().to_string().chars().take(8).collect(),
                            commit_message: commit
                                .summary()
                                .ok()
                                .flatten()
                                .unwrap_or_default()
                                .to_string(),
                            author: author.name().unwrap_or("Unknown").to_string(),
                            is_current,
                            is_remote: false,
                        });
                    }
                }
            }
        }

        let repo_path = repo
            .workdir()
            .or_else(|| Some(repo.path()))
            .and_then(|p| p.to_str())
            .map(RepoPath::from)
            .ok_or_else(|| {
                Nip34Error::Git(git2::Error::from_str(
                    "cannot determine repository path",
                ))
            })?;

        let pow_summary = if commits.is_empty() {
            AccumulatedPowSummary::default()
        } else {
            accumulated_pow(
                &repo_path,
                &format!("HEAD...HEAD~{}", commits.len()),
                None,
            )
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to compute accumulated PoW: {e}");
                AccumulatedPowSummary::default()
            })
        };

        let public_key = private_key.public_key();

        let repo_reference = cli
            .repo_reference
            .clone()
            .or_else(|| config.repo.reference.clone())
            .unwrap_or_else(|| "gnostr-org/gnostr".to_string());

        let mut nip34_events = vec![];
        let mut startup_warnings = Vec::new();
        let mut repo_ref = resolve_repo_config(cli, config, public_key)?;
        let relays: Vec<String> = repo_ref
            .relays
            .iter()
            .map(|url| url.to_string())
            .collect();
        repo_ref.root_commit = commits
            .first()
            .map(|commit| commit.full_hash.clone())
            .unwrap_or_default();

        match repo_ref.to_event(&private_key) {
            Ok(event) => nip34_events.push(event),
            Err(e) => {
                let msg = format!("Failed to build repo announcement event: {e}");
                tracing::warn!("{}", msg);
                startup_warnings.push(msg);
            }
        }

        let mut state = std::collections::HashMap::new();
        if let Some(commit) = commits.first() {
            state.insert("refs/heads/main".to_string(), commit.full_hash.clone());
        }
        match RepoState::build(repo_ref.identifier.clone(), state, &private_key) {
            Ok(repo_state) => nip34_events.push(repo_state.event),
            Err(e) => {
                let msg = format!("Failed to build repo state event: {e}");
                tracing::warn!("{}", msg);
                startup_warnings.push(msg);
            }
        }

        let patch_event = build_sample_event(
            EventKind::from(1617),
            vec![
                TagV3::new_identifier("fix-auth-bug".to_string()),
                TagV3::new_tag("repository", &repo_reference),
            ],
            "Fix critical authentication bug in asyncgit NIP-34 implementation".to_string(),
            &private_key,
        )?;
        nip34_events.push(patch_event);

        let mut commit_state = ListState::default();
        if !commits.is_empty() {
            commit_state.select(Some(0));
        }

        let mut branch_state = ListState::default();
        if !branches.is_empty() {
            branch_state.select(Some(0));
        }

        let mut nip34_state = ListState::default();
        if !nip34_events.is_empty() {
            nip34_state.select(Some(0));
        }

        Ok(Self {
            commits,
            branches,
            nip34_events,
            commit_state,
            branch_state,
            nip34_state,
            current_mode: NavigatorMode::Commits,
            repo,
            selected_commits: HashSet::new(),
            selected_nip34_events: HashSet::new(),
            full_commit_details: None,
            show_full_commit: false,
            private_key,
            _public_key: public_key,
            repo_reference,
            relays,
            pow_summary,
            status_message: None,
            error_message: startup_warnings.into_iter().next(),
            show_help: false,
            popup_message: None,
        })
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.error_message = None;
    }

    fn set_error(&mut self, msg: impl Into<String>) {
        self.error_message = Some(msg.into());
        self.status_message = None;
    }

    fn clear_messages(&mut self) {
        self.status_message = None;
        self.error_message = None;
        self.popup_message = None;
    }

    fn show_popup(&mut self, msg: impl Into<String>) {
        self.popup_message = Some(msg.into());
    }

    fn dismiss_popup(&mut self) {
        self.popup_message = None;
        self.show_help = false;
    }

    /// Switches between navigation modes.
    fn switch_mode(&mut self) {
        self.current_mode = match self.current_mode {
            NavigatorMode::Commits => NavigatorMode::Branches,
            NavigatorMode::Branches => NavigatorMode::Nip34Events,
            NavigatorMode::Nip34Events => NavigatorMode::Commits,
        };
    }

    /// Sets the navigation mode.
    fn set_mode(&mut self, mode: NavigatorMode) {
        self.current_mode = mode;
    }

    /// Moves the selection up in the current mode.
    fn previous(&mut self) {
        let state = match self.current_mode {
            NavigatorMode::Commits => &mut self.commit_state,
            NavigatorMode::Branches => &mut self.branch_state,
            NavigatorMode::Nip34Events => &mut self.nip34_state,
        };

        let items = match self.current_mode {
            NavigatorMode::Commits => self.commits.len(),
            NavigatorMode::Branches => self.branches.len(),
            NavigatorMode::Nip34Events => self.nip34_events.len(),
        };

        if items == 0 {
            return;
        }

        let i = match state.selected() {
            Some(i) => {
                if i == 0 {
                    items - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    /// Moves the selection down in the current mode.
    fn next(&mut self) {
        let state = match self.current_mode {
            NavigatorMode::Commits => &mut self.commit_state,
            NavigatorMode::Branches => &mut self.branch_state,
            NavigatorMode::Nip34Events => &mut self.nip34_state,
        };

        let items = match self.current_mode {
            NavigatorMode::Commits => self.commits.len(),
            NavigatorMode::Branches => self.branches.len(),
            NavigatorMode::Nip34Events => self.nip34_events.len(),
        };

        if items == 0 {
            return;
        }

        let i = match state.selected() {
            Some(i) => {
                if i >= items - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        state.select(Some(i));
    }

    /// Toggles selection of current commit (max 2 commits).
    /// Automatically shows diff when exactly 2 commits are selected.
    fn toggle_commit_selection(&mut self) {
        if let Some(selected_index) = self.commit_state.selected() {
            if self.selected_commits.contains(&selected_index) {
                self.selected_commits.remove(&selected_index);
            } else if self.selected_commits.len() < 2 {
                self.selected_commits.insert(selected_index);

                // Auto-show diff when exactly 2 commits are selected
                if self.selected_commits.len() == 2 {
                    if let Err(e) = self.load_full_commit() {
                        self.set_error(format!("{e}"));
                    }
                }
            }
            // If we already have 2 selected and trying to add a third,
            // replace the oldest selection and auto-show new diff
            else if self.selected_commits.len() >= 2 {
                let mut indices: Vec<_> = self.selected_commits.iter().cloned().collect();
                indices.sort();
                self.selected_commits.remove(&indices[0]); // Remove oldest
                self.selected_commits.insert(selected_index);

                // Auto-show diff for new selection
                if let Err(e) = self.load_full_commit() {
                    self.set_error(format!("{e}"));
                }
            }
        }
    }

    /// Toggles selection of NIP-34 events.
    fn toggle_nip34_selection(&mut self) {
        if let Some(selected_index) = self.nip34_state.selected() {
            if self.selected_nip34_events.contains(&selected_index) {
                self.selected_nip34_events.remove(&selected_index);
            } else {
                self.selected_nip34_events.insert(selected_index);
            }
        }
    }

    /// Clears all selected commits/events.
    fn clear_selection(&mut self) {
        self.selected_commits.clear();
        self.selected_nip34_events.clear();
    }

    /// Returns number of selected items in current mode.
    fn selected_count(&self) -> usize {
        match self.current_mode {
            NavigatorMode::Commits => self.selected_commits.len(),
            NavigatorMode::Branches => 0,
            NavigatorMode::Nip34Events => self.selected_nip34_events.len(),
        }
    }

    /// Creates NIP-34 event from selected commits.
    fn create_nip34_patch_event(&mut self) -> Result<()> {
        if self.selected_commits.len() != 2 {
            return Err(Nip34Error::PatchCreation(
                "Need exactly 2 commits selected to create patch".into(),
            ));
        }

        let mut indices: Vec<_> = self.selected_commits.iter().cloned().collect();
        indices.sort();

        if let (Some(from_index), Some(to_index)) = (indices.get(0), indices.get(1)) {
            if let (Some(from_commit), Some(to_commit)) =
                (self.commits.get(*from_index), self.commits.get(*to_index))
            {
                let event = {
                    let from_oid = git2::Oid::from_str(&from_commit.full_hash)?;
                    let to_oid = git2::Oid::from_str(&to_commit.full_hash)?;

                    let from_commit_obj = self.repo.find_commit(from_oid)?;
                    let to_commit_obj = self.repo.find_commit(to_oid)?;

                    let from_tree = from_commit_obj.tree()?;
                    let to_tree = to_commit_obj.tree()?;

                    let diff = self
                        .repo
                        .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), None)?;

                    let mut patch = String::new();
                    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                        patch.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
                        true
                    })?;

                    build_sample_event(
                        EventKind::from(1617),
                        vec![
                            TagV3::new_identifier(format!(
                                "{}..{}",
                                from_commit.full_hash, to_commit.full_hash
                            )),
                            TagV3::new_tag("repository", &self.repo_reference),
                        ],
                        patch,
                        &self.private_key,
                    )?
                };

                self.nip34_events.push(event);
                self.clear_selection();

                Ok(())
            } else {
                Err(Nip34Error::PatchCreation("Invalid commit selection".into()))
            }
        } else {
            Err(Nip34Error::PatchCreation("No commits selected".into()))
        }
    }

    /// Republishes the selected NIP-34 event.
    fn republish_nip34_event(&mut self) -> Result<()> {
        if let Some(selected_index) = self.nip34_state.selected() {
            if let Some(event_to_republish) = self.nip34_events.get(selected_index).cloned() {
                let new_event = build_sample_event(
                    event_to_republish.kind,
                    event_to_republish.tags,
                    event_to_republish.content,
                    &self.private_key,
                )?;
                //ensure the event is pushed to all relays
                self.nip34_events.push(new_event);
            }
        }
        Ok(())
    }

    /// Publishes the selected NIP-34 event to the configured relays.
    fn publish_selected_event(&mut self) -> Result<()> {
        let selected_index = self
            .nip34_state
            .selected()
            .ok_or_else(|| Nip34Error::Publish("no event selected".into()))?;
        let event = self
            .nip34_events
            .get(selected_index)
            .ok_or_else(|| Nip34Error::Publish("selected event not found".into()))?;
        if self.relays.is_empty() {
            return Err(Nip34Error::Publish(
                "no relays configured; use --relays or config".into(),
            ));
        }

        let rt = tokio::runtime::Runtime::new().map_err(Nip34Error::Terminal)?;
        let results = rt.block_on(publish_event_to_relays(event, &self.relays));

        let ok_count = results.iter().filter(|r| r.contains("\"true\"")).count();
        if ok_count == 0 {
            self.set_error(format!("Publish failed on all {} relays", results.len()));
        } else {
            self.set_status(format!(
                "Published to {}/{} relays",
                ok_count,
                results.len()
            ));
        }
        let summary = results.join("\n");
        self.show_popup(format!(
            "Publish results ({}/{} OK):\n{}",
            ok_count,
            results.len(),
            summary
        ));
        Ok(())
    }

    /// Publishes the selected event to the configured relays plus the
    /// crawler/p2p syndication relay set.
    #[cfg(feature = "nip34-syndicate")]
    fn syndicate_selected_event(&mut self) -> Result<()> {
        let selected_index = self
            .nip34_state
            .selected()
            .ok_or_else(|| Nip34Error::Publish("no event selected".into()))?;
        let event = self
            .nip34_events
            .get(selected_index)
            .ok_or_else(|| Nip34Error::Publish("selected event not found".into()))?;

        let mut relays: std::collections::HashSet<String> = self.relays.iter().cloned().collect();
        relays.extend(collect_syndication_relays());
        let relays: Vec<String> = relays.into_iter().collect();

        if relays.is_empty() {
            return Err(Nip34Error::Publish(
                "no relays configured or discovered".into(),
            ));
        }

        let rt = tokio::runtime::Runtime::new().map_err(Nip34Error::Terminal)?;
        let results = rt.block_on(publish_event_to_relays(event, &relays));

        let ok_count = results.iter().filter(|r| r.contains("\"true\"")).count();
        if ok_count == 0 {
            self.set_error(format!(
                "Syndication failed on all {} relays",
                results.len()
            ));
        } else {
            self.set_status(format!(
                "Syndicated to {}/{} relays",
                ok_count,
                results.len()
            ));
        }
        let summary = results.join("\n");
        self.show_popup(format!(
            "Syndication results ({}/{} OK):\n{}",
            ok_count,
            results.len(),
            summary
        ));
        Ok(())
    }

    /// Loads git diff for selected commits (max 2 for range diff).
    fn load_full_commit(&mut self) -> Result<()> {
        let mut diff_content = String::new();

        if !self.selected_commits.is_empty() || self.show_full_commit {
            let mut selected_indices: Vec<_> = self.selected_commits.iter().cloned().collect();
            selected_indices.sort();

            if self.selected_commits.len() == 2 {
                // Show diff range between two commits
                if let (Some(from_index), Some(to_index)) =
                    (selected_indices.get(0), selected_indices.get(1))
                {
                    if let (Some(from_commit), Some(to_commit)) =
                        (self.commits.get(*from_index), self.commits.get(*to_index))
                    {
                        diff_content
                            .push_str("╭─────────────────────────────────────────────────╮\n");
                        diff_content
                            .push_str("│                   Diff Range                        │\n");
                        diff_content
                            .push_str("╰─────────────────────────────────────────────────╯\n\n");

                        diff_content.push_str(&format!(
                            "From: [{}] {} - {}\n",
                            from_commit.hash, from_commit.author, from_commit.summary
                        ));
                        diff_content.push_str(&format!(
                            "To:   [{}] {} - {}\n\n",
                            to_commit.hash, to_commit.author, to_commit.summary
                        ));

                        diff_content
                            .push_str("╭─────────────────────────────────────────────────╮\n");
                        diff_content
                            .push_str("│                      Git Diff                       │\n");
                        diff_content
                            .push_str("╰─────────────────────────────────────────────────╯\n\n");

                        diff_content.push_str("📝 Changes between commits:\n\n");

                        // Get diff between two commits
                        let from_oid = git2::Oid::from_str(&from_commit.full_hash)?;
                        let to_oid = git2::Oid::from_str(&to_commit.full_hash)?;

                        if let (Ok(from_commit_obj), Ok(to_commit_obj)) = (
                            self.repo.find_commit(from_oid),
                            self.repo.find_commit(to_oid),
                        ) {
                            let from_tree = from_commit_obj.tree()?;
                            let to_tree = to_commit_obj.tree()?;
                            let diff = self.repo.diff_tree_to_tree(
                                Some(&from_tree),
                                Some(&to_tree),
                                None,
                            )?;
                            self.format_diff(&diff, &mut diff_content)?;
                        }
                    }
                }
            } else {
                // Show selection summary (1 or 3+ commits selected)
                diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                diff_content.push_str(&format!(
                    "│              Selected {} Commit(s)                │\n",
                    self.selected_count()
                ));
                diff_content.push_str("╰─────────────────────────────────────────────────╯\n\n");

                for (count, &index) in selected_indices.iter().enumerate() {
                    if let Some(commit) = self.commits.get(index) {
                        diff_content.push_str(&format!(
                            "{}. [{}] {} - {}\n",
                            count + 1,
                            commit.hash,
                            commit.author,
                            commit.summary
                        ));
                    }
                }

                diff_content.push_str("\n");
                if self.selected_commits.len() == 1 {
                    diff_content.push_str("💡 Tip: Select another commit to view diff range\n");
                } else {
                    diff_content.push_str(
                        "💡 Tip: Only 2 commits allowed for diff range. Press 'c' to clear.\n",
                    );
                }
            }
        } else if let Some(selected_index) = self.commit_state.selected() {
            // No commits selected, load current focused commit
            if let Some(commit) = self.commits.get(selected_index) {
                let commit_oid = git2::Oid::from_str(&commit.full_hash)?;
                if let Ok(git_commit) = self.repo.find_commit(commit_oid) {
                    diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                    diff_content
                        .push_str("│                    Commit Details                    │\n");
                    diff_content.push_str("╰─────────────────────────────────────────────────╯\n");
                    diff_content.push_str(&format!("Commit: {}\n", git_commit.id()));
                    diff_content.push_str(&format!(
                        "Author: {} <{}>\n",
                        git_commit.author().name().unwrap_or("Unknown"),
                        git_commit.author().email().unwrap_or("unknown@example.com")
                    ));

                    // Add committer if different from author
                    if git_commit.author() != git_commit.committer() {
                        diff_content.push_str(&format!(
                            "Committer: {} <{}>\n",
                            git_commit.committer().name().unwrap_or("Unknown"),
                            git_commit
                                .committer()
                                .email()
                                .unwrap_or("unknown@example.com")
                        ));
                        diff_content.push_str(&format!(
                            "Commit Date: {}\n",
                            chrono::DateTime::from_timestamp(
                                git_commit.committer().when().seconds(),
                                0
                            )
                            .map(|dt| dt.naive_local())
                            .unwrap_or_default()
                            .format("%Y-%m-%d %H:%M:%S")
                        ));
                    } else {
                        diff_content.push_str(&format!(
                            "Date: {}\n",
                            chrono::DateTime::from_timestamp(
                                git_commit.author().when().seconds(),
                                0
                            )
                            .map(|dt| dt.naive_local())
                            .unwrap_or_default()
                            .format("%Y-%m-%d %H:%M:%S")
                        ));
                    }

                    diff_content.push_str("\n");
                    diff_content.push_str("Message:\n");
                    diff_content.push_str(&format!(
                        "    {}\n",
                        git_commit.summary().ok().flatten().unwrap_or("")
                    ));

                    // Add full commit message body if it exists
                    if let Ok(message) = git_commit.message() {
                        let lines: Vec<&str> = message.lines().collect();
                        if lines.len() > 1 {
                            for line in lines.iter().skip(1) {
                                if !line.trim().is_empty() {
                                    diff_content.push_str(&format!("    {}\n", line));
                                }
                            }
                        }
                    }

                    diff_content.push_str("\n");
                    diff_content.push_str("╭─────────────────────────────────────────────────╮\n");
                    diff_content
                        .push_str("│                      Git Diff                       │\n");
                    diff_content
                        .push_str("╰─────────────────────────────────────────────────╯\n\n");

                    // Get diff against parent(s)
                    let parent_count = git_commit.parent_count();

                    if parent_count == 0 {
                        // Initial commit - show all files
                        diff_content.push_str("🌟 Initial Commit - All files:\n\n");

                        let tree = git_commit.tree()?;
                        let diff = self.repo.diff_tree_to_tree(None, Some(&tree), None)?;
                        self.format_diff(&diff, &mut diff_content)?;
                    } else {
                        // Show diff against first parent
                        if parent_count > 1 {
                            diff_content
                                .push_str("🔀 Merge Commit - Diff against first parent:\n\n");
                        } else {
                            diff_content.push_str("📝 Changes - Diff against parent:\n\n");
                        }

                        if let Ok(parent) = git_commit.parent(0) {
                            let parent_tree = parent.tree()?;
                            let current_tree = git_commit.tree()?;
                            let diff = self.repo.diff_tree_to_tree(
                                Some(&parent_tree),
                                Some(&current_tree),
                                None,
                            )?;
                            self.format_diff(&diff, &mut diff_content)?;
                        }
                    }
                }
            }
        }

        self.full_commit_details = Some(diff_content);
        self.show_full_commit = true;
        Ok(())
    }

    /// Formats a git diff object into a readable string.
    fn format_diff(&self, diff: &git2::Diff, output: &mut String) -> Result<()> {
        let mut patch = String::new();
        diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
            let origin_char = line.origin();
            let content_str = std::str::from_utf8(line.content()).unwrap_or("");

            match origin_char {
                '+' => {
                    patch.push('+');
                    patch.push_str(content_str);
                }
                '-' => {
                    patch.push('-');
                    patch.push_str(content_str);
                }
                ' ' => {
                    patch.push(' ');
                    patch.push_str(content_str);
                }
                'F' => {
                    // File header
                    if let Some(new_path) = delta.new_file().path() {
                        if let Some(old_path) = delta.old_file().path() {
                            patch.push_str(&format!(
                                "diff --git a/{} b/{}\n",
                                old_path.display(),
                                new_path.display()
                            ));
                        }
                    }
                }
                '>' => {
                    // Add/delete/rename operations
                    patch.push_str(content_str);
                }
                '<' => {
                    // Add/delete/rename operations
                    patch.push_str(content_str);
                }
                _ => {
                    // Other line types including file headers
                    patch.push(origin_char);
                    patch.push_str(content_str);
                }
            }
            true
        })?;

        output.push_str(&patch);
        Ok(())
    }
}

/// RAII terminal guard that restores the screen on drop.
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn new(terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Self {
        Self { terminal }
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

/// Helper to render a centered rectangle for popups/overlays.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Runs the TUI application loop.
fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<()> {
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Global popup / help handling first.
                    if app.show_help {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc | KeyCode::Char('?') | KeyCode::F(1) => {
                                app.show_help = false;
                            }
                            _ => {}
                        }
                        continue;
                    }
                    if app.popup_message.is_some() {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Esc | KeyCode::Enter => app.dismiss_popup(),
                            _ => {}
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('?') | KeyCode::F(1) => app.show_help = true,
                        KeyCode::Tab => app.switch_mode(),
                        KeyCode::Char('1') => app.set_mode(NavigatorMode::Commits),
                        KeyCode::Char('2') => app.set_mode(NavigatorMode::Branches),
                        KeyCode::Char('3') => app.set_mode(NavigatorMode::Nip34Events),
                        KeyCode::Down | KeyCode::Char('j') => {

                            app.next();
                            if app.current_mode == NavigatorMode::Commits {
                                app.show_full_commit = false; //always show_full_commit

                                if let Err(e) = app.load_full_commit() {
                                    app.set_error(format!("{e}"));
                                }

                            }

                        },
                        KeyCode::Up | KeyCode::Char('k') => {

                           app.previous();
                            if app.current_mode == NavigatorMode::Commits {
                                app.show_full_commit = false; //always show_full_commit

                                if let Err(e) = app.load_full_commit() {
                                    app.set_error(format!("{e}"));
                                }

                            }

                        },
                        /*KeyCode::Enter | */
                        KeyCode::Char(' ') => match app.current_mode {

                            NavigatorMode::Commits => {

                               app.toggle_commit_selection()

                            },
                            NavigatorMode::Branches => {

                                // Could add branch checkout here

                            }
                            NavigatorMode::Nip34Events => app.toggle_nip34_selection(),
                        },
                        KeyCode::Right => {
                            if app.current_mode == NavigatorMode::Commits {
                                app.show_full_commit = false; //always show_full_commit

                                if let Err(e) = app.load_full_commit() {
                                    app.set_error(format!("{e}"));
                                }

                            }
                        }
                        KeyCode::Left => {
                            if app.current_mode == NavigatorMode::Commits && app.show_full_commit {
                                  app.show_full_commit = false;
                                  if let Err(e) = app.load_full_commit() {
                                      app.set_error(format!("{e}"));
                                  }
                            }
                        }
                        KeyCode::Char('c') => {

                            app.clear_selection();
                            app.show_full_commit = false;
                            app.clear_messages();

                        }
                        KeyCode::Char('n') => {
                            // Create NIP-34 patch from selected commits
                            if app.current_mode == NavigatorMode::Commits {

                                match app.create_nip34_patch_event() {
                                    Ok(()) => app.set_status("NIP-34 patch event created"),
                                    Err(e) => app.set_error(format!("{e}")),
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            if app.current_mode == NavigatorMode::Nip34Events {

                                match app.republish_nip34_event() {
                                    Ok(()) => app.set_status("NIP-34 event republished"),
                                    Err(e) => app.set_error(format!("{e}")),
                                }
                            }
                        }
                        KeyCode::Char('p') => {
                            if app.current_mode == NavigatorMode::Nip34Events {
                                if let Err(e) = app.publish_selected_event() {
                                    app.set_error(format!("{e}"));
                                }
                            }
                        }
                        KeyCode::Char('s') => {
                            if app.current_mode == NavigatorMode::Nip34Events {
                                #[cfg(feature = "nip34-syndicate")]
                                {
                                    if let Err(e) = app.syndicate_selected_event() {
                                        app.set_error(format!("{e}"));
                                    }
                                }
                                #[cfg(not(feature = "nip34-syndicate"))]
                                {
                                    app.set_error(
                                        "Syndication requires the nip34-syndicate feature"
                                            .to_string(),
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
}

/// Draws the UI to the terminal frame.
fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();

    // Top navigation tabs
    let titles = vec!["Commits", "Branches", "NIP-34 Events"];
    let selected_index = match app.current_mode {
        NavigatorMode::Commits => 0,
        NavigatorMode::Branches => 1,
        NavigatorMode::Nip34Events => 2,
    };

    let event_pow: u32 = app
        .nip34_events
        .iter()
        .map(|e| u32::from(get_leading_zero_bits(&e.id.0)))
        .sum();
    let total_apow = app.pow_summary.total_pow + event_pow;

    let title = Line::from(vec![
        Span::styled(
            "NIP-34 Gnostr Navigator",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(
            format!("aPoW: {} bits", total_apow),
            Style::default().fg(Color::Yellow),
        ),
        Span::raw(" "),
        Span::styled(
            format!(
                "(commits: {}, notes: {}, events: {})",
                app.pow_summary.commit_pow, app.pow_summary.note_pow, event_pow
            ),
            Style::default().fg(Color::Gray),
        ),
    ]);

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title),
        )
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(Style::default().fg(Color::White).bg(Color::Blue))
        .divider(" | ")
        .select(selected_index);

    f.render_widget(tabs, Rect::new(0, 0, size.width, 3));

    // Main content area below tabs
    let content_area = Rect::new(0, 3, size.width, size.height - 4);

    match app.current_mode {
        NavigatorMode::Commits => render_commits_view(f, app, content_area),
        NavigatorMode::Branches => render_branches_view(f, app, content_area),
        NavigatorMode::Nip34Events => render_nip34_view(f, app, content_area),
    }

    // Message area above help text
    let message_area = Rect::new(0, size.height.saturating_sub(2), size.width, 1);
    render_message(f, app, message_area);

    // Help text at bottom
    let help_spans = get_help_spans(app);
    let help_widget = Paragraph::new(Line::from(help_spans))
        .style(Style::default().bg(Color::Black).fg(Color::White))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default());

    let help_area = Rect::new(0, size.height.saturating_sub(1), size.width, 1);
    f.render_widget(help_widget, help_area);

    // Overlays
    if app.show_help {
        render_help_overlay(f, size);
    }
    if let Some(ref msg) = app.popup_message {
        render_popup(f, "Notice", msg, Color::Blue);
    }
}

fn render_message(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref msg) = app.error_message {
        let line = Line::from(vec![
            Span::styled("ERROR ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(msg.clone(), Style::default().fg(Color::Red)),
        ]);
        let widget = Paragraph::new(line)
            .style(Style::default().bg(Color::Black))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default());
        f.render_widget(widget, area);
    } else if let Some(ref msg) = app.status_message {
        let line = Line::from(vec![
            Span::styled("OK ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(msg.clone(), Style::default().fg(Color::Green)),
        ]);
        let widget = Paragraph::new(line)
            .style(Style::default().bg(Color::Black))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default());
        f.render_widget(widget, area);
    }
}

/// Renders the commits view.
fn render_commits_view(f: &mut Frame, app: &mut App, area: Rect) {
    // Layout for list (left) and details (right)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- Commit List ---
    let items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(index, c)| {
            let selected_indicator = if app.selected_commits.contains(&index) {
                "✓ "
            } else {
                "  "
            };
            let content = format!(
                "{}[{}] {} - {} ({})\n", // Added committer_date here
                selected_indicator,
                c.hash,
                c.author,
                c.summary,
                c.committer_date // Added committer_date
            );
            let style = if app.selected_commits.contains(&index) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let title = if app.selected_commits.len() > 0 {
        format!("Commit History ({} selected)", app.selected_commits.len())
    } else {
        "Commit History".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if app.selected_commits.len() > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.commit_state);

    // --- Details Panel ---
    let details_block = Block::default()
        .title("Gnostr NIP-34 Operations")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(details_block, chunks[1]);

    if app.show_full_commit {
        if let Some(ref details) = app.full_commit_details {
            let details_chunk = chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });
            f.render_widget(
                Paragraph::new(details.clone()).style(Style::default().fg(Color::White)),
                details_chunk,
            );
        }
    } else {
        let details_chunk = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(8), // PoW summary
                Constraint::Length(1), // spacer
                Constraint::Length(3), // commit share gauge
                Constraint::Length(3), // note share gauge
                Constraint::Length(3), // event share gauge
                Constraint::Length(1), // spacer
                Constraint::Min(0),    // Tips
            ])
            .split(chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            }));

        let event_pow: u32 = app
            .nip34_events
            .iter()
            .map(|e| u32::from(get_leading_zero_bits(&e.id.0)))
            .sum();
        let total = app.pow_summary.total_pow + event_pow;
        let max_scale = (total.max(64) as f64).max(1.0);

        let commit_ratio = app.pow_summary.commit_pow as f64 / max_scale;
        let note_ratio = app.pow_summary.note_pow as f64 / max_scale;
        let event_ratio = event_pow as f64 / max_scale;

        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("aPoW", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::raw(" = accumulated proof-of-work"),
                ]),
                Line::from(vec![
                    Span::raw("commits "),
                    Span::styled(format!("{} bits", app.pow_summary.commit_pow), Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::raw("notes   "),
                    Span::styled(format!("{} bits", app.pow_summary.note_pow), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(vec![
                    Span::raw("events  "),
                    Span::styled(format!("{} bits", event_pow), Style::default().fg(Color::Magenta)),
                ]),
                Line::from(vec![
                    Span::raw("total   "),
                    Span::styled(format!("{} bits", total), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]),
            ])
            .block(Block::default().title("PoW").borders(Borders::ALL)),
            details_chunk[0],
        );

        f.render_widget(
            Gauge::default()
                .block(Block::default().title("commit share").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
                .ratio(commit_ratio.min(1.0))
                .label(format!("{:.0}%", commit_ratio * 100.0)),
            details_chunk[2],
        );

        f.render_widget(
            Gauge::default()
                .block(Block::default().title("note share").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Yellow).bg(Color::Black))
                .ratio(note_ratio.min(1.0))
                .label(format!("{:.0}%", note_ratio * 100.0)),
            details_chunk[3],
        );

        f.render_widget(
            Gauge::default()
                .block(Block::default().title("event share").borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Magenta).bg(Color::Black))
                .ratio(event_ratio.min(1.0))
                .label(format!("{:.0}%", event_ratio * 100.0)),
            details_chunk[4],
        );

        f.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::styled("Selected: ", Style::default().fg(Color::Yellow)),
                    Span::raw(format!("{} commits", app.selected_commits.len())),
                ]),
                Line::from(vec![
                    Span::styled("Tip: ", Style::default().fg(Color::Green)),
                    Span::raw("select 2 commits, then press 'n' to create a NIP-34 patch"),
                ]),
            ])
            .wrap(Wrap { trim: true }),
            details_chunk[6],
        );
    }
}

/// Renders the branches view.
fn render_branches_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- Branch List ---
    let items: Vec<ListItem> = app
        .branches
        .iter()
        .map(|b| {
            let prefix = if b.is_current {
                "* "
            } else if b.is_remote {
                "R "
            } else {
                "  "
            };
            let content = format!(
                "{}{} - {} ({}) - {}\n",
                prefix, b.name, b.commit_message, b.author, b.commit_hash
            ); // Added commit_hash and author
            let style = if b.is_current {
                Style::default().fg(Color::Green)
            } else if b.is_remote {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title("Git Branches")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.branch_state);

    // --- Details Panel ---
    let details_block = Block::default()
        .title("Branch Operations")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(details_block, chunks[1]);
}

/// Renders the NIP-34 events view.
fn render_nip34_view(f: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area);

    // --- NIP-34 Events List ---
    let items: Vec<ListItem> = app
        .nip34_events
        .iter()
        .enumerate()
        .map(|(index, event)| {
            let selected_indicator = if app.selected_nip34_events.contains(&index) {
                "✓ "
            } else {
                "  "
            };
            let kind_name = kind_label(event.kind);
            let id_hex = event.id.as_hex_string();

            let content_preview = if event.content.len() > 50 {
                format!(
                    "{}\
...",
                    &event.content[..47]
                )
            } else {
                event.content.clone()
            };

            let content = format!(
                "{}[{}] {} - {}\n",
                selected_indicator,
                &id_hex[..8],
                kind_name,
                content_preview
            );
            let style = if app.selected_nip34_events.contains(&index) {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let title = if app.selected_nip34_events.len() > 0 {
        format!(
            "NIP-34 Events ({} selected)",
            app.selected_nip34_events.len()
        )
    } else {
        "NIP-34 Events".to_string()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(if app.selected_nip34_events.len() > 0 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                }),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(ratatui::style::Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut app.nip34_state);

    // --- Event Details Panel ---
    let details_block = Block::default()
        .title("NIP-34 Event Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));
    f.render_widget(details_block, chunks[1]);

    if let Some(selected_index) = app.nip34_state.selected() {
        if let Some(event) = app.nip34_events.get(selected_index) {
            let details_chunk = chunks[1].inner(ratatui::layout::Margin {
                horizontal: 1,
                vertical: 1,
            });
            let id_hex = event.id.as_hex_string();
            let sig_hex = event.sig.as_hex_string();

            let event_pow = get_leading_zero_bits(&event.id.0);
            let event_details = format!(
                "Event ID: {}\n\\
                Public Key: {}\n\\
                Kind: {} ({})\n\\
                Created: {}\n\\
                PoW: {} bits\n\\
                Signature: {}\n\\
                Content: {}\n\n\\
                Tags:\n{}",
                id_hex,
                event.pubkey,
                u32::from(event.kind),
                kind_label(event.kind),
                chrono::DateTime::from_timestamp(event.created_at.0, 0)
                    .map(|dt| dt.naive_local())
                    .unwrap_or_default()
                    .format("%Y-%m-%d %H:%M:%S"),
                event_pow,
                &sig_hex[..16],
                event.content,
                event
                    .tags
                    .iter()
                    .map(|tag| format!("  {}: {}\n", tag.tagname(), tag.value()))
                    .collect::<Vec<_>>()
                    .join("")
            );

            f.render_widget(
                Paragraph::new(event_details).style(Style::default().fg(Color::White)),
                details_chunk,
            );
        }
    }
}

/// Returns styled help spans for the bottom bar.
fn get_help_spans(app: &App) -> Vec<Span<'static>> {
    let mut spans = vec![
        Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" help "),
        Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" quit "),
        Span::styled("1/2/3", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" tabs "),
        Span::styled("tab", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" switch "),
        Span::styled("j/k", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" move "),
    ];

    match app.current_mode {
        NavigatorMode::Commits => {
            spans.extend([
                Span::styled("space", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" select "),
                Span::styled("n", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" patch "),
                Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" clear "),
                Span::styled("right", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" diff "),
            ]);
        }
        NavigatorMode::Branches => {
            spans.extend([
                Span::styled("enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" focus "),
            ]);
        }
        NavigatorMode::Nip34Events => {
            spans.extend([
                Span::styled("space", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" select "),
                Span::styled("p", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" publish "),
                Span::styled("s", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" syndicate "),
                Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" republish "),
                Span::styled("c", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw(" clear "),
            ]);
        }
    }

    spans
}

/// Renders a centered popup with the given title and colored border.
fn render_popup(f: &mut Frame, title: &str, body: &str, color: Color) {
    let area = centered_rect(64, 40, f.area());
    f.render_widget(Clear, area);

    let text = Paragraph::new(body.to_string())
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(title.to_string())
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        );
    f.render_widget(text, area);
}

/// Renders the full help overlay.
fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup = centered_rect(72, 70, area);
    f.render_widget(Clear, popup);

    let lines = vec![
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("1/2/3  jump to Commits / Branches / NIP-34 Events"),
        Line::from("tab    cycle through tabs"),
        Line::from("j/k    move selection up/down (also arrow keys)"),
        Line::from("q/esc  quit the application"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Commits", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("space  select/deselect a commit (max 2 for patch)"),
        Line::from("right  show diff / details for selected commit"),
        Line::from("left   return to commit list summary"),
        Line::from("n      create a NIP-34 patch event from the selected range"),
        Line::from("c      clear selection and messages"),
        Line::from(""),
        Line::from(vec![
            Span::styled("NIP-34 Events", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("space  select/deselect an event"),
        Line::from("p      publish the selected event to configured relays"),
        Line::from("s      syndicate the selected event via crawler/p2p relays (if enabled)"),
        Line::from("r      republish the selected event with a fresh timestamp"),
        Line::from("c      clear selection and messages"),
        Line::from(""),
        Line::from(vec![
            Span::styled("PoW / aPoW", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from("The header shows accumulated proof-of-work across git commits and NIP-34 events."),
        Line::from("Event PoW is the number of leading zero bits in the event id."),
    ];

    let help = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title("Help (press esc, ?, or F1 to close)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(help, popup);
}

/// Initializes the terminal and runs the application.
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_env("GNOSTR_NIP34_LOG"))
        .init();

    let cli = Cli::parse();
    let config_path = cli.config.clone().or_else(default_config_path);
    let config = load_config(config_path.as_ref())?;
    let private_key = resolve_private_key(&cli, &config)?;

    // Setup terminal with RAII cleanup on panic or normal exit.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let mut guard = TerminalGuard::new(terminal);

    // Create app and run it
    let app = App::new(&cli, &config, private_key)?;
    let tick_rate = Duration::from_millis(250);
    let res = run_app(&mut guard.terminal, app, tick_rate);

    if let Err(e) = res {
        println!("{e}");
    }

    Ok(())
}
