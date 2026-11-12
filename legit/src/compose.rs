use std::{collections::HashMap, fs, io::Read, path::PathBuf, time::Duration};

use anyhow::{ensure, Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use gnostr_asyncgit::git2::{Oid, Reference, Repository};
use gnostr_types::nostr::{
    client::Options as ClientOptions,
    nip34::{
        RepoRef, RepoState, GIT_REPLY_KIND, PULL_REQUEST_KIND, PULL_REQUEST_UPDATE_KIND,
        REPO_ANNOUNCEMENT_KIND, USER_GRASP_LIST_KIND,
    },
    patch::{
        build_patch_event, commit_nonce_tag, install_rustls_crypto_provider, ParentCommitLink,
    },
    Client, Event, EventBuilder, EventKind, Id, Keys, NAddr, PublicKey, RelayUrl, Tag,
    UncheckedUrl,
};

/// Compose a signed NIP-34 event without mining or modifying Git.
#[derive(Args)]
pub struct Compose {
    /// Send the composed event (requires explicit --relay destinations)
    #[arg(long, global = true, requires = "relays")]
    publish: bool,
    /// Publication destination; repeat for additional relays
    #[arg(long = "relay", global = true, value_parser = parse_relay)]
    relays: Vec<String>,
    /// Additional hashtag; all events already include #gnostr
    #[arg(long = "hashtag", global = true)]
    hashtags: Vec<String>,
    /// Git mining worker/counter nonce, for example 01-00000122 (not NIP-13 PoW)
    #[arg(long, global = true, value_parser = parse_commit_nonce)]
    commit_nonce: Option<String>,
    #[command(subcommand)]
    event: EventCommand,
}

#[derive(Subcommand)]
enum EventCommand {
    /// Repository announcement (30617)
    RepoAnnouncement(RepositoryAnnouncement),
    /// Repository branch/tag state (30618)
    RepoState(RepositoryState),
    /// Git format-patch content (1617)
    Patch(Patch),
    /// Pull request (1618)
    PullRequest(PullRequest),
    /// Update an existing pull request (1619)
    PullRequestUpdate(PullRequestUpdate),
    /// Repository issue (1621)
    Issue(Issue),
    /// NIP-22 comment (1111), or deprecated Git reply with --legacy (1622)
    Reply(Reply),
    /// Open, applied, closed, or draft status (1630-1633)
    Status(Status),
    /// Preferred grasp servers (10317)
    GraspList(GraspList),
}

#[derive(Args)]
struct RepositoryAnnouncement {
    #[arg(long, value_parser = clap::builder::NonEmptyStringValueParser::new())]
    identifier: String,
    #[arg(long, default_value = "")]
    name: String,
    #[arg(long, default_value = "")]
    description: String,
    #[arg(long, value_parser = parse_commit)]
    root_commit: String,
    #[arg(long = "clone-url")]
    clone_urls: Vec<String>,
    #[arg(long = "web-url")]
    web_urls: Vec<String>,
    /// Relays monitored by this repository, not publication destinations
    #[arg(long = "monitor-relay", value_parser = parse_relay)]
    monitor_relays: Vec<String>,
    #[arg(long = "maintainer", value_parser = parse_pubkey)]
    maintainers: Vec<PublicKey>,
}

#[derive(Args)]
struct RepositoryState {
    #[arg(long, value_parser = clap::builder::NonEmptyStringValueParser::new())]
    identifier: String,
    /// refs/heads/main=<commit>, refs/tags/v1=<commit>, or HEAD=ref: refs/heads/main
    #[arg(long = "ref", value_parser = parse_ref)]
    refs: Vec<(String, String)>,
}

#[derive(Args)]
struct Target {
    /// Repository address: 30617:<owner pubkey>:<identifier>, or naddr1...
    #[arg(long, value_parser = parse_repository)]
    repository: NAddr,
    #[arg(long, value_parser = parse_commit)]
    root_commit: Option<String>,
    /// Additional public key to notify (hex or npub1...)
    #[arg(long, value_parser = parse_pubkey)]
    notify: Vec<PublicKey>,
}

impl Target {
    fn tags(&self) -> Vec<Tag> {
        let mut tags = vec![
            Tag::new_tag(
                "a",
                &format!("30617:{}:{}", self.repository.author, self.repository.d),
            ),
            Tag::new_pubkey(self.repository.author, None, None),
        ];
        if let Some(commit) = &self.root_commit {
            tags.push(Tag::new_tag("r", commit));
        }
        for pubkey in &self.notify {
            let tag = Tag::new_pubkey(*pubkey, None, None);
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
        tags
    }
}

#[derive(Args)]
struct Content {
    /// Literal event content
    #[arg(long, default_value = "", conflicts_with = "content_file")]
    content: String,
    /// UTF-8 content file, or - to read standard input
    #[arg(long, value_name = "PATH")]
    content_file: Option<PathBuf>,
}

impl Content {
    fn read(&self) -> Result<String> {
        match &self.content_file {
            Some(path) if path.as_os_str() == "-" => {
                let mut content = String::new();
                std::io::stdin()
                    .read_to_string(&mut content)
                    .context("Failed to read event content from stdin")?;
                Ok(content)
            }
            Some(path) => fs::read_to_string(path)
                .with_context(|| format!("Failed to read event content from {}", path.display())),
            None => Ok(self.content.clone()),
        }
    }
}

#[derive(Args)]
struct Patch {
    #[command(flatten)]
    target: Target,
    #[command(flatten)]
    body: Content,
    #[arg(long, value_parser = parse_commit)]
    commit: String,
    #[arg(long, value_parser = parse_commit)]
    parent_commit: Option<String>,
    /// Previous patch or original patch revision event (hex or note1...)
    #[arg(long, value_parser = parse_id)]
    previous_patch: Option<Id>,
}

#[derive(Args)]
struct PullRequest {
    #[command(flatten)]
    target: Target,
    #[command(flatten)]
    body: Content,
    #[arg(long, value_parser = clap::builder::NonEmptyStringValueParser::new())]
    subject: String,
    #[arg(long, value_parser = parse_commit)]
    commit: String,
    #[arg(long = "clone-url", required = true, value_parser = clap::builder::NonEmptyStringValueParser::new())]
    clone_urls: Vec<String>,
    #[arg(long)]
    branch_name: Option<String>,
    #[arg(long, value_parser = parse_commit)]
    merge_base: Option<String>,
    #[arg(long, value_parser = parse_id)]
    original_patch: Option<Id>,
}

#[derive(Args)]
struct PullRequestUpdate {
    #[command(flatten)]
    target: Target,
    #[arg(long, value_parser = parse_id)]
    event: Id,
    #[arg(long, value_parser = parse_pubkey)]
    author: PublicKey,
    #[arg(long, value_parser = parse_commit)]
    commit: String,
    #[arg(long = "clone-url", required = true, value_parser = clap::builder::NonEmptyStringValueParser::new())]
    clone_urls: Vec<String>,
    #[arg(long, value_parser = parse_commit)]
    merge_base: Option<String>,
}

#[derive(Args)]
struct Issue {
    #[command(flatten)]
    target: Target,
    #[command(flatten)]
    body: Content,
    #[arg(long)]
    subject: Option<String>,
}

#[derive(Args)]
struct Reply {
    #[command(flatten)]
    target: Target,
    #[command(flatten)]
    body: Content,
    #[arg(long, value_parser = parse_id)]
    event: Id,
    #[arg(long, value_parser = parse_pubkey)]
    author: PublicKey,
    /// Kind of the root event: patch (1617), pull request (1618), or issue (1621)
    #[arg(long, value_parser = clap::value_parser!(u32))]
    root_kind: u32,
    #[arg(long, value_parser = parse_id, requires_all = ["reply_author", "reply_kind"])]
    reply_event: Option<Id>,
    #[arg(long, value_parser = parse_pubkey, requires_all = ["reply_event", "reply_kind"])]
    reply_author: Option<PublicKey>,
    #[arg(long, requires_all = ["reply_event", "reply_author"])]
    reply_kind: Option<u32>,
    /// Use deprecated NIP-34 kind 1622 rather than a NIP-22 comment
    #[arg(long)]
    legacy: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum StatusKind {
    Open,
    Applied,
    Closed,
    Draft,
}

#[derive(Args)]
struct Status {
    #[arg(value_enum)]
    status: StatusKind,
    #[command(flatten)]
    target: Target,
    #[command(flatten)]
    body: Content,
    #[arg(long, value_parser = parse_id)]
    event: Id,
    #[arg(long, value_parser = parse_pubkey)]
    author: PublicKey,
    #[arg(long, value_parser = parse_id)]
    revision: Option<Id>,
    #[arg(long, value_parser = parse_commit)]
    merge_commit: Option<String>,
    #[arg(long = "applied-commit", value_parser = parse_commit)]
    applied_commits: Vec<String>,
    #[arg(long = "applied-patch", value_parser = parse_id)]
    applied_patches: Vec<Id>,
}

#[derive(Args)]
struct GraspList {
    #[arg(long = "server", value_parser = parse_relay)]
    servers: Vec<String>,
}

impl Compose {
    /// Build and sign one event. This performs no networking or Git mutation.
    pub fn build(&self, keys: &Keys) -> Result<Event> {
        self.build_with_pow(keys, 0)
    }

    /// Build and sign one event with an independent NIP-13 bit target.
    pub fn build_with_pow(&self, keys: &Keys, difficulty: u8) -> Result<Event> {
        self.build_with_mining(
            keys,
            &gnostr_types::nostr::EventMining::Difficulty(difficulty),
        )
    }

    /// Build with an independent bit or hexadecimal-prefix target.
    pub fn build_with_mining(
        &self,
        keys: &Keys,
        target: &gnostr_types::nostr::EventMining,
    ) -> Result<Event> {
        let secret = keys.secret_key()?;
        let (kind, content, mut tags) = match &self.event {
            EventCommand::RepoAnnouncement(args) => {
                let repo = RepoRef {
                    name: args.name.clone(),
                    description: args.description.clone(),
                    identifier: args.identifier.clone(),
                    root_commit: args.root_commit.clone(),
                    git_server: args.clone_urls.clone(),
                    web: args.web_urls.clone(),
                    relays: args
                        .monitor_relays
                        .iter()
                        .map(|relay| UncheckedUrl::from_str(relay))
                        .collect(),
                    hashtags: vec![],
                    maintainers: if args.maintainers.is_empty() {
                        vec![keys.public_key()]
                    } else {
                        args.maintainers.clone()
                    },
                    trusted_maintainer: keys.public_key(),
                    events: HashMap::new(),
                };
                let event = repo.to_event(&secret)?;
                (event.kind, String::new(), event.tags)
            }
            EventCommand::RepoState(args) => {
                let mut state = HashMap::new();
                for (name, value) in &args.refs {
                    ensure!(
                        state.insert(name.clone(), value.clone()).is_none(),
                        "Duplicate Git reference: {}",
                        name
                    );
                }
                let event = RepoState::build(args.identifier.clone(), state, &secret)?.event;
                (event.kind, event.content, event.tags)
            }
            EventCommand::Patch(args) => {
                let content = args.body.read()?;
                ensure!(
                    content.starts_with("From ")
                        && content
                            .lines()
                            .next()
                            .and_then(|line| line.split_whitespace().nth(1))
                            == Some(args.commit.as_str()),
                    "Patch content must be git format-patch output for --commit"
                );
                let parent = args
                    .parent_commit
                    .as_ref()
                    .map(|commit| {
                        Keys::parse(format!("{:0>64}", commit))
                            .map(|keys| ParentCommitLink {
                                pubkey: keys.public_key(),
                                event_id: args.previous_patch,
                            })
                            .context("Failed to derive parent commit public key")
                    })
                    .transpose()?;
                let event = build_patch_event(
                    keys,
                    &args.target.repository.d,
                    &args.commit,
                    &content,
                    None,
                    parent.as_ref(),
                )?;
                let mut tags = event.tags;
                tags.extend(args.target.tags());
                tags.push(Tag::new_tag("r", &args.commit));
                if let Some(commit) = &args.parent_commit {
                    tags.push(Tag::new_tag("parent-commit", commit));
                }
                if let Some(event) = args.previous_patch {
                    if parent.is_none() {
                        tags.push(Tag::new_event(event, None, Some("reply".to_string())));
                    }
                } else {
                    tags.push(Tag::new_tag("t", "root"));
                }
                (event.kind, content, tags)
            }
            EventCommand::PullRequest(args) => {
                let mut tags = args.target.tags();
                tags.push(Tag::new_tag("subject", &args.subject));
                tags.push(Tag::new_tag("c", &args.commit));
                tags.push(multi_tag("clone", &args.clone_urls));
                if let Some(branch) = &args.branch_name {
                    tags.push(Tag::new_tag("branch-name", branch));
                }
                if let Some(commit) = &args.merge_base {
                    tags.push(Tag::new_tag("merge-base", commit));
                }
                if let Some(event) = args.original_patch {
                    tags.push(Tag::new_event(event, None, Some("root".to_string())));
                }
                (EventKind::from(PULL_REQUEST_KIND), args.body.read()?, tags)
            }
            EventCommand::PullRequestUpdate(args) => {
                let mut tags = args.target.tags();
                tags.extend([
                    Tag::new_tag("E", &args.event.as_hex_string()),
                    Tag::new_tag("P", &args.author.as_hex_string()),
                    Tag::new_tag("c", &args.commit),
                    multi_tag("clone", &args.clone_urls),
                ]);
                if let Some(commit) = &args.merge_base {
                    tags.push(Tag::new_tag("merge-base", commit));
                }
                (
                    EventKind::from(PULL_REQUEST_UPDATE_KIND),
                    String::new(),
                    tags,
                )
            }
            EventCommand::Issue(args) => {
                let content = args.body.read()?;
                ensure!(
                    !content.is_empty(),
                    "Issue requires --content or --content-file"
                );
                let mut tags = args.target.tags();
                if let Some(subject) = &args.subject {
                    tags.push(Tag::new_tag("subject", subject));
                }
                (EventKind::GitIssue, content, tags)
            }
            EventCommand::Reply(args) => {
                ensure!(
                    [1617, PULL_REQUEST_KIND, 1621].contains(&args.root_kind),
                    "--root-kind must be 1617, 1618, or 1621"
                );
                let content = args.body.read()?;
                ensure!(
                    !content.is_empty(),
                    "Reply requires --content or --content-file"
                );
                let mut tags = args.target.tags();
                let parent_event = args.reply_event.unwrap_or(args.event);
                let parent_author = args.reply_author.unwrap_or(args.author);
                let parent_kind = args.reply_kind.unwrap_or(args.root_kind);
                ensure!(
                    [1111, 1617, PULL_REQUEST_KIND, 1621, GIT_REPLY_KIND].contains(&parent_kind),
                    "--reply-kind must be 1111, 1617, 1618, 1621, or 1622"
                );
                if args.legacy {
                    tags.push(Tag::new_event(args.event, None, Some("root".to_string())));
                    if args.reply_event.is_some() {
                        tags.push(Tag::new_event(
                            parent_event,
                            None,
                            Some("reply".to_string()),
                        ));
                    }
                } else {
                    tags.extend([
                        Tag::new_tag("E", &args.event.as_hex_string()),
                        Tag::new_tag("K", &args.root_kind.to_string()),
                        Tag::new_tag("P", &args.author.as_hex_string()),
                        Tag::new_tag("e", &parent_event.as_hex_string()),
                        Tag::new_tag("k", &parent_kind.to_string()),
                    ]);
                }
                for author in [args.author, parent_author] {
                    let tag = Tag::new_pubkey(author, None, None);
                    if !tags.contains(&tag) {
                        tags.push(tag);
                    }
                }
                (
                    EventKind::from(if args.legacy { GIT_REPLY_KIND } else { 1111 }),
                    content,
                    tags,
                )
            }
            EventCommand::Status(args) => {
                ensure!(
                    args.status == StatusKind::Applied
                        || (args.merge_commit.is_none()
                            && args.applied_commits.is_empty()
                            && args.applied_patches.is_empty()),
                    "--merge-commit, --applied-commit, and --applied-patch require status applied"
                );
                let kind = match args.status {
                    StatusKind::Open => EventKind::GitStatusOpen,
                    StatusKind::Applied => EventKind::GitStatusApplied,
                    StatusKind::Closed => EventKind::GitStatusClosed,
                    StatusKind::Draft => EventKind::GitStatusDraft,
                };
                let mut tags = args.target.tags();
                tags.push(Tag::new_event(args.event, None, Some("root".to_string())));
                let author = Tag::new_pubkey(args.author, None, None);
                if !tags.contains(&author) {
                    tags.push(author);
                }
                if let Some(event) = args.revision {
                    tags.push(Tag::new_event(event, None, Some("reply".to_string())));
                }
                if let Some(commit) = &args.merge_commit {
                    tags.push(Tag::new_tag("merge-commit", commit));
                    tags.push(Tag::new_tag("r", commit));
                }
                if !args.applied_commits.is_empty() {
                    tags.push(multi_tag("applied-as-commits", &args.applied_commits));
                    tags.extend(
                        args.applied_commits
                            .iter()
                            .map(|commit| Tag::new_tag("r", commit)),
                    );
                }
                tags.extend(
                    args.applied_patches
                        .iter()
                        .map(|event| Tag::new_tag("q", &event.as_hex_string())),
                );
                (kind, args.body.read()?, tags)
            }
            EventCommand::GraspList(args) => (
                EventKind::from(USER_GRASP_LIST_KIND),
                String::new(),
                args.servers
                    .iter()
                    .map(|server| Tag::new_tag("g", server))
                    .collect(),
            ),
        };
        for hashtag in std::iter::once("gnostr").chain(self.hashtags.iter().map(String::as_str)) {
            ensure!(
                !hashtag.trim_start_matches('#').is_empty(),
                "Hashtags must contain more than # characters"
            );
            let tag = Tag::new_tag("t", hashtag.trim_start_matches('#'));
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }
        if let Some(nonce) = &self.commit_nonce {
            tags.push(commit_nonce_tag(nonce)?);
        }
        EventBuilder::new(kind, content, tags).to_mined_event(&secret, target)
    }

    /// Print signed JSON; contact only explicitly selected relays with --publish.
    pub fn run(&self, signing_keys: Option<Keys>, repo: &str) -> Result<()> {
        self.run_with_pow(signing_keys, repo, 0)
    }

    /// Compose an optionally mined event and publish only when requested.
    pub fn run_with_pow(
        &self,
        signing_keys: Option<Keys>,
        repo: &str,
        difficulty: u8,
    ) -> Result<()> {
        self.run_with_mining(
            signing_keys,
            repo,
            &gnostr_types::nostr::EventMining::Difficulty(difficulty),
        )
    }

    /// Compose with a bit or prefix target, retaining explicit publication opt-in.
    pub fn run_with_mining(
        &self,
        signing_keys: Option<Keys>,
        repo: &str,
        target: &gnostr_types::nostr::EventMining,
    ) -> Result<()> {
        let keys = match signing_keys {
            Some(keys) => keys,
            None => {
                let repo = Repository::open(repo)
                    .context("Without --nsec, composition requires an existing Git HEAD")?;
                let head = repo.head()?.peel_to_commit()?.id().to_string();
                Keys::parse(format!("{:0>64}", head))
                    .context("Failed to derive signing keys from HEAD")?
            }
        };
        let event = self.build_with_mining(&keys, target)?;
        info!(
            "Composed NIP-34 event: id={}, kind={}, pubkey={}",
            event.id,
            u32::from(event.kind),
            event.pubkey
        );
        let json = serde_json::to_string(&event)?;
        if self.publish {
            ensure!(
                !self.relays.is_empty(),
                "--publish requires at least one --relay"
            );
            install_rustls_crypto_provider();
            tokio::runtime::Runtime::new()?.block_on(async {
                let mut client = Client::new(
                    &keys,
                    ClientOptions::new().send_timeout(Some(Duration::from_secs(5))),
                );
                client.add_relays(self.relays.clone()).await?;
                client.connect().await;
                let id = client.send_event(event).await?;
                info!("Sent composed NIP-34 event {}", id);
                Ok::<_, anyhow::Error>(())
            })?;
        }
        println!("{}", json);
        Ok(())
    }
}

fn multi_tag(name: &str, values: &[String]) -> Tag {
    let mut fields = vec![name.to_string()];
    fields.extend_from_slice(values);
    Tag::from_strings(fields)
}

fn parse_commit(value: &str) -> Result<String, String> {
    if value.len() != 40 {
        return Err("expected a 40-character Git commit hash".to_string());
    }
    Oid::from_str(value)
        .map(|oid| oid.to_string())
        .map_err(|err| err.to_string())
}

fn parse_pubkey(value: &str) -> Result<PublicKey, String> {
    PublicKey::try_from_hex_string(value, true)
        .or_else(|_| PublicKey::try_from_bech32_string(value, true))
        .map_err(|_| "expected a valid hex or npub1... public key".to_string())
}

fn parse_id(value: &str) -> Result<Id, String> {
    Id::try_from_hex_string(value)
        .or_else(|_| Id::try_from_bech32_string(value))
        .map_err(|_| "expected a 64-character event ID or note1... reference".to_string())
}

fn parse_relay(value: &str) -> Result<String, String> {
    let relay = RelayUrl::try_from_str(value)
        .map_err(|err| err.to_string())?
        .to_string();
    if !(relay.starts_with("ws://") || relay.starts_with("wss://")) {
        return Err("relay URL must use ws:// or wss://".to_string());
    }
    Ok(relay)
}

fn parse_commit_nonce(value: &str) -> Result<String, String> {
    commit_nonce_tag(value)
        .map(|_| value.to_string())
        .map_err(|_| "expected a Git worker/counter nonce such as 01-00000122".to_string())
}

fn parse_repository(value: &str) -> Result<NAddr, String> {
    let address = if value.starts_with("naddr1") {
        NAddr::try_from_bech32_string(value).map_err(|err| err.to_string())?
    } else {
        let fields: Vec<_> = value.splitn(3, ':').collect();
        if fields.len() != 3 || fields[0] != "30617" {
            return Err("expected 30617:<owner pubkey>:<identifier> or naddr1...".to_string());
        }
        NAddr {
            d: fields[2].to_string(),
            relays: vec![],
            kind: EventKind::RepositoryAnnouncement,
            author: parse_pubkey(fields[1])?,
        }
    };
    if u32::from(address.kind) != REPO_ANNOUNCEMENT_KIND || address.d.is_empty() {
        return Err(
            "repository address must identify a kind-30617 event with a nonempty identifier"
                .to_string(),
        );
    }
    Ok(address)
}

fn parse_ref(value: &str) -> Result<(String, String), String> {
    let (name, target) = value
        .split_once('=')
        .ok_or("expected <reference>=<commit hash> or HEAD=ref: refs/heads/<branch>")?;
    if name == "HEAD" {
        if let Some(branch) = target.strip_prefix("ref: ") {
            if branch.starts_with("refs/heads/") && Reference::is_valid_name(branch) {
                return Ok((name.to_string(), target.to_string()));
            }
            return Err("HEAD symbolic target must be a valid refs/heads/ reference".to_string());
        }
    } else if !(name.starts_with("refs/heads/") || name.starts_with("refs/tags/"))
        || !Reference::is_valid_name(name)
    {
        return Err("expected a valid refs/heads/ or refs/tags/ reference".to_string());
    }
    Ok((name.to_string(), parse_commit(target)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use gnostr_types::nostr::nip13::NIP13Event;

    const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    #[derive(Parser)]
    struct TestCli {
        #[command(flatten)]
        compose: Compose,
    }

    fn parse(args: &[&str]) -> Compose {
        TestCli::try_parse_from(std::iter::once("compose").chain(args.iter().copied()))
            .unwrap()
            .compose
    }

    #[test]
    fn composes_every_nip34_kind_with_valid_signatures_and_gnostr_tags() {
        let keys = Keys::generate();
        let public_key = keys.public_key().as_hex_string();
        let repository = format!("30617:{}:demo", public_key);
        let event = Id::mock().as_hex_string();
        let reference = format!("refs/heads/main={}", COMMIT);
        let patch = format!(
            "From {} Mon Sep 17 00:00:00 2001\nSubject: [PATCH] demo\n\nbody\n",
            COMMIT
        );
        let mut cases = vec![
            (
                vec![
                    "repo-announcement",
                    "--identifier",
                    "demo",
                    "--root-commit",
                    COMMIT,
                    "--clone-url",
                    "https://example.com/demo.git",
                    "--clone-url",
                    "ssh://example.com/demo.git",
                ],
                30617,
            ),
            (
                vec!["repo-state", "--identifier", "demo", "--ref", &reference],
                30618,
            ),
            (
                vec![
                    "patch",
                    "--repository",
                    &repository,
                    "--commit",
                    COMMIT,
                    "--content",
                    &patch,
                ],
                1617,
            ),
            (
                vec![
                    "pull-request",
                    "--repository",
                    &repository,
                    "--subject",
                    "demo",
                    "--commit",
                    COMMIT,
                    "--clone-url",
                    "https://example.com/demo.git",
                ],
                1618,
            ),
            (
                vec![
                    "pull-request-update",
                    "--repository",
                    &repository,
                    "--event",
                    &event,
                    "--author",
                    &public_key,
                    "--commit",
                    COMMIT,
                    "--clone-url",
                    "https://example.com/demo.git",
                ],
                1619,
            ),
            (
                vec!["issue", "--repository", &repository, "--content", "issue"],
                1621,
            ),
            (
                vec![
                    "reply",
                    "--repository",
                    &repository,
                    "--event",
                    &event,
                    "--author",
                    &public_key,
                    "--root-kind",
                    "1621",
                    "--content",
                    "reply",
                ],
                1111,
            ),
            (
                vec![
                    "reply",
                    "--legacy",
                    "--repository",
                    &repository,
                    "--event",
                    &event,
                    "--author",
                    &public_key,
                    "--root-kind",
                    "1621",
                    "--content",
                    "reply",
                ],
                1622,
            ),
            (
                vec!["grasp-list", "--server", "wss://grasp.example.com"],
                10317,
            ),
        ];
        for (status, kind) in [
            ("open", 1630),
            ("applied", 1631),
            ("closed", 1632),
            ("draft", 1633),
        ] {
            cases.push((
                vec![
                    "status",
                    status,
                    "--repository",
                    &repository,
                    "--event",
                    &event,
                    "--author",
                    &public_key,
                ],
                kind,
            ));
        }
        for (mut args, kind) in cases {
            args.extend(["--commit-nonce", "01-00000122"]);
            println!("========== COMPOSING NIP-34 KIND {} ==========", kind);
            let command = parse(&args);
            assert!(!command.publish);
            let event = command.build(&keys).unwrap();
            event.verify(None).unwrap();
            assert_eq!(u32::from(event.kind), kind);
            assert_eq!(event.pubkey, keys.public_key());
            assert!(event
                .tags
                .contains(&Tag::new_tag("commit-nonce", "01-00000122")));
            assert!(event.nonce_data().is_none());
            let mined = command.build_with_pow(&keys, 8).unwrap();
            mined.verify(None).unwrap();
            assert!(gnostr_types::nostr::get_leading_zero_bits(&mined.id.0) >= 8);
            assert_eq!(mined.pow(), 8);
            assert_eq!(mined.nonce_data().unwrap().1, 8);
            assert_eq!(mined.kind, event.kind);
            assert_eq!(mined.content, event.content);
            assert!(event.tags.iter().all(|tag| mined.tags.contains(tag)));
            let prefixed = command
                .build_with_mining(
                    &keys,
                    &gnostr_types::nostr::EventMining::prefix("f").unwrap(),
                )
                .unwrap();
            prefixed.verify(None).unwrap();
            assert!(prefixed.id.as_hex_string().starts_with('f'));
            assert_eq!(prefixed.pow(), 0);
            assert_eq!(prefixed.nonce_data().unwrap().1, 0);
            assert_eq!(prefixed.kind, event.kind);
            assert_eq!(prefixed.content, event.content);
            assert!(event.tags.iter().all(|tag| prefixed.tags.contains(tag)));
            assert_eq!(
                event
                    .tags
                    .iter()
                    .filter(|tag| **tag == Tag::new_tag("t", "gnostr"))
                    .count(),
                1
            );
            if matches!(kind, 1111 | 1617..=1633) {
                assert!(event.tags.contains(&Tag::new_tag("a", &repository)));
                assert!(event
                    .tags
                    .contains(&Tag::new_pubkey(keys.public_key(), None, None)));
            }
            match kind {
                30617 => assert!(event.tags.contains(&Tag::new(&[
                    "clone",
                    "https://example.com/demo.git",
                    "ssh://example.com/demo.git"
                ]))),
                30618 => assert!(event
                    .tags
                    .contains(&Tag::new_tag("HEAD", "ref: refs/heads/main"))),
                1617 => assert_eq!(event.content, patch),
                1618 | 1619 => assert!(gnostr_types::nostr::nip34::event_is_valid_pr_or_pr_update(
                    &event
                )),
                10317 => assert!(event
                    .tags
                    .contains(&Tag::new_tag("g", "wss://grasp.example.com/"))),
                1630..=1633 => assert!(event.tags.contains(&Tag::new_event(
                    Id::mock(),
                    None,
                    Some("root".to_string())
                ))),
                _ => {}
            }
            let decoded: Event =
                serde_json::from_str(&serde_json::to_string(&event).unwrap()).unwrap();
            decoded.verify(None).unwrap();
            assert_eq!(decoded.id, event.id);
        }
    }

    #[test]
    fn update_and_nested_reply_preserve_distinct_nip22_references() {
        let keys = Keys::generate();
        let public_key = keys.public_key().as_hex_string();
        let repository = format!("30617:{}:demo", public_key);
        let root = Id([1; 32]).as_hex_string();
        let parent = Id([2; 32]).as_hex_string();
        let parent_author = Keys::generate().public_key().as_hex_string();
        let update = parse(&[
            "pull-request-update",
            "--repository",
            &repository,
            "--event",
            &root,
            "--author",
            &public_key,
            "--commit",
            COMMIT,
            "--clone-url",
            "https://example.com/repo.git",
        ])
        .build(&keys)
        .unwrap();
        assert!(update.tags.contains(&Tag::new_tag("E", &root)));
        assert!(update.tags.contains(&Tag::new_tag("P", &public_key)));
        let reply = parse(&[
            "reply",
            "--repository",
            &repository,
            "--event",
            &root,
            "--author",
            &public_key,
            "--root-kind",
            "1621",
            "--reply-event",
            &parent,
            "--reply-author",
            &parent_author,
            "--reply-kind",
            "1111",
            "--content",
            "nested reply",
        ])
        .build(&keys)
        .unwrap();
        for tag in [
            Tag::new_tag("E", &root),
            Tag::new_tag("K", "1621"),
            Tag::new_tag("P", &public_key),
            Tag::new_tag("e", &parent),
            Tag::new_tag("k", "1111"),
            Tag::new_tag("p", &parent_author),
        ] {
            assert!(reply.tags.contains(&tag));
        }
    }

    #[test]
    fn validates_status_fields_refs_patch_content_and_publication_opt_in() {
        let keys = Keys::generate();
        let public_key = keys.public_key().as_hex_string();
        let repository = format!("30617:{}:demo", public_key);
        let event = Id::mock().as_hex_string();
        let applied = parse(&[
            "status",
            "applied",
            "--repository",
            &repository,
            "--event",
            &event,
            "--author",
            &public_key,
            "--merge-commit",
            COMMIT,
            "--applied-commit",
            COMMIT,
            "--applied-patch",
            &event,
        ])
        .build(&keys)
        .unwrap();
        for tag in [
            Tag::new_tag("merge-commit", COMMIT),
            Tag::new_tag("applied-as-commits", COMMIT),
            Tag::new_tag("r", COMMIT),
            Tag::new_tag("q", &event),
        ] {
            assert!(applied.tags.contains(&tag));
        }
        assert!(parse(&[
            "status",
            "closed",
            "--repository",
            &repository,
            "--event",
            &event,
            "--author",
            &public_key,
            "--merge-commit",
            COMMIT
        ])
        .build(&keys)
        .is_err());
        let reference = format!("refs/heads/main={}", COMMIT);
        assert!(parse(&[
            "repo-state",
            "--identifier",
            "demo",
            "--ref",
            &reference,
            "--ref",
            &reference
        ])
        .build(&keys)
        .is_err());
        assert!(parse(&[
            "patch",
            "--repository",
            &repository,
            "--commit",
            COMMIT,
            "--content",
            "not a format-patch"
        ])
        .build(&keys)
        .is_err());
        assert!(TestCli::try_parse_from(["compose", "--publish", "grasp-list"]).is_err());
        assert!(parse(&["--publish", "--relay", "ws://127.0.0.1:8080", "grasp-list"]).publish);
        for reference in [
            "refs/heads/../main=bad",
            "HEAD=ref: refs/tags/v1",
            "refs/heads/main=bad",
        ] {
            assert!(TestCli::try_parse_from([
                "compose",
                "repo-state",
                "--identifier",
                "demo",
                "--ref",
                reference
            ])
            .is_err());
        }
        assert!(TestCli::try_parse_from([
            "compose",
            "--relay",
            "https://example.com",
            "grasp-list"
        ])
        .is_err());
        assert!(TestCli::try_parse_from([
            "compose",
            "issue",
            "--repository",
            "not-a-repository",
            "--content",
            "issue"
        ])
        .is_err());
    }

    #[test]
    fn file_content_and_repository_naddr_are_preserved() {
        let keys = Keys::generate();
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("issue.md");
        fs::write(&path, "issue body\n\nsecond paragraph\n").unwrap();
        let repository = NAddr {
            d: "demo".to_string(),
            relays: vec![],
            kind: EventKind::RepositoryAnnouncement,
            author: keys.public_key(),
        };
        let event = parse(&[
            "--hashtag",
            "#gnostr",
            "issue",
            "--repository",
            &repository.as_bech32_string(),
            "--content-file",
            path.to_str().unwrap(),
        ])
        .build(&keys)
        .unwrap();
        assert_eq!(event.content, "issue body\n\nsecond paragraph\n");
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|tag| **tag == Tag::new_tag("t", "gnostr"))
                .count(),
            1
        );
        assert!(event.tags.contains(&Tag::new_tag(
            "a",
            &format!("30617:{}:demo", keys.public_key())
        )));
        assert!(parse(&[
            "issue",
            "--repository",
            &repository.as_bech32_string(),
            "--content-file",
            directory.path().join("missing").to_str().unwrap()
        ])
        .build(&keys)
        .is_err());
    }
}
