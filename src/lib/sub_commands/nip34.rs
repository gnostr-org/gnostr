use std::borrow::Cow;

use clap::{Args, Subcommand};
use anyhow::{Result, Error as AnyhowError};
use crate::types::{
    Client, Event, EventKind, Id, Keys, TagV3 as Tag, PrivateKey, PreEventV3, Unixtime, KeySigner,
    Signer,
};

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct Nip34Command {
    #[command(subcommand)]
    command: Nip34SubCommand,
}

#[derive(Subcommand, Debug)]
enum Nip34SubCommand {
    /// Announce a git repository
    RepoAnnouncement(RepoAnnouncementCommand),
    /// Announce the current state of branches and tags for a repository
    RepoState(RepoStateCommand),
    /// Announce a patch
    Patch(PatchCommand),
    /// Announce a pull request
    PullRequest(PullRequestCommand),
    /// Announce an issue
    Issue(IssueCommand),
    /// Update the status of another event
    Status(StatusCommand),
}

#[derive(Args, Debug)]
struct RepoAnnouncementCommand {
    /// Repository name
    #[arg(long)]
    name: String,
    /// Repository description
    #[arg(long)]
    description: String,
    /// Repository clone URL
    #[arg(long)]
    clone_url: String,
    /// Repository web URL
    #[arg(long)]
    web_url: String,
    /// Relays
    #[arg(long, action = clap::ArgAction::Append)]
    relays: Vec<String>,
    /// Maintainers
    #[arg(long, action = clap::ArgAction::Append)]
    maintainers: Vec<String>,
    /// Root commit
    #[arg(long, default_value = "4b825dc642cb6eb9a060e54bf8d69288fbee4904")]
    root_commit: String,
    /// Hashtags
    #[arg(long, action = clap::ArgAction::Append)]
    hashtags: Vec<String>,
}

#[derive(Args, Debug)]
struct RepoStateCommand {
    /// Repository identifier (d-tag)
    #[arg(long)]
    identifier: String,
    /// Git references (e.g., "refs/heads/main|...")
    #[arg(long, action = clap::ArgAction::Append)]
    refs: Vec<String>,
}

#[derive(Args, Debug)]
struct PatchCommand {
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
    /// Commit hash
    #[arg(long)]
    commit: String,
    /// Parent commit hash
    #[arg(long)]
    parent_commit: String,
    /// Patch content
    #[arg(long)]
    content: String,
}

#[derive(Args, Debug)]
struct PullRequestCommand {
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
    /// Subject
    #[arg(long)]
    subject: String,
    /// Branch name
    #[arg(long)]
    branch_name: String,
    /// Merge base
    #[arg(long)]
    merge_base: String,
}

#[derive(Args, Debug)]
struct IssueCommand {
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
    /// Subject
    #[arg(long)]
    subject: String,
    /// Content
    #[arg(long)]
    content: String,
}

#[derive(Args, Debug)]
struct StatusCommand {
    #[command(subcommand)]
    command: StatusSubCommand,
}

#[derive(Subcommand, Debug)]
enum StatusSubCommand {
    /// Open
    Open(StatusOpenCommand),
    /// Applied/Merged/Resolved
    Applied(StatusAppliedCommand),
    /// Closed
    Closed(StatusClosedCommand),
    /// Draft
    Draft(StatusDraftCommand),
}

#[derive(Args, Debug)]
struct StatusOpenCommand {
    /// Event ID
    #[arg(long)]
    event_id: String,
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
}

#[derive(Args, Debug)]
struct StatusAppliedCommand {
    /// Event ID
    #[arg(long)]
    event_id: String,
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
    /// Applied as commits
    #[arg(long, action = clap::ArgAction::Append)]
    applied_as_commits: Vec<String>,
}

#[derive(Args, Debug)]
struct StatusClosedCommand {
    /// Event ID
    #[arg(long)]
    event_id: String,
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
}

#[derive(Args, Debug)]
struct StatusDraftCommand {
    /// Event ID
    #[arg(long)]
    event_id: String,
    /// Repository identifier (a-tag)
    #[arg(long)]
    repo: String,
}

