use git2::{ErrorCode, Oid, Signature};
use scopetime::scope_time;

use super::{repository::repo, RepoPath};
use crate::error::Result;

/// A note attached to a git object.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct NoteInfo {
    pub note_id: Oid,
    pub annotated_id: Oid,
    pub notes_ref: Option<String>,
    pub message: String,
    pub author: String,
    pub committer: String,
    pub committer_time: i64,
}

/// Commands supported by the notes backend.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotesCommand {
    /// Return the repository default notes ref.
    DefaultRef,
    /// Add a note to an object.
    Add {
        object_id: Oid,
        note: String,
        notes_ref: Option<String>,
        force: bool,
    },
    /// Show a note attached to an object.
    Show {
        object_id: Oid,
        notes_ref: Option<String>,
    },
    /// List notes under a ref.
    List { notes_ref: Option<String> },
    /// Remove a note attached to an object.
    Remove {
        object_id: Oid,
        notes_ref: Option<String>,
    },
}

/// Results returned by a notes backend command.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotesCommandResult {
    DefaultRef(String),
    NoteId(Oid),
    Note(Option<NoteInfo>),
    Notes(Vec<NoteInfo>),
    Removed,
}

fn signature_allow_undefined_name(
    repo: &git2::Repository,
) -> std::result::Result<Signature<'_>, git2::Error> {
    let signature = repo.signature();

    if let Err(ref e) = signature {
        if e.code() == ErrorCode::NotFound {
            let config = repo.config()?;

            if config.get_entry("user.name").is_err() {
                if let Ok(email_entry) = config.get_entry("user.email") {
                    if let Some(email) = email_entry.value() {
                        return Signature::now("unknown", email);
                    }
                }
            }
        }
    }

    signature
}

fn note_info(
    repo: &git2::Repository,
    notes_ref: Option<&str>,
    note_id: Oid,
    annotated_id: Oid,
) -> Result<NoteInfo> {
    let note = repo.find_note(notes_ref, annotated_id)?;
    let message = note.message().unwrap_or_default().to_string();
    let author = note.author().name().unwrap_or_default().to_string();
    let committer = note.committer().name().unwrap_or_default().to_string();
    let committer_time = note.committer().when().seconds();

    let info = NoteInfo {
        note_id,
        annotated_id,
        notes_ref: notes_ref.map(str::to_string),
        message,
        author,
        committer,
        committer_time,
    };

    Ok(info)
}

/// Returns the repository's default notes reference.
pub fn default_notes_ref(repo_path: &RepoPath) -> Result<String> {
    scope_time!("default_notes_ref");

    let repo = repo(repo_path)?;
    Ok(repo.note_default_ref()?)
}

/// Adds a note for an object.
pub fn add_note<T: Into<Oid>>(
    repo_path: &RepoPath,
    object_id: T,
    note: &str,
    notes_ref: Option<&str>,
    force: bool,
) -> Result<Oid> {
    scope_time!("add_note");

    let repo = repo(repo_path)?;
    let signature = signature_allow_undefined_name(&repo)?;

    Ok(repo.note(
        &signature,
        &signature,
        notes_ref,
        object_id.into(),
        note,
        force,
    )?)
}

/// Lists notes for the given notes reference.
pub fn list_notes(repo_path: &RepoPath, notes_ref: Option<&str>) -> Result<Vec<NoteInfo>> {
    scope_time!("list_notes");

    let repo = repo(repo_path)?;
    let notes = match repo.notes(notes_ref) {
        Ok(notes) => notes,
        Err(err) if err.code() == ErrorCode::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err.into()),
    };

    let mut result = Vec::new();
    for note in notes {
        let (note_id, annotated_id) = note?;
        result.push(note_info(&repo, notes_ref, note_id, annotated_id)?);
    }

    result.sort_by(|a, b| {
        a.committer_time
            .cmp(&b.committer_time)
            .then_with(|| a.note_id.to_string().cmp(&b.note_id.to_string()))
    });

    Ok(result)
}

/// Shows the note for a specific object.
pub fn show_note<T: Into<Oid>>(
    repo_path: &RepoPath,
    object_id: T,
    notes_ref: Option<&str>,
) -> Result<Option<NoteInfo>> {
    scope_time!("show_note");

    let repo = repo(repo_path)?;
    let object_id = object_id.into();

    let note = match repo.find_note(notes_ref, object_id) {
        Ok(note) => Some(NoteInfo {
            note_id: note.id(),
            annotated_id: object_id,
            notes_ref: notes_ref.map(str::to_string),
            message: note.message().unwrap_or_default().to_string(),
            author: note.author().name().unwrap_or_default().to_string(),
            committer: note.committer().name().unwrap_or_default().to_string(),
            committer_time: note.committer().when().seconds(),
        }),
        Err(err) if err.code() == ErrorCode::NotFound => None,
        Err(err) => return Err(err.into()),
    };

    Ok(note)
}

