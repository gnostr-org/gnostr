use std::sync::Arc;

use anyhow::Result;
use gnostr_asyncgit::{
    git2::Oid,
    sync::{add_note, default_notes_ref, remove_note, show_note, RepoPath},
};
use gnostr_ngit::{
    git::{oid_to_sha1, Repo},
    git_events::{
        event_is_cover_letter, event_is_patch_set_root, event_is_revision_root,
        event_is_valid_pr_or_pr_update, generate_cover_letter_and_patch_events,
        generate_git_note_event, generate_git_note_event_with_pow,
        generate_unsigned_pr_or_update_event, git_note_tags, patch_supports_commit_ids,
        KIND_PULL_REQUEST, KIND_PULL_REQUEST_UPDATE,
    },
    repo_ref::RepoRef,
};
use nostr_sdk::{Keys, NostrSigner};
use test_utils::{generate_repo_ref_event, git::GitTestRepo};

fn seeded_keys_from_oid(oid: &Oid) -> Result<Keys> {
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

fn mine_pow_commit(repo: &GitTestRepo) -> Result<String> {
    let mut config = repo.git_repo.config()?;
    config.set_str("user.name", "randymcmillan")?;
    config.set_str("user.email", "randymcmillan@example.com")?;

    let opts = gnostr_legit::gitminer::Options {
        threads: 1,
        target: "00".to_string(),
        message: vec!["proof-of-work commit".to_string()],
        repo: repo.dir.to_str().unwrap().to_string(),
        timestamp: time::OffsetDateTime::now_utc(),
        kind: None,
    };

    let mut miner = gnostr_legit::gitminer::Gitminer::new(opts).map_err(anyhow::Error::msg)?;
    miner.mine().map_err(anyhow::Error::msg)
}

fn mine_git_note(
    repo_path: &RepoPath,
    commit_id: Oid,
    note_base_message: &str,
    notes_ref: Option<&str>,
) -> Result<gnostr_asyncgit::sync::NoteInfo> {
    let mut nonce = 0u32;

    loop {
        let note_message = format!("{note_base_message} #{nonce}");
        let note_id = add_note(repo_path, commit_id, &note_message, notes_ref, true)?;
        let note = show_note(repo_path, commit_id, notes_ref)?.expect("note exists");
        println!(
            "matrix note attempt: nonce={nonce} note_id={note_id} annotated_id={} message={}",
            note.annotated_id, note.message
        );
        if note.note_id.to_string().starts_with('0') {
            assert_eq!(note.note_id, note_id);
            assert!(note.message.starts_with(note_base_message));
            return Ok(note);
        }
        nonce = nonce.wrapping_add(1);
    }
}

#[tokio::test]
async fn repo_announcement_round_trips_through_repo_ref() -> Result<()> {
    println!("[ngit] repo_announcement_round_trips_through_repo_ref");
    let repo_ref = repo_ref_fixture()?;
    let signer_keys = seeded_keys_from_oid(&Oid::from_str(&repo_ref.root_commit)?)?;
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
    println!("[ngit] cover_letter_and_patch_events_use_git_patch_kind");
    let (git_repo, repo) = repo_fixture()?;
    let repo_ref = repo_ref_fixture()?;
    let head = git_repo.git_repo.head()?.peel_to_commit()?.id();
    let parent = git_repo.git_repo.find_commit(head)?.parent(0)?.id();
    let root_commit = oid_to_sha1(&parent);
    let root_commit_str = root_commit.to_string();
    let commit = oid_to_sha1(&head);
    let signer: Arc<dyn NostrSigner> = Arc::new(seeded_keys_from_oid(&head)?);

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
    assert!(!event_is_patch_set_root(&events[1]));
    assert!(!event_is_revision_root(&events[1]));
    assert!(patch_supports_commit_ids(&events[1]));

    let cover_letter_id = events[0].id;
    let patch_parent = events[1]
        .tags
        .iter()
        .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e"))
        .expect("patch reply tag");
    assert_eq!(patch_parent.as_slice()[1], cover_letter_id.to_string());
    assert!(events[1].tags.iter().any(|tag| {
        tag.as_slice().first().map(|s| s.as_str()) == Some("r")
            && tag.as_slice().get(1).map(|s| s.as_str()) == Some(root_commit_str.as_str())
    }));
    Ok(())
}

#[tokio::test]
async fn pull_request_and_update_events_use_default_signer() -> Result<()> {
    println!("[ngit] pull_request_and_update_events_use_default_signer");
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

    let patch_events = generate_cover_letter_and_patch_events(
        Some(("example title".to_string(), "example description".to_string())),
        &repo,
        &[commit],
        &signer,
        &repo_ref,
        &None,
        &[],
    )
    .await?;

    let patch_root = patch_events[0].clone();
    let unsigned_pr = generate_unsigned_pr_or_update_event(
        &repo,
        &repo_ref,
        &keys.public_key(),
        Some(&patch_root),
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
    assert!(event_is_revision_root(&pr_event));
    assert!(pr_event
        .tags
        .iter()
        .any(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e")));

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
    assert!(!event_is_revision_root(&update_event));
    assert!(update_event
        .tags
        .iter()
        .any(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("E")));
    assert!(update_event
        .tags
        .iter()
        .any(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("P")));
    Ok(())
}

#[tokio::test]
async fn nip34_event_matrix_with_git_notes_attached() -> Result<()> {
    println!("[ngit] nip34_event_matrix_with_git_notes_attached");
    let cases = [
        ("plain-commit/plain-note/plain-event", false, false, false),
        ("plain-commit/plain-note/pow-event", false, false, true),
        ("plain-commit/mined-note/plain-event", false, true, false),
        ("plain-commit/mined-note/pow-event", false, true, true),
        ("mined-commit/plain-note/plain-event", true, false, false),
        ("mined-commit/plain-note/pow-event", true, false, true),
        ("mined-commit/mined-note/plain-event", true, true, false),
        ("mined-commit/mined-note/pow-event", true, true, true),
    ];

    for (label, mine_commit_flag, mine_note_flag, pow_event_flag) in cases {
        println!(
            "nip34 matrix start: label={label} mine_commit={mine_commit_flag} mine_note={mine_note_flag} pow_event={pow_event_flag}"
        );

        let git_repo = GitTestRepo::new("main")?;
        git_repo.populate()?;
        if mine_commit_flag {
            let mined_hash = mine_pow_commit(&git_repo)?;
            println!("nip34 matrix mined commit: {mined_hash}");
        }

        let repo = Repo::from_path(&git_repo.dir)?;
        let head = git_repo.git_repo.head()?.target().unwrap();
        let repo_path_owned: RepoPath = git_repo.dir.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let notes_ref = default_notes_ref(repo_path)?;
        let note_base_message = format!("{label} note");

        let note = if mine_note_flag {
            mine_git_note(repo_path, head, &note_base_message, Some(notes_ref.as_str()))?
        } else {
            let note_id = add_note(
                repo_path,
                head,
                &note_base_message,
                Some(notes_ref.as_str()),
                false,
            )?;
            let note = show_note(repo_path, head, Some(notes_ref.as_str()))?.expect("note exists");
            println!(
                "nip34 matrix note created: note_id={note_id} annotated_id={} message={}",
                note.annotated_id, note.message
            );
            note
        };

        assert_eq!(note.annotated_id, head);
        assert!(note.message.starts_with(&note_base_message));

        let repo_ref = repo_ref_fixture()?;
        let signer_keys = seeded_keys_from_oid(&head)?;
        let signer: Arc<dyn NostrSigner> = Arc::new(signer_keys.clone());
        let repo_announcement = repo_ref.to_event(&signer).await?;
        let commit = oid_to_sha1(&head);
        let patch_events = generate_cover_letter_and_patch_events(
            Some(("example title".to_string(), "example description".to_string())),
            &repo,
            &[commit],
            &signer,
            &repo_ref,
            &None,
            &[],
        )
        .await?;
        let git_note_event = if pow_event_flag {
            generate_git_note_event_with_pow(&note, &signer_keys, 4).await?
        } else {
            generate_git_note_event(&note, &signer).await?
        };

        let clone_url = git_repo.dir.to_str().unwrap().to_string();
        let clone_hints = vec![clone_url.as_str()];
        let patch_root = patch_events[0].clone();
        let unsigned_pr = generate_unsigned_pr_or_update_event(
            &repo,
            &repo_ref,
            &signer_keys.public_key(),
            Some(&patch_root),
            &None,
            &commit,
            &commit,
            None,
            &clone_hints,
            &[],
        )?;
        let pr_event = unsigned_pr.sign_with_keys(&signer_keys)?;
        let unsigned_update = generate_unsigned_pr_or_update_event(
            &repo,
            &repo_ref,
            &signer_keys.public_key(),
            Some(&pr_event),
            &None,
            &commit,
            &commit,
            None,
            &clone_hints,
            &[],
        )?;
        let update_event = unsigned_update.sign_with_keys(&signer_keys)?;

        println!("nip34 matrix repo announcement: {repo_announcement:#?}");
        println!("nip34 matrix patch events: {patch_events:#?}");
        println!("nip34 matrix git note event: {git_note_event:#?}");
        println!("nip34 matrix pull request event: {pr_event:#?}");
        println!("nip34 matrix pull request update event: {update_event:#?}");
        println!("nip34 matrix git note tags: {:?}", git_note_tags(&note)?);

        assert_eq!(repo_announcement.kind, nostr_sdk::Kind::GitRepoAnnouncement);
        assert_eq!(patch_events.len(), 2);
        assert_eq!(patch_events[0].kind, nostr_sdk::Kind::GitPatch);
        assert_eq!(patch_events[1].kind, nostr_sdk::Kind::GitPatch);
        assert!(event_is_cover_letter(&patch_events[0]));
        assert!(event_is_patch_set_root(&patch_events[0]));
        assert!(!event_is_cover_letter(&patch_events[1]));
        assert!(!event_is_patch_set_root(&patch_events[1]));
        assert!(patch_supports_commit_ids(&patch_events[1]));
        assert_eq!(git_note_event.kind, nostr_sdk::Kind::GitPatch);
        assert_eq!(git_note_event.content, note.message);
        assert!(git_note_event
            .tags
            .iter()
            .any(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e")));
        assert!(git_note_event
            .tags
            .iter()
            .any(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("commit")));
        assert_eq!(pr_event.kind, KIND_PULL_REQUEST);
        assert!(event_is_valid_pr_or_pr_update(&pr_event));
        assert!(event_is_revision_root(&pr_event));
        assert_eq!(update_event.kind, KIND_PULL_REQUEST_UPDATE);
        assert!(event_is_valid_pr_or_pr_update(&update_event));
        assert!(!event_is_revision_root(&update_event));

        remove_note(repo_path, head, Some(notes_ref.as_str()))?;
        println!("nip34 matrix done: label={label}");
    }

    Ok(())
}
