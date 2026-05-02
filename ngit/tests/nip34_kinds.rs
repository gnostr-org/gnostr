use std::sync::Arc;

use anyhow::Result;
use gnostr_ngit::{
    git::{oid_to_sha1, Repo},
    git_events::{
        event_is_cover_letter, event_is_patch_set_root, event_is_revision_root,
        event_is_valid_pr_or_pr_update, generate_cover_letter_and_patch_events,
        generate_unsigned_pr_or_update_event, patch_supports_commit_ids, KIND_PULL_REQUEST,
        KIND_PULL_REQUEST_UPDATE,
    },
    repo_ref::RepoRef,
};
use nostr_sdk::{Keys, NostrSigner};
use test_utils::{generate_repo_ref_event, git::GitTestRepo};

fn seeded_keys_from_oid(oid: &git2::Oid) -> Result<Keys> {
    Ok(Keys::parse(&format!("{:0>64}", oid))?)
}

fn repo_fixture() -> Result<(GitTestRepo, Repo)> {
    let git_repo = GitTestRepo::new("main")?;
    git_repo.populate_minus_1()?;
    let repo = Repo::from_path(&git_repo.dir)?;
    Ok((git_repo, repo))
}

fn repo_ref_fixture() -> Result<RepoRef> {
    Ok(RepoRef::try_from((generate_repo_ref_event(), None))?)
}

#[tokio::test]
async fn repo_announcement_round_trips_through_repo_ref() -> Result<()> {
    let repo_ref = repo_ref_fixture()?;
    let signer_keys = Keys::generate();
    let signer: Arc<dyn NostrSigner> = Arc::new(signer_keys.clone());

    let event = repo_ref.to_event(&signer).await?;
    println!("repo announcement event: {event:#?}");

    assert_eq!(event.kind, nostr_sdk::Kind::GitRepoAnnouncement);
    let parsed = RepoRef::try_from((event.clone(), Some(signer_keys.public_key())))?;
    assert_eq!(parsed.identifier, repo_ref.identifier);
    assert_eq!(parsed.root_commit, repo_ref.root_commit);
    Ok(())
}

#[tokio::test]
async fn cover_letter_and_patch_events_use_git_patch_kind() -> Result<()> {
    let (git_repo, repo) = repo_fixture()?;
    let repo_ref = repo_ref_fixture()?;
    let head = git_repo.git_repo.head()?.peel_to_commit()?.id();
    let parent = git_repo.git_repo.find_commit(head)?.parent(0)?.id();
    let root_commit = oid_to_sha1(&parent);
    let commit = oid_to_sha1(&head);
    let keys = seeded_keys_from_oid(&head)?;
    let signer: Arc<dyn NostrSigner> = Arc::new(keys.clone());

    let events = generate_cover_letter_and_patch_events(
        Some(("example title".to_string(), "example description".to_string())),
        &repo,
        &[commit],
        &signer,
        &repo_ref,
        &None,
        &[],
    )
    .await?;

    println!("nip34 patch events: {events:#?}");

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, nostr_sdk::Kind::GitPatch);
    assert!(event_is_cover_letter(&events[0]));
    assert!(event_is_patch_set_root(&events[0]));
    assert_eq!(events[1].kind, nostr_sdk::Kind::GitPatch);
    assert!(!event_is_cover_letter(&events[1]));
    assert!(event_is_patch_set_root(&events[1]));
    assert!(patch_supports_commit_ids(&events[1]));

    let patch_root = events[1]
        .tags
        .iter()
        .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e"))
        .expect("patch root tag");
    assert_eq!(patch_root.as_slice()[1], format!("{:0>64}", root_commit));
    Ok(())
}

#[tokio::test]
async fn pull_request_and_update_events_use_commit_seeded_signer() -> Result<()> {
    let (git_repo, repo) = repo_fixture()?;
    git_repo.create_branch("feature")?;
    git_repo.checkout("feature")?;
    std::fs::write(git_repo.dir.join("feature.md"), "some content")?;
    git_repo.stage_and_commit("add feature.md")?;

    let head = git_repo.git_repo.head()?.peel_to_commit()?.id();
    let commit = oid_to_sha1(&head);
    let keys = seeded_keys_from_oid(&head)?;
    let signer: Arc<dyn NostrSigner> = Arc::new(keys.clone());
    let repo_ref = repo_ref_fixture()?;
    let clone_url = git_repo.dir.to_str().unwrap().to_string();
    let clone_hints = vec![clone_url.as_str()];

    let unsigned_pr = generate_unsigned_pr_or_update_event(
        &repo,
        &repo_ref,
        &keys.public_key(),
        None,
        &None,
        &commit,
        &commit,
        None,
        &clone_hints,
        &[],
    )?;
    let pr_event = unsigned_pr.sign_with_keys(&keys)?;
    println!("pull request event: {pr_event:#?}");

    assert_eq!(pr_event.kind, KIND_PULL_REQUEST);
    assert!(event_is_valid_pr_or_pr_update(&pr_event));
    assert!(!event_is_revision_root(&pr_event));

    let unsigned_update = generate_unsigned_pr_or_update_event(
        &repo,
        &repo_ref,
        &keys.public_key(),
        Some(&pr_event),
        &None,
        &commit,
        &commit,
        None,
        &clone_hints,
        &[],
    )?;
    let update_event = unsigned_update.sign_with_keys(&keys)?;
    println!("pull request update event: {update_event:#?}");

    assert_eq!(update_event.kind, KIND_PULL_REQUEST_UPDATE);
    assert!(event_is_valid_pr_or_pr_update(&update_event));
    assert!(event_is_revision_root(&update_event));
    Ok(())
}