/// Removes the note for a specific object.
pub fn remove_note<T: Into<Oid>>(
    repo_path: &RepoPath,
    object_id: T,
    notes_ref: Option<&str>,
) -> Result<()> {
    scope_time!("remove_note");

    let repo = repo(repo_path)?;
    let signature = signature_allow_undefined_name(&repo)?;

    repo.note_delete(object_id.into(), notes_ref, &signature, &signature)?;

    Ok(())
}

/// Run a notes backend command through a single typed surface.
pub fn run_notes_command(
    repo_path: &RepoPath,
    command: NotesCommand,
) -> Result<NotesCommandResult> {
    match command {
        NotesCommand::DefaultRef => Ok(NotesCommandResult::DefaultRef(default_notes_ref(
            repo_path,
        )?)),
        NotesCommand::Add {
            object_id,
            note,
            notes_ref,
            force,
        } => Ok(NotesCommandResult::NoteId(add_note(
            repo_path,
            object_id,
            &note,
            notes_ref.as_deref(),
            force,
        )?)),
        NotesCommand::Show {
            object_id,
            notes_ref,
        } => Ok(NotesCommandResult::Note(show_note(
            repo_path,
            object_id,
            notes_ref.as_deref(),
        )?)),
        NotesCommand::List { notes_ref } => Ok(NotesCommandResult::Notes(list_notes(
            repo_path,
            notes_ref.as_deref(),
        )?)),
        NotesCommand::Remove {
            object_id,
            notes_ref,
        } => {
            remove_note(repo_path, object_id, notes_ref.as_deref())?;
            Ok(NotesCommandResult::Removed)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::Write,
        path::Path,
    };

    use nostr::Event as NostrEvent;
    use nostr_sdk::{Client, Keys};
    use time::OffsetDateTime;

    use crate::{
        sync::{
            commit::{self, mine_commit, CommitMineOptions},
            stage_add_file,
            tests::repo_init_empty,
        },
        types::{
            generate_git_note_event, generate_git_note_event_with_pow, get_leading_zero_bits,
            EventKind, PrivateKey, Unixtime,
        },
        types::nip13::NIP13Event,
    };

    use super::*;
    use crate::sync::tests::repo_init;

    #[test]
    fn notes_roundtrip() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let head = repo.head()?.target().unwrap();

        let notes_ref = default_notes_ref(repo_path)?;
        println!("notes default ref: {notes_ref}");
        assert_eq!(notes_ref, "refs/notes/commits");

        let note_id = add_note(repo_path, head, "hello notes", None, false)?;
        println!("notes created: note_id={note_id} annotated_id={head} message=hello notes");

        let note = show_note(repo_path, head, None)?.expect("note exists");
        println!("notes show: {note:#?}");
        assert_eq!(note.message, "hello notes");
        assert_eq!(note.annotated_id, head);

        let notes = list_notes(repo_path, None)?;
        println!("notes list: {notes:#?}");
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].message, "hello notes");

        remove_note(repo_path, head, None)?;
        println!("notes removed: annotated_id={head}");
        assert!(show_note(repo_path, head, None)?.is_none());
        assert!(list_notes(repo_path, None)?.is_empty());

        Ok(())
    }

    #[test]
    fn custom_notes_ref_roundtrip() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let head = repo.head()?.target().unwrap();

        let note_id = add_note(
            repo_path,
            head,
            "hello custom notes",
            Some("refs/notes/reviews"),
            false,
        )?;
        println!(
            "custom notes created: note_id={note_id} annotated_id={head} notes_ref=refs/notes/reviews message=hello custom notes"
        );

        let note = show_note(repo_path, head, Some("refs/notes/reviews"))?.expect("note exists");
        println!("custom notes show: {note:#?}");
        assert_eq!(note.message, "hello custom notes");
        let default_notes = list_notes(repo_path, None)?;
        println!("custom default notes list: {default_notes:#?}");
        assert!(default_notes.is_empty());
        let review_notes = list_notes(repo_path, Some("refs/notes/reviews"))?;
        println!("custom notes list: {review_notes:#?}");
        assert_eq!(review_notes.len(), 1);

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn git_note_event_matrix_covers_commit_and_pow_variants() -> Result<()> {
        let private_key = PrivateKey::generate();
        let client_keys = Keys::generate();
        let relay_urls = vec![
            "wss://relay.damus.io".to_string(),
            "wss://nos.lol".to_string(),
        ];
        let mut client = Client::new(&client_keys);
        for relay_url in &relay_urls {
            client
                .add_relay(relay_url)
                .await
                .map_err(|err| crate::error::Error::Generic(err.to_string()))?;
        }
        client.connect().await;

        let cases = [
            ("plain-commit/plain-event", false, false),
            ("plain-commit/pow-event", false, true),
            ("mined-commit/plain-event", true, false),
            ("mined-commit/pow-event", true, true),
        ];

        for (label, mine_the_commit, pow_the_event) in cases {
            println!(
                "matrix case start: label={label} mine_commit={mine_the_commit} pow_event={pow_the_event}"
            );
            let (_td, repo) = repo_init_empty()?;
            let root = repo.path().parent().unwrap();
            let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
            let repo_path: &RepoPath = &repo_path_owned;
            let file_path = Path::new("matrix.txt");
            File::create(root.join(file_path))?.write_all(label.as_bytes())?;
            println!("matrix case file written: path={}", file_path.display());
            stage_add_file(repo_path, file_path)?;
            println!("matrix case staged: label={label}");

            let commit_id = if mine_the_commit {
                let mined = mine_commit(
                    repo_path,
                    CommitMineOptions {
                        threads: 1,
                        target: "0".to_string(),
                        message: vec![format!("{label} commit")],
                        timestamp: OffsetDateTime::from_unix_timestamp(0).unwrap(),
                    },
                )?;
                println!("matrix case mined commit: {mined}");
                mined
            } else {
                let committed = commit::commit(repo_path, &format!("{label} commit"))?;
                println!("matrix case committed: {committed}");
                committed
            };

            let note_message = format!("{label} note");
            let note_id = add_note(repo_path, commit_id, &note_message, None, false)?;
            let note = show_note(repo_path, commit_id, None)?.expect("note exists");
            println!(
                "matrix case note created: note_id={note_id} annotated_id={} message={}",
                note.annotated_id, note.message
            );

            assert_eq!(note.note_id, note_id);
            assert_eq!(note.annotated_id, commit_id.into());
            assert_eq!(note.message, note_message);

            let event = if pow_the_event {
                generate_git_note_event_with_pow(&note, &private_key, 4)
                    .map_err(|err| crate::error::Error::Generic(err.to_string()))?
            } else {
                generate_git_note_event(&note, &private_key)
                    .map_err(|err| crate::error::Error::Generic(err.to_string()))?
            };
            println!(
                "matrix case event built: kind={:?} id={} pow={} nonce={:?}",
                event.kind,
                event.id,
                pow_the_event,
                event.nonce_data()
            );
            println!(
                "matrix case event json: {}",
                serde_json::to_string_pretty(&event).expect("serialize matrix event")
            );
            let live_event = NostrEvent::from_json(
                &serde_json::to_string(&event).expect("serialize matrix event"),
            )
            .map_err(|err| crate::error::Error::Generic(err.to_string()))?;
            let event_output = client
                .send_event(&live_event)
                .await
                .map_err(|err| crate::error::Error::Generic(err.to_string()))?;
            println!(
                "matrix case event published: label={label} event_id={} successes={:?} failures={:?}",
                event_output.val,
                event_output.success,
                event_output.failed
            );
            assert!(
                !event_output.success.is_empty(),
                "matrix case {label} was not accepted by any relay"
            );

            assert_eq!(event.kind, EventKind::TextNote);
            assert_eq!(event.content, note_message);
            assert_eq!(event.created_at, Unixtime(note.committer_time));
            assert!(event.tags.iter().any(|tag| tag.tagname() == "e" && tag.marker() == "root"));
            assert!(event.tags.iter().any(|tag| {
                tag.tagname() == "commit" && tag.value() == commit_id.to_string()
            }));

            if pow_the_event {
                assert!(event.tags.iter().any(|tag| tag.tagname() == "nonce"));
                assert!(event.nonce_data().is_some());
                assert!(get_leading_zero_bits(&event.id.0) >= 4);
            } else {
                assert!(event.nonce_data().is_none());
                assert!(!event.tags.iter().any(|tag| tag.tagname() == "nonce"));
            }

            println!("matrix case done: label={label}");
        }

        Ok(())
    }

    #[test]
    fn notes_command_roundtrip() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let head = repo.head()?.target().unwrap();

        assert_eq!(
            run_notes_command(repo_path, NotesCommand::DefaultRef)?,
            NotesCommandResult::DefaultRef("refs/notes/commits".to_string())
        );

        assert!(matches!(
            run_notes_command(
                repo_path,
                NotesCommand::Show {
                    object_id: head,
                    notes_ref: None,
                }
            )?,
            NotesCommandResult::Note(None)
        ));

        assert!(matches!(
            run_notes_command(
                repo_path,
                NotesCommand::Add {
                    object_id: head,
                    note: "hello command notes".to_string(),
                    notes_ref: None,
                    force: false,
                }
            )?,
            NotesCommandResult::NoteId(_)
        ));

        println!("notes command add created note for head={head}");

        assert!(matches!(
            run_notes_command(
                repo_path,
                NotesCommand::List { notes_ref: None }
            )?,
            NotesCommandResult::Notes(notes) if notes.len() == 1
        ));

        println!("notes command list returned 1 note for head={head}");

        assert!(matches!(
            run_notes_command(
                repo_path,
                NotesCommand::Remove {
                    object_id: head,
                    notes_ref: None,
                }
            )?,
            NotesCommandResult::Removed
        ));

        println!("notes command remove cleared note for head={head}");

        Ok(())
    }
}
