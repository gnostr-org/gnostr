use std::sync::Arc;

use anyhow::Result;
use gnostr_asyncgit::sync::{add_note, default_notes_ref, list_notes, remove_note, show_note, RepoPath};
use gnostr_asyncgit::git2::Oid;
use gnostr_legit::gitminer::{Gitminer, Options as LegitOptions};
use gnostr_ngit::{
    git::{oid_to_sha1, Repo},
    git_events::{
        event_is_cover_letter, event_is_patch_set_root, event_is_revision_root,
        event_is_valid_pr_or_pr_update, generate_cover_letter_and_patch_events,
        generate_git_note_event, generate_git_note_event_with_pow, generate_unsigned_pr_or_update_event, git_note_event_id,
        git_note_tags, patch_supports_commit_ids, KIND_PULL_REQUEST, KIND_PULL_REQUEST_UPDATE,
    },
    repo_ref::RepoRef,
};
use nostr_sdk::{Keys, NostrSigner};
use serial_test::serial;
use test_utils::{generate_repo_ref_event, git::GitTestRepo};
use time::OffsetDateTime;
use std::sync::Once;

fn init_test_log() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        let _ = env_logger::builder()
            .parse_default_env()
            .is_test(true)
            .filter_module("ureq", log::LevelFilter::Off)
            .filter_module("serial_test", log::LevelFilter::Off)
            .filter_module("mio", log::LevelFilter::Off)
            .filter_module("tungstenite", log::LevelFilter::Off)
            .filter_module("tokio_tungstenite", log::LevelFilter::Off)
            .filter_level(log::LevelFilter::Info)
            .try_init();
    });
}

fn seeded_keys_from_oid(oid: &Oid) -> Result<Keys> {
    Ok(Keys::parse(&format!("{:0>64}", oid))?)
}

fn repo_fixture() -> Result<(GitTestRepo, Repo)> {
    let git_repo = GitTestRepo::new("main")?;
    git_repo.populate_minus_1()?;
    let mined_hash = mine_pow_commit(&git_repo)?;
    let repo = Repo::from_path(&git_repo.dir)?;
    println!("pow commit mined for fixture: {mined_hash}");
    Ok((git_repo, repo))
}

fn repo_ref_fixture() -> Result<RepoRef> {
    Ok(RepoRef::try_from((generate_repo_ref_event(), None))?)
}

fn mine_pow_commit(repo: &GitTestRepo) -> Result<String> {
    let mut config = repo.git_repo.config()?;
    config.set_str("user.name", "randymcmillan")?;
    config.set_str("user.email", "randymcmillan@example.com")?;

    let opts = LegitOptions {
        threads: 1,
        target: "00".to_string(),
        message: vec!["proof-of-work commit".to_string()],
        repo: repo.dir.to_str().unwrap().to_string(),
        timestamp: OffsetDateTime::now_utc(),
        kind: None,
    };

    let mut miner = Gitminer::new(opts).map_err(anyhow::Error::msg)?;
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
            "note mining attempt: nonce={nonce} note_id={note_id} annotated_id={} message={}",
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
#[serial]
#[ignore]
async fn real_repo_git_notes_workflow_creates_signed_event() -> Result<()> {
    println!("[ngit] real_repo_git_notes_workflow_creates_signed_event");
    init_test_log();
    let repo = GitTestRepo::new("main")?;
    repo.populate()?;
    let mined_hash = mine_pow_commit(&repo)?;

    let head = repo.git_repo.head()?.target().unwrap();
    let repo_path_owned: RepoPath = repo.dir.as_os_str().to_str().unwrap().into();
    let repo_path: &RepoPath = &repo_path_owned;

    let notes_ref = default_notes_ref(repo_path)?;
    println!("notes default ref: {notes_ref}");

    let note_text = "nip34:git note protocol example:deterministically linked git note";

    let note_id = add_note(
        repo_path,
        head,
        note_text,
        Some(notes_ref.as_str()),
        false,
    )?;
    println!(
        "notes created: note_id={note_id} annotated_id={head} notes_ref={notes_ref} message={note_text}"
    );
    println!("pow commit mined: {mined_hash}");

    let note = show_note(repo_path, head, Some(notes_ref.as_str()))?.expect("note exists");
    println!("notes show: {note:#?}");

    let notes = list_notes(repo_path, Some(notes_ref.as_str()))?;
    println!("notes list: {notes:#?}");

    let signer: Arc<dyn NostrSigner> = Arc::new(Keys::generate());
    let event = generate_git_note_event(&note, &signer).await?;
    println!("nip34 git note event: {event:#?}");
    println!(
        "nip34 git note event root e tag: {:?}",
        event
            .tags
            .iter()
            .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e"))
            .map(|tag| tag.as_slice().to_vec())
    );
    println!(
        "nip34 git note event commit tag: {:?}",
        event
            .tags
            .iter()
            .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("commit"))
            .map(|tag| tag.as_slice().to_vec())
    );
    let expected_root = seeded_keys_from_oid(&head)?.public_key().to_string();

    assert_eq!(event.kind, nostr_sdk::Kind::TextNote);
    assert_eq!(event.content, note_text);
    assert_eq!(
        event
            .tags
            .iter()
            .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e"))
            .expect("e tag")
            .as_slice()[1],
        expected_root
    );
    assert_eq!(
        git_note_event_id(&head.to_string())?.to_hex(),
        expected_root
    );
    assert_eq!(git_note_tags(&note)?.len(), 6);

    remove_note(repo_path, head, Some(notes_ref.as_str()))?;
    println!("notes removed: annotated_id={head}");
    assert!(show_note(repo_path, head, Some(notes_ref.as_str()))?.is_none());

    Ok(())
}

