use git2::{ErrorCode, Oid, Signature};
use scopetime::scope_time;

use super::{repository::repo, RepoPath};
use crate::error::Result;

/// A note attached to a git object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteInfo {
    pub note_id: Oid,
    pub annotated_id: Oid,
    pub message: String,
    pub author: String,
    pub committer: String,
}

fn signature_allow_undefined_name(
    repo: &git2::Repository,
) -> std::result::Result<Signature<'_>, git2::Error> {
    let signature = repo.signature();

    if let Err(ref e) = signature {
        if e.code() == ErrorCode::NotFound {
            let config = repo.config()?;

            if let (Err(_), Ok(email_entry)) = (
                config.get_entry("user.name"),
                config.get_entry("user.email"),
            ) {
                if let Some(email) = email_entry.value() {
                    return Signature::now("unknown", email);
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

    Ok(NoteInfo {
        note_id,
        annotated_id,
        message: note.message().unwrap_or_default().to_string(),
        author: note.author().name().unwrap_or_default().to_string(),
        committer: note.committer().name().unwrap_or_default().to_string(),
    })
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

    match repo.find_note(notes_ref, object_id) {
        Ok(note) => Ok(Some(NoteInfo {
            note_id: note.id(),
            annotated_id: object_id,
            message: note.message().unwrap_or_default().to_string(),
            author: note.author().name().unwrap_or_default().to_string(),
            committer: note.committer().name().unwrap_or_default().to_string(),
        })),
        Err(err) if err.code() == ErrorCode::NotFound => Ok(None),
        Err(err) => Err(err.into()),
    }
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

    repo.note_delete(
        object_id.into(),
        notes_ref,
        &signature,
        &signature,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::tests::repo_init;

    #[test]
    fn notes_roundtrip() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();
        let head = repo.head()?.target().unwrap();

        let notes_ref = default_notes_ref(repo_path)?;
        assert_eq!(notes_ref, "refs/notes/commits");

        add_note(repo_path, head, "hello notes", None, false)?;

        let note = show_note(repo_path, head, None)?.expect("note exists");
        assert_eq!(note.message, "hello notes");
        assert_eq!(note.annotated_id, head);

        let notes = list_notes(repo_path, None)?;
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].message, "hello notes");

        remove_note(repo_path, head, None)?;
        assert!(show_note(repo_path, head, None)?.is_none());
        assert!(list_notes(repo_path, None)?.is_empty());

        Ok(())
    }

    #[test]
    fn custom_notes_ref_roundtrip() -> Result<()> {
        let (_td, repo) = repo_init()?;
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();
        let head = repo.head()?.target().unwrap();

        add_note(
            repo_path,
            head,
            "hello custom notes",
            Some("refs/notes/reviews"),
            false,
        )?;

        let note = show_note(repo_path, head, Some("refs/notes/reviews"))?.expect("note exists");
        assert_eq!(note.message, "hello custom notes");
        assert!(list_notes(repo_path, None)?.is_empty());
        assert_eq!(list_notes(repo_path, Some("refs/notes/reviews"))?.len(), 1);

        Ok(())
    }
}
