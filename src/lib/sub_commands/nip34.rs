use std::borrow::Cow;

use clap::{Args, Subcommand};
use nostr_sdk_0_32_0::prelude::*;

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
) -> Result<()> {
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
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let mut tags = vec![
        Tag::custom(TagKind::Custom(Cow::from("d")), vec![args.name.clone()]),
        Tag::custom(TagKind::Custom(Cow::from("name")), vec![args.name.clone()]),
        Tag::custom(
            TagKind::Custom(Cow::from("description")),
            vec![args.description.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("web")),
            vec![args.web_url.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("clone")),
            vec![args.clone_url.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("relays")),
            args.relays.clone(),
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("r")),
            vec![args.root_commit.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("maintainers")),
            args.maintainers.clone(),
        ),
    ];

    for hashtag in &args.hashtags {
        tags.push(Tag::custom(
            TagKind::Custom(Cow::from("t")),
            vec![hashtag.clone()],
        ));
    }

    let event = EventBuilder::new(Kind::Custom(30617), "", tags).to_pow_event(&keys, difficulty_target)?;

    let event_id = client.send_event(event).await?;

    println!("{}", event_id.to_bech32()?);

    Ok(())
}

async fn repo_state(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &RepoStateCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let mut tags = vec![Tag::custom(
        TagKind::Custom(Cow::from("d")),
        vec![args.identifier.clone()],
    )];

    for r in &args.refs {
        let parts: Vec<&str> = r.split('|').collect();
        if parts.len() == 2 {
            tags.push(Tag::custom(
                TagKind::Custom(Cow::from(parts[0])),
                vec![parts[1].to_string()],
            ));
        }
    }

    let event = EventBuilder::new(Kind::Custom(30618), "", tags).to_pow_event(&keys, difficulty_target)?;

    let event_id = client.send_event(event).await?;

    println!("{}", event_id.to_bech32()?);

    Ok(())
}

async fn patch(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &PatchCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let tags = vec![
        Tag::custom(
            TagKind::Custom(Cow::from("a")),
            vec![args.repo.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("commit")),
            vec![args.commit.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("parent-commit")),
            vec![args.parent_commit.clone()],
        ),
    ];

    let event = EventBuilder::new(Kind::Custom(1617), args.content.clone(), tags)
        .to_pow_event(&keys, difficulty_target)?;

    let event_id = client.send_event(event).await?;

    println!("{}", event_id.to_bech32()?);

    Ok(())
}

async fn pull_request(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &PullRequestCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let tags = vec![
        Tag::custom(
            TagKind::Custom(Cow::from("a")),
            vec![args.repo.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("subject")),
            vec![args.subject.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("branch-name")),
            vec![args.branch_name.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("merge-base")),
            vec![args.merge_base.clone()],
        ),
    ];

    let event = EventBuilder::new(Kind::Custom(1618), "", tags)
        .to_pow_event(&keys, difficulty_target)?;

    let event_id = client.send_event(event).await?;

    println!("{}", event_id.to_bech32()?);

    Ok(())
}

async fn issue(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &IssueCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let tags = vec![
        Tag::custom(
            TagKind::Custom(Cow::from("a")),
            vec![args.repo.clone()],
        ),
        Tag::custom(
            TagKind::Custom(Cow::from("subject")),
            vec![args.subject.clone()],
        ),
    ];

    let event = EventBuilder::new(Kind::Custom(1621), args.content.clone(), tags)
        .to_pow_event(&keys, difficulty_target)?;

    let event_id = client.send_event(event).await?;

    println!("{}", event_id.to_bech32()?);

    Ok(())
}

async fn status(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    args: &StatusCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let (kind, tags) = match &args.command {
        StatusSubCommand::Open(args) => (
            Kind::Custom(1630),
            vec![
                Tag::custom(
                    TagKind::Custom(Cow::from("e")),
                    vec![args.event_id.clone(), "root".to_string()],
                ),
                Tag::custom(
                    TagKind::Custom(Cow::from("a")),
                    vec![args.repo.clone()],
                ),
            ],
        ),
        StatusSubCommand::Applied(args) => (
            Kind::Custom(1631),
            vec![
                Tag::custom(
                    TagKind::Custom(Cow::from("e")),
                    vec![args.event_id.clone(), "root".to_string()],
                ),
                Tag::custom(
                    TagKind::Custom(Cow::from("a")),
                    vec![args.repo.clone()],
                ),
                Tag::custom(
                    TagKind::Custom(Cow::from("applied-as-commits")),
                    args.applied_as_commits.clone(),
                ),
            ],
        ),
        StatusSubCommand::Closed(args) => (
            Kind::Custom(1632),
            vec![
                Tag::custom(
                    TagKind::Custom(Cow::from("e")),
                    vec![args.event_id.clone(), "root".to_string()],
                ),
                Tag::custom(
                    TagKind::Custom(Cow::from("a")),
                    vec![args.repo.clone()],
                ),
            ],
        ),
        StatusSubCommand::Draft(args) => (
            Kind::Custom(1633),
            vec![
                Tag::custom(
                    TagKind::Custom(Cow::from("e")),
                    vec![args.event_id.clone(), "root".to_string()],
                ),
                Tag::custom(
                    TagKind::Custom(Cow::from("a")),
                    vec![args.repo.clone()],
                ),
            ],
        ),
    };

        let event = EventBuilder::new(kind, "", tags).to_pow_event(&keys, difficulty_target)?;

    

        let event_id = client.send_event(event).await?;

    

        println!("{}", event_id.to_bech32()?);

    

        Ok(())

    }