#[tokio::test]
#[serial]
#[ignore]
async fn nip34_examples_for_all_kinds() -> Result<()> {
    println!("[ngit] nip34_examples_for_all_kinds");
    init_test_log();
    let repo_ref = repo_ref_fixture()?;
    let (git_repo, repo) = repo_fixture()?;
    let head = git_repo.git_repo.head()?.peel_to_commit()?.id();
    let commit = oid_to_sha1(&head);
    let signer_keys = seeded_keys_from_oid(&head)?;
    let signer: Arc<dyn NostrSigner> = Arc::new(signer_keys.clone());

    let repo_announcement = repo_ref.to_event(&signer).await?;
    println!("repo announcement event: {repo_announcement:#?}");

    let note_repo_path_owned: RepoPath = git_repo.dir.as_os_str().to_str().unwrap().into();
    let note_repo_path: &RepoPath = &note_repo_path_owned;
    let notes_ref = default_notes_ref(note_repo_path)?;
    let note_text = "nip34:git note protocol example:deterministically linked git note";
    let note_id = add_note(note_repo_path, head, note_text, Some(notes_ref.as_str()), false)?;
    let note = show_note(note_repo_path, head, Some(notes_ref.as_str()))?.expect("note exists");
    let git_note_event = generate_git_note_event(&note, &signer).await?;
    println!(
        "notes created: note_id={note_id} annotated_id={head} notes_ref={notes_ref} message={note_text}"
    );
    println!("git note event: {git_note_event:#?}");

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
    println!("nip34 patch events: {patch_events:#?}");

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
    println!("pull request event: {pr_event:#?}");

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
    println!("pull request update event: {update_event:#?}");

    assert_eq!(repo_announcement.kind, nostr_sdk::Kind::GitRepoAnnouncement);
    assert_eq!(git_note_event.kind, nostr_sdk::Kind::TextNote);
    assert!(patch_events.len() >= 2);
    assert_eq!(patch_events[0].kind, nostr_sdk::Kind::GitPatch);
    assert!(event_is_cover_letter(&patch_events[0]));
    assert!(event_is_patch_set_root(&patch_events[0]));
    assert_eq!(patch_events[1].kind, nostr_sdk::Kind::GitPatch);
    assert!(!event_is_patch_set_root(&patch_events[1]));
    assert!(!event_is_revision_root(&patch_events[1]));
    assert!(patch_supports_commit_ids(&patch_events[1]));
    assert_eq!(pr_event.kind, KIND_PULL_REQUEST);
    assert!(event_is_valid_pr_or_pr_update(&pr_event));
    assert!(event_is_revision_root(&pr_event));
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
    remove_note(note_repo_path, head, Some(notes_ref.as_str()))?;
    Ok(())
}

#[tokio::test]
#[serial]
#[ignore]
async fn nip34_event_matrix_covers_commit_note_and_pow_variants() -> Result<()> {
    println!("[ngit] nip34_event_matrix_covers_commit_note_and_pow_variants");
    init_test_log();

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
            "matrix case start: label={label} mine_commit={mine_commit_flag} mine_note={mine_note_flag} pow_event={pow_event_flag}"
        );
        let repo = GitTestRepo::new("main")?;
        repo.populate()?;
        if mine_commit_flag {
            let mined_hash = mine_pow_commit(&repo)?;
            println!("matrix case mined commit: {mined_hash}");
        }

        let head = repo.git_repo.head()?.target().unwrap();
        let repo_path_owned: RepoPath = repo.dir.as_os_str().to_str().unwrap().into();
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
                "matrix case note created: note_id={note_id} annotated_id={} message={}",
                note.annotated_id, note.message
            );
            note
        };

        assert_eq!(note.annotated_id, head);
        assert!(note.message.starts_with(&note_base_message));

        let signer_keys = Keys::generate();
        let signer: Arc<dyn NostrSigner> = Arc::new(signer_keys.clone());
        let event = if pow_event_flag {
            generate_git_note_event_with_pow(&note, &signer_keys, 4).await?
        } else {
            generate_git_note_event(&note, &signer).await?
        };
        let expected_root = seeded_keys_from_oid(&head)?.public_key().to_string();

        println!(
            "matrix case event built: kind={:?} id={} pow={} tags={:?}",
            event.kind,
            event.id,
            pow_event_flag,
            event.tags
        );

        assert_eq!(event.kind, nostr_sdk::Kind::TextNote);
        assert_eq!(event.content, note.message);
        assert_eq!(
            event
                .tags
                .iter()
                .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e"))
                .expect("e tag")
                .as_slice()[1],
            expected_root
        );
        if pow_event_flag {
            assert!(event.tags.iter().any(|tag| {
                tag.as_slice().first().map(|s| s.as_str()) == Some("nonce")
            }));
        } else {
            assert!(!event.tags.iter().any(|tag| {
                tag.as_slice().first().map(|s| s.as_str()) == Some("nonce")
            }));
        }
    }

    Ok(())
}
