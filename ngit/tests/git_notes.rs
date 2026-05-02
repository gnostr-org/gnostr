use std::sync::Arc;

use anyhow::Result;
use gnostr_asyncgit::sync::{add_note, default_notes_ref, list_notes, remove_note, show_note, RepoPath};
use gnostr_ngit::git_events::{generate_git_note_event, git_note_event_id, git_note_tags};
use nostr_sdk::{Keys, NostrSigner};
use serial_test::serial;
use test_utils::git::GitTestRepo;

#[tokio::test]
#[serial]
async fn real_repo_git_notes_workflow_creates_signed_event() -> Result<()> {
    let repo = GitTestRepo::new("main")?;
    repo.populate()?;

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

    let note = show_note(repo_path, head, Some(notes_ref.as_str()))?.expect("note exists");
    println!("notes show: {note:#?}");

    let notes = list_notes(repo_path, Some(notes_ref.as_str()))?;
    println!("notes list: {notes:#?}");

    let signer: Arc<dyn NostrSigner> = Arc::new(Keys::generate());
    let event = generate_git_note_event(&note, &signer).await?;
    println!("git note event: {event:#?}");

    assert_eq!(event.kind, nostr_sdk::Kind::TextNote);
    assert_eq!(event.content, note_text);
    assert_eq!(
        event
            .tags
            .iter()
            .find(|tag| tag.as_slice().first().map(|s| s.as_str()) == Some("e"))
            .expect("e tag")
            .as_slice()[1],
        format!("{:0>64}", head)
    );
    assert_eq!(
        git_note_event_id(&head.to_string())?.to_hex(),
        format!("{:0>64}", head)
    );
    assert_eq!(git_note_tags(&note)?.len(), 3);

    remove_note(repo_path, head, Some(notes_ref.as_str()))?;
    println!("notes removed: annotated_id={head}");
    assert!(show_note(repo_path, head, Some(notes_ref.as_str()))?.is_none());

    Ok(())
}
