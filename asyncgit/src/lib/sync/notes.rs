use git2::{ErrorCode, Oid, Signature};
use hex::decode as hex_decode;
use scopetime::scope_time;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use uuid::Uuid;

use super::{repository::repo, RepoPath};
use crate::error::Result;
use crate::types::get_leading_zero_bits;

// This module owns git note storage. NIP-34 shaping lives in `types::nip34`.
//
// Here, "git note" means a real git notes object attached to a commit, not a
// generic Nostr text note. The public NIP-34 event builders re-exported below
// preserve that distinction by emitting `GitPatch` events.
pub use crate::types::nip34::{
    generate_git_note_event, generate_git_note_event_with_pow, git_note_event_id, git_note_tags,
    GitNote,
};

/// A note attached to a git object.
///
/// This is the repository's git-backed note record, which later becomes a
/// Nostr `GitPatch` event through `types::nip34`.
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

impl Serialize for NoteInfo {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct NoteInfoSerde<'a> {
            note_id: String,
            annotated_id: String,
            notes_ref: Option<&'a str>,
            message: &'a str,
            author: &'a str,
            committer: &'a str,
            committer_time: i64,
        }

        NoteInfoSerde {
            note_id: self.note_id.to_string(),
            annotated_id: self.annotated_id.to_string(),
            notes_ref: self.notes_ref.as_deref(),
            message: &self.message,
            author: &self.author,
            committer: &self.committer,
            committer_time: self.committer_time,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NoteInfo {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NoteInfoSerde {
            note_id: String,
            annotated_id: String,
            notes_ref: Option<String>,
            message: String,
            author: String,
            committer: String,
            committer_time: i64,
        }

        let note = NoteInfoSerde::deserialize(deserializer)?;
        Ok(Self {
            note_id: Oid::from_str(&note.note_id)
                .map_err(|error| DeError::custom(format!("invalid note_id: {error}")))?,
            annotated_id: Oid::from_str(&note.annotated_id)
                .map_err(|error| DeError::custom(format!("invalid annotated_id: {error}")))?,
            notes_ref: note.notes_ref,
            message: note.message,
            author: note.author,
            committer: note.committer,
            committer_time: note.committer_time,
        })
    }
}

impl From<NoteInfo> for GitNote {
    fn from(note: NoteInfo) -> Self {
        Self {
            note_id: note.note_id,
            annotated_id: note.annotated_id,
            notes_ref: note.notes_ref,
            message: note.message,
            author: note.author,
            committer: note.committer,
            committer_time: note.committer_time,
        }
    }
}