pub async fn launch(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &Nip34Command,
) -> Result<(), AnyhowError> {
    match &sub_command_args.command {
        Nip34SubCommand::RepoAnnouncement(args) => {
            repo_announcement(private_key, relays, difficulty_target, args).await?;
        }
        Nip34SubCommand::RepoState(args) => {
            repo_state(private_key, relays, difficulty_target, args).await?;
        }
        Nip34SubCommand::Patch(args) => {
            patch(private_key, relays, difficulty_target, args).await?;
        }
        Nip34SubCommand::PullRequest(args) => {
            pull_request(private_key, relays, difficulty_target, args).await?;
        }
        Nip34SubCommand::Issue(args) => {
            issue(private_key, relays, difficulty_target, args).await?;
        }
        Nip34SubCommand::Status(args) => {
            status(private_key, relays, difficulty_target, args).await?;
        }
    }
    Ok(())
}

async fn repo_announcement(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &RepoAnnouncementCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let mut tags = vec![
        Tag::new(&["d", &args.name]),
        Tag::new(&["name", &args.name]),
        Tag::new(&["description", &args.description]),
        Tag::new(&["web", &args.web_url]),
        Tag::new(&["clone", &args.clone_url]),
        Tag::new(&["relays", &args.relays.join(" ")]),
        Tag::new(&["r", &args.root_commit]),
        Tag::new(&["maintainers", &args.maintainers.join(" ")]),
    ];

    for hashtag in &args.hashtags {
        tags.push(Tag::new(&["t", hashtag]));
    }

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(30617),
        tags,
        content: "".to_string(),
    };
    
    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    println!("{}", event_id.as_bech32_string());

    Ok(())
}

async fn repo_state(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &RepoStateCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let mut tags = vec![Tag::new(&["d", &args.identifier])];

    for r in &args.refs {
        let parts: Vec<&str> = r.split('|').collect();
        if parts.len() == 2 {
            tags.push(Tag::new(&[parts[0], parts[1]]));
        }
    }

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(30618),
        tags,
        content: "".to_string(),
    };
    
    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    println!("{}", event_id.as_bech32_string());

    Ok(())
}

async fn patch(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &PatchCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let tags = vec![
        Tag::new(&["a", &args.repo]),
        Tag::new(&["commit", &args.commit]),
        Tag::new(&["parent-commit", &args.parent_commit]),
    ];

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(1617),
        tags,
        content: args.content.clone(),
    };
    
    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    println!("{}", event_id.as_bech32_string());

    Ok(())
}

async fn pull_request(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &PullRequestCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let tags = vec![
        Tag::new(&["a", &args.repo]),
        Tag::new(&["subject", &args.subject]),
        Tag::new(&["branch-name", &args.branch_name]),
        Tag::new(&["merge-base", &args.merge_base]),
    ];

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(1618),
        tags,
        content: "".to_string(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;
    
    let event_id = client.send_event(event).await?;
    println!("{}", event_id.as_bech32_string());

    Ok(())
}

async fn issue(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &IssueCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let tags = vec![
        Tag::new(&["a", &args.repo]),
        Tag::new(&["subject", &args.subject]),
    ];

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(1621),
        tags,
        content: args.content.clone(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    println!("{}", event_id.as_bech32_string());

    Ok(())
}

async fn status(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &StatusCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let (kind, tags) = match &args.command {
        StatusSubCommand::Open(args) => (
            EventKind::from(1630),
            vec![
                Tag::new(&["e", &args.event_id, "root"]),
                Tag::new(&["a", &args.repo]),
            ],
        ),
        StatusSubCommand::Applied(args) => (
            EventKind::from(1631),
            vec![
                Tag::new(&["e", &args.event_id, "root"]),
                Tag::new(&["a", &args.repo]),
                Tag::new(&["applied-as-commits", &args.applied_as_commits.join(",")]),
            ],
        ),
        StatusSubCommand::Closed(args) => (
            EventKind::from(1632),
            vec![
                Tag::new(&["e", &args.event_id, "root"]),
                Tag::new(&["a", &args.repo]),
            ],
        ),
        StatusSubCommand::Draft(args) => (
            EventKind::from(1633),
            vec![
                Tag::new(&["e", &args.event_id, "root"]),
                Tag::new(&["a", &args.repo]),
            ],
        ),
    };

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind,
        tags,
        content: "".to_string(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    println!("{}", event_id.as_bech32_string());

    Ok(())
}
