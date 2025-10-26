use std::borrow::Cow;

use clap::Args;
use nostr_sdk_0_32_0::prelude::*;

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct CustomEventCommand {
    ///
    ///
    /// NIP-01: Basic Text Note
    ///
    /// > gnostr custom-event -k 1 -c "Hello Nostr!" -r wss://relay.example.com
    /// 
    ///
    /// NIP-10: Threaded Notes (Reply)
    /// 
    ///	Reply to an event with ID 'abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789'
    /// 
    /// > gnostr custom-event -k 1 -c "This is a reply." -r wss://relay.example.com -t "in_reply_to|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
    /// 
    ///
    /// NIP-25: Reactions
    /// 
    ///	React to an event with ID 'abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789' with a 👍 emoji
    /// 
    /// > gnostr custom-event -k 7 -r wss://relay.example.example.com -t "reference|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789" -t "+"
    /// 
    ///
    /// - NIP-33: Parameterized Replaceable Events (Profile Update)
    /// 
    /// -- Update profile name and picture using tags
    /// 
    /// > gnostr custom-event -k 0 -c '{"name": "Bob", "picture": "https://example.com/bob.jpg"}' -r wss://relay.example.com -t "name|Bob" -t "picture|https://example.com/bob.jpg"
    /// 
    ///
    /// - NIP-34: Git Collaboration - Repository Announcement
    /// 
    /// -- Announce a git repository
    /// 
    /// > gnostr custom-event -k 34000 -c '{"name": "my-awesome-repo", "description": "A cool project."}' -r wss://relay.example.com -t "r|https://github.com/example/repo"
    /// 
    ///
    /// - NIP-34: Git Collaboration - Patch Announcement
    /// 
    /// -- Announce a patch for a repository
    /// 
    /// > gnostr custom-event -k 34001 -c "--- a/main.rs\n+++ b/main.rs\n@@ -1 +1 @@\n-let x = 5;\n+let x = 10;" -r wss://relay.example.com -t "r|https://github.com/example/repo" -t "patch_hash|deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef" -t "branch|main"
    /// 
    ///
    /// - NIP-57: Lightning Zaps (Zap Request)
    /// 
    /// -- Send a zap request for 1000 sats to a recipient
    /// 
    /// > gnostr custom-event -k 9735 -c '{"amount": 1000, "bolt11": "lnbc100..."}' -r wss://relay.example.com -t "p|recipient_pubkey..." -t "amount|1000"
    /// 
    #[arg(short, long)]
    kind: u16,

    /// Note content
    #[arg(short, long)]
    content: Option<String>,

    /// Tags are key-value pairs used to add metadata to events.
    /// They can follow specific NIPs or be custom.
    ///
    /// Example of a custom tag format (e.g., for NIP-12):
    /// "d|my-custom-tag-name"
    ///
    /// Example of an 'a' tag (e.g., for NIP-33 Parameterized Replaceable Events):
    /// "a|30001:b2d670de53b27691c0c3400225b65c35a26d06093bcc41f48ffc71e0907f9d4a:bookmark|wss://nostr.oxtr.dev"
    ///
    /// The format is generally `TAG_KIND|TAG_VALUE1|TAG_VALUE2|...`
    ///
    /// - NIP-34: Git Collaboration - Repository Announcement
    ///
    /// -- Announce a git repository
    ///
    /// > gnostr custom-event -k 30617 -c '{"name": "my-awesome-repo", "description": "A cool project for demonstrating NIP-34."}' --tags "d|my-awesome-repo" --tags "name|My Awesome Repo" --tags "description|A cool project for demonstrating NIP-34." --tags "web|https://example.com/my-awesome-repo" --tags "clone|https://github.com/example/my-awesome-repo.git" --tags "relays|wss://relay.example.com" --tags "r|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789" --tags "maintainers|pubkey1..." --tags "t|personal-fork" --tags "example-repo"
    ///
    /// - NIP-34: Repository State Announcement
    ///
    /// -- Announce the current state of branches and tags for a repository.
    ///
    /// > gnostr custom-event -k 30618 --tags "d|my-awesome-repo" --tags "refs/heads/main|ref: refs/heads/main" --tags "refs/tags/v1.0.0|ref: refs/tags/v1.0.0" --tags "HEAD|ref: refs/heads/main"
    ///
    /// - NIP-34: Patch Announcement (Kind 1617)
    ///
    /// -- Announce a patch for a repository.
    ///
    /// > gnostr custom-event -k 1617 --content "--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-let x = 5;\n+let x = 10;" --tags "a|30617:my-awesome-repo" --tags "r|wss://relay.example.com" --tags "p|recipient_pubkey..." --tags "t|root" --tags "commit|deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef" --tags "parent-commit|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
    ///
    /// - NIP-34: Pull Request Announcement (Kind 1618)
    ///
    /// -- Announce a pull request.
    ///
    /// > gnostr custom-event -k 1618 --content "" --tags "a|30617:my-awesome-repo" --tags "r|wss://relay.example.com" --tags "p|recipient_pubkey..." --tags "subject|Implement new feature X" --tags "t|feature" --tags "t|enhancement" --tags "c|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789" --tags "clone|https://github.com/example/my-awesome-repo.git" --tags "branch-name|feature/new-x" --tags "merge-base|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456780"
    ///
    /// - NIP-34: Pull Request Update (Kind 1619)
    ///
    /// -- Update an existing pull request.
    ///
    /// > gnostr custom-event -k 1619 --content "" --tags "a|30617:my-awesome-repo" --tags "r|wss://relay.example.com" --tags "p|recipient_pubkey..." --tags "E|pr_event_id_of_the_pr_tip_being_updated" --tags "c|fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210" --tags "clone|https://github.com/example/my-awesome-repo.git" --tags "merge-base|abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456780"
    ///
    /// - NIP-34: Issue Announcement (Kind 1621)
    ///
    /// -- Announce an issue.
    ///
    /// > gnostr custom-event -k 1621 --content "The login button is not working on the staging environment. It returns a 500 error." --tags "a|30617:my-awesome-repo" --tags "p|recipient_pubkey..." --tags "subject|Login button returns 500 error" --tags "t|bug" --tags "t|staging"
    ///
    /// - NIP-34: Status Events (Kinds 1630-1633)
    ///
    /// -- Update the status of another event (e.g., a patch, PR, or issue).
    ///
    /// Kind 1630 (Open):
    /// > gnostr custom-event -k 1630 --tags "e|issue_event_id|root" --tags "a|30617:my-awesome-repo"
    ///
    /// Kind 1631 (Applied/Merged/Resolved):
    /// > gnostr custom-event -k 1631 --tags "e|patch_event_id|root" --tags "a|30617:my-awesome-repo" --tags "applied-as-commits|commit1_hash,commit2_hash" --tags "r|wss://relay.example.com"
    ///
    /// Kind 1632 (Closed):
    /// > gnostr custom-event -k 1632 --tags "e|pr_event_id|root" --tags "a|30617:my-awesome-repo"
    ///
    /// Kind 1633 (Draft):
    /// > gnostr custom-event -k 1633 --tags "e|patch_event_id|root" --tags "a|30617:my-awesome-repo"
    ///
    /// - NIP-34: User Grasp List (Kind 10317)
    ///
    /// -- List preferred "grasp servers" for NIP-34 activities.
    ///
    /// > gnostr custom-event -k 10317 --tags "g|wss://grasp.example.com" --tags "g|wss://another-grasp.example.com"
    ///
    ///		///
    ///
	/// Nostr Event Kind (NIP-01, NIP-10, NIP-25, etc.). See https://github.com/nostr-protocol/nips for a full list.
    #[arg(short, long, action = clap::ArgAction::Append)]
    tags: Vec<String>,

    // Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn create_custom_event(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &CustomEventCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    // Parse kind input
    let kind = Kind::Custom(sub_command_args.kind);

    // Set content
    let content = sub_command_args
        .content
        .clone()
        .unwrap_or_else(|| String::from(""));

    // Set up tags
    let mut tags: Vec<Tag> = vec![];

    for tag in sub_command_args.tags.clone().iter() {
        let parts: Vec<String> = tag.split('|').map(String::from).collect();
        let tag_kind = parts.first().unwrap().clone();
        tags.push(Tag::custom(
            TagKind::Custom(Cow::from(tag_kind)),
            parts[1..].to_vec(),
        ));
    }

    // Initialize event builder
    let event = EventBuilder::new(kind, content, tags).to_pow_event(&keys, difficulty_target)?;

    // Publish event
    let event_id = client.send_event(event).await?;

    if !sub_command_args.hex {
        println!("{}", event_id.to_bech32()?);
    } else {
        println!("{}", event_id.to_hex());
    }

    Ok(())
}