impl From<&NoteInfo> for GitNote {
    fn from(note: &NoteInfo) -> Self {
        Self::from(note.clone())
    }
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
    /// Mine a note by appending a nonce until the PoW target is met.
    Mine {
        object_id: Oid,
        note: String,
        notes_ref: Option<String>,
        pow_target_bits: u8,
        prefix: Option<String>,
    },
    /// Amend a note by appending content and re-adding it.
    Amend {
        object_id: Oid,
        note: String,
        notes_ref: Option<String>,
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

pub const DEFAULT_MINE_NOTE_PREFIX: &str = "000";
pub const DEFAULT_MINE_NOTE_TARGET_BITS: u8 = 0;

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

fn note_id_leading_zero_bits(note_id: &Oid) -> u8 {
    let bytes = hex_decode(note_id.to_string()).expect("git oid must be valid hex");
    get_leading_zero_bits(&bytes)
}

fn append_nonce(message: &str, nonce: u64, prefix: &str) -> String {
    if message.is_empty() {
        format!("{prefix}-{nonce:08x}")
    } else {
        format!("{message}\n\n{prefix}-{nonce:08x}")
    }
}

fn combine_note_messages(existing: Option<&str>, note: &str) -> String {
    match existing.map(str::trim_end) {
        Some(existing) if existing.is_empty() => note.to_string(),
        Some(existing) if note.is_empty() => existing.to_string(),
        Some(existing) => format!("{existing}\n{note}"),
        None => note.to_string(),
    }
}

/// Append a plain-text public attestation entry to a note log.
pub fn append_public_attestation_log(
    existing: Option<&str>,
    timestamp: i64,
    event_id: &str,
    commit_id: &str,
    pow_bits: u8,
) -> String {
    let entry = format!(
        "attestation timestamp={timestamp} event_id={event_id} commit_id={commit_id} pow={pow_bits}"
    );

    match existing.map(str::trim_end) {
        Some(existing) if existing.is_empty() => entry,
        Some(existing) => format!("{existing}\n{entry}"),
        None => entry,
    }
}

/// Amends the note for an object by appending new content and re-adding it.
///
/// If a note already exists, the existing content is preserved and the new
/// content is appended on a new line before the updated note is written back.
pub fn amend_note<T: Into<Oid>>(
    repo_path: &RepoPath,
    object_id: T,
    note: &str,
    notes_ref: Option<&str>,
) -> Result<Oid> {
    scope_time!("amend_note");

    let repo = repo(repo_path)?;
    let object_id = object_id.into();
    let signature = signature_allow_undefined_name(&repo)?;
    let existing_note = match repo.find_note(notes_ref, object_id) {
        Ok(existing_note) => Some(existing_note),
        Err(err) if err.code() == ErrorCode::NotFound => None,
        Err(err) => return Err(err.into()),
    };
    let has_existing_note = existing_note.is_some();
    let combined_note = if let Some(existing_note) = existing_note {
        let existing_message = existing_note.message().unwrap_or_default();
        if existing_message.is_empty() {
            note.to_string()
        } else if note.is_empty() {
            existing_message.to_string()
        } else {
            format!("{existing_message}\n{note}")
        }
    } else {
        note.to_string()
    };

    if has_existing_note {
        repo.note_delete(object_id, notes_ref, &signature, &signature)?;
    }

    Ok(repo.note(
        &signature,
        &signature,
        notes_ref,
        object_id,
        &combined_note,
        false,
    )?)
}

/// Mine a note by appending a nonce until the requested PoW target is met.
///
/// If a note already exists, its content is preserved and the new content is
/// appended before mining. The existing note is only replaced after a valid
/// candidate has been found.
pub fn mine_note<T: Into<Oid>>(
    repo_path: &RepoPath,
    object_id: T,
    note: &str,
    notes_ref: Option<&str>,
    pow_target_bits: u8,
    prefix: Option<&str>,
) -> Result<Oid> {
    scope_time!("mine_note");

    let repo = repo(repo_path)?;
    let object_id = object_id.into();
    let signature = signature_allow_undefined_name(&repo)?;
    let prefix = prefix.unwrap_or(DEFAULT_MINE_NOTE_PREFIX);
    let existing_note = match repo.find_note(notes_ref, object_id) {
        Ok(existing_note) => Some(existing_note),
        Err(err) if err.code() == ErrorCode::NotFound => None,
        Err(err) => return Err(err.into()),
    };
    let base_message = combine_note_messages(existing_note.as_ref().and_then(|n| n.message()), note);
    let temp_notes_ref = format!(
        "refs/notes/gnostr-mine/{}-{}",
        object_id,
        Uuid::new_v4().simple()
    );

    let mut nonce = 0u64;
    let candidate_note_id = loop {
        let candidate_message = append_nonce(&base_message, nonce, prefix);
        let note_id = repo.note(
            &signature,
            &signature,
            Some(temp_notes_ref.as_str()),
            object_id,
            &candidate_message,
            true,
        )?;
        if note_id.to_string().starts_with(prefix)
            && note_id_leading_zero_bits(&note_id) >= pow_target_bits
        {
            break note_id;
        }
        nonce = nonce.wrapping_add(1);
    };

    let final_message = append_nonce(&base_message, nonce, prefix);
    let force = existing_note.is_some();
    let final_note_id = repo.note(
        &signature,
        &signature,
        notes_ref,
        object_id,
        &final_message,
        force,
    )
    .map_err(|err| {
        let _ = repo.note_delete(
            object_id,
            Some(temp_notes_ref.as_str()),
            &signature,
            &signature,
        );
        err
    })?;

    let _ = repo.note_delete(
        object_id,
        Some(temp_notes_ref.as_str()),
        &signature,
        &signature,
    );

    Ok(if final_note_id == candidate_note_id {
        final_note_id
    } else {
        candidate_note_id
    })
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
        NotesCommand::Mine {
            object_id,
            note,
            notes_ref,
            pow_target_bits,
            prefix,
        } => Ok(NotesCommandResult::NoteId(mine_note(
            repo_path,
            object_id,
            &note,
            notes_ref.as_deref(),
            pow_target_bits,
            prefix.as_deref(),
        )?)),
        NotesCommand::Amend {
            object_id,
            note,
            notes_ref,
        } => Ok(NotesCommandResult::NoteId(amend_note(
            repo_path,
            object_id,
            &note,
            notes_ref.as_deref(),
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

    use serial_test::serial;
    use time::OffsetDateTime;

    use crate::{
        profiles::{bitcoindev_1, bitcoindev_2, bitcoindev_3},
        sync::{
            commit::{self, mine_commit, padded_commit_id, CommitMineOptions},
            stage_add_file,
            tests::repo_init_empty,
        },
        types::{
            generate_git_note_event, generate_git_note_event_with_pow, get_leading_zero_bits, Id,
            Keys,
            nip3::create_attestation_with_pow, EventKind, PrivateKey, Unixtime,
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

    #[test]
    fn mine_note_appends_existing_content_and_keeps_pow_target() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let head = repo.head()?.target().unwrap();

        let first_note_id = add_note(repo_path, head, "hello notes", None, false)?;
        let mined_note_id = mine_note(repo_path, head, "more notes", None, 0, Some("0"))?;
        let remined_note_id = mine_note(repo_path, head, "even more notes", None, 0, Some("00"))?;
        let triple_mined_note_id =
            mine_note(repo_path, head, "yet even more notes", None, 0, Some("000"))?;
        let note = show_note(repo_path, head, None)?.expect("note exists");

        println!(
            "mined notes: first_note_id={first_note_id} mined_note_id={mined_note_id} remined_note_id={remined_note_id} triple_mined_note_id={triple_mined_note_id} note={note:#?}"
        );
        assert!(note.message.contains("hello notes"));
        assert!(note.message.contains("more notes"));
        assert!(note.message.contains("even more notes"));
        assert!(note.message.contains("yet even more notes"));
        assert!(note.message.contains("000-"));
        assert_eq!(note.note_id, triple_mined_note_id);
        assert!(note.note_id.to_string().starts_with("000"));
        Ok(())
    }

    #[test]
    fn amend_note_appends_existing_content() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let head = repo.head()?.target().unwrap();

        let first_note_id = add_note(repo_path, head, "hello notes", None, false)?;
        let amended_note_id = amend_note(repo_path, head, "more notes", None)?;
        let note = show_note(repo_path, head, None)?.expect("note exists");

        println!("amended notes: first_note_id={first_note_id} amended_note_id={amended_note_id} note={note:#?}");
        assert_eq!(note.message, "hello notes\nmore notes");
        assert_eq!(note.note_id, amended_note_id);
        Ok(())
    }

    #[test]
    fn public_attestation_log_appends_plain_text_entries() {
        let fixtures = [bitcoindev_1, bitcoindev_2, bitcoindev_3];
        let mut logged = String::from("hello notes");

        for (index, profile) in fixtures.iter().enumerate() {
            let ts = 1234 + index as i64;
            let event_id = format!("event-{index}");
            let commit_id = format!("commit-{index}");
            logged = append_public_attestation_log(
                Some(&logged),
                ts,
                &event_id,
                &commit_id,
                7 + index as u8,
            );
            println!(
                "pretty_print_attestations\n{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "profile": profile.label,
                    "npub": profile.npub(),
                    "nsec": profile.nsec(),
                    "metadata": profile.metadata(),
                    "log": logged,
                }))
                .unwrap()
            );
        }

        assert_eq!(
            logged,
            "hello notes\nattestation timestamp=1234 event_id=event-0 commit_id=commit-0 pow=7\nattestation timestamp=1235 event_id=event-1 commit_id=commit-1 pow=8\nattestation timestamp=1236 event_id=event-2 commit_id=commit-2 pow=9"
        );
    }

    #[test]
    fn pretty_print_attestations() -> Result<()> {
        let (_td, repo) = repo_init_empty()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let fixtures = [bitcoindev_1, bitcoindev_2, bitcoindev_3];
        let mut previous_attestation_id: Option<String> = None;

        for (index, profile) in fixtures.iter().enumerate() {
            let file_name = format!("pretty-print-attestations-{index}.txt");
            File::create(root.join(&file_name))?.write_all(profile.label.as_bytes())?;
            stage_add_file(repo_path, Path::new(&file_name))?;

            let commit_id = mine_commit(
                repo_path,
                CommitMineOptions {
                    threads: 1,
                    target: "0".to_string(),
                    message: vec![format!("{} commit", profile.label)],
                    timestamp: OffsetDateTime::from_unix_timestamp(0).unwrap(),
                },
            )?;

            let attestation_target = Id::try_from_hex_string(&padded_commit_id(commit_id.to_string()))
                .map_err(|err| crate::error::Error::Generic(err.to_string()))?;
            let secret_key = profile.private_key().0.clone();
            let (xonly_public_key, _parity) = secret_key.x_only_public_key(secp256k1::SECP256K1);
            let attestation = create_attestation_with_pow(
                attestation_target,
                profile.metadata_json(),
                &xonly_public_key,
                &secret_key,
                5,
            );
            let notes_ref = previous_attestation_id
                .as_deref()
                .map(|event_id| format!("refs/notes/public-attestations/{event_id}"))
                .unwrap_or_else(|| "refs/notes/public-attestations/root".to_string());

            let note_message = append_public_attestation_log(
                None,
                1234 + index as i64,
                &attestation.id.as_hex_string(),
                &commit_id.to_string(),
                attestation.nonce_data().map(|(_, bits)| bits).unwrap_or(0),
            );
            let note_id = add_note(repo_path, commit_id, &note_message, Some(&notes_ref), true)?;
            let note = show_note(repo_path, commit_id, Some(&notes_ref))?.expect("note exists");

            println!(
                "pretty_print_attestations\n{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "profile": profile.label,
                    "commit": commit_id.to_string(),
                    "note_id": note_id.to_string(),
                    "note": &note,
                    "author": note.author,
                    "committer": note.committer,
                    "committer_time": note.committer_time,
                    "profile_metadata": profile.metadata(),
                    "profile_npub": profile.npub(),
                    "profile_nsec": profile.nsec(),
                    "attestation_id": attestation.id.to_string(),
                    "attestation_signature": format!("{:?}", attestation.sig),
                    "attestation_nonce": attestation.nonce_data().map(|(nonce, bits)| serde_json::json!({"nonce": nonce, "bits": bits})),
                    "attestation_kind": format!("{:?}", attestation.kind),
                    "attestation_tags": attestation.tags,
                    "attestation_content": attestation.content,
                }))
                .unwrap()
            );

            assert_eq!(note.note_id, note_id);
            assert!(note.message.contains(&attestation.id.as_hex_string()));
            let commit_id_string = commit_id.to_string();
            assert!(note.message.contains(&commit_id_string));
            previous_attestation_id = Some(attestation.id.as_hex_string());
        }

        Ok(())
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn git_note_event_matrix_covers_commit_and_pow_variants() -> Result<()> {
        println!("[asyncgit] git_note_event_matrix_covers_commit_and_pow_variants");
        let private_key = PrivateKey::generate();

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

        for (label, mine_the_commit, mine_the_note, pow_the_event) in cases {
            println!(
                "matrix case start: label={label} mine_commit={mine_the_commit} mine_note={mine_the_note} pow_event={pow_the_event}"
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

            let note_base_message = format!("{label} note");
            let note = if mine_the_note {
                let mut nonce = 0u32;
                loop {
                    let candidate_message = format!("{note_base_message} #{nonce}");
                    let note_id = add_note(repo_path, commit_id, &candidate_message, None, true)?;
                    let note = show_note(repo_path, commit_id, None)?.expect("note exists");
                    println!(
                        "matrix case note attempt: label={label} nonce={nonce} note_id={} annotated_id={} message={}",
                        note_id, note.annotated_id, note.message
                    );
                    if note.note_id.to_string().starts_with('0') {
                        assert_eq!(note.note_id, note_id);
                        break note;
                    }
                    nonce = nonce.wrapping_add(1);
                }
            } else {
                let note_id = add_note(repo_path, commit_id, &note_base_message, None, false)?;
                let note = show_note(repo_path, commit_id, None)?.expect("note exists");
                println!(
                    "matrix case note created: note_id={note_id} annotated_id={} message={}",
                    note.annotated_id, note.message
                );
                note
            };

            assert_eq!(note.annotated_id, commit_id.into());
            assert!(note.message.starts_with(&note_base_message));

            let git_note = GitNote::from(&note);
            let event = if pow_the_event {
                generate_git_note_event_with_pow(&git_note, &private_key, 4)
                    .map_err(|err| crate::error::Error::Generic(err.to_string()))?
            } else {
                generate_git_note_event(&git_note, &private_key)
                    .map_err(|err| crate::error::Error::Generic(err.to_string()))?
            };
            println!(
                "matrix case nip34 note event built: kind={:?} id={} pow={} nonce={:?}",
                event.kind,
                event.id,
                pow_the_event,
                event.nonce_data()
            );
            println!(
                "matrix case nip34 note event json: {}",
                serde_json::to_string_pretty(&event).expect("serialize matrix event")
            );
            println!(
                "matrix case nip34 note event e tag: {:?}",
                event
                    .tags
                    .iter()
                    .find(|tag| tag.tagname() == "e")
                    .map(|tag| &tag.0)
            );
            println!(
                "matrix case nip34 note event commit tag: {:?}",
                event
                    .tags
                    .iter()
                    .find(|tag| tag.tagname() == "commit")
                    .map(|tag| &tag.0)
            );
            assert_eq!(event.kind, EventKind::Patches);
            assert_eq!(event.content, note.message);
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
