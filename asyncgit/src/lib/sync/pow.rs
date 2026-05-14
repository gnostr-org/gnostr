use git2::{Oid, Sort};

use super::{notes::show_note, repository::repo, RepoPath};
use crate::{error::Result, types::get_leading_zero_bits};

/// Per-commit proof-of-work contribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccumulatedPowEntry {
    pub commit_id: Oid,
    pub commit_pow: u32,
    pub note_id: Option<Oid>,
    pub note_pow: u32,
    pub total_pow: u32,
}

/// Proof-of-work summary for a commit range.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccumulatedPowSummary {
    pub entries: Vec<AccumulatedPowEntry>,
    pub commit_pow: u32,
    pub note_pow: u32,
    pub total_pow: u32,
}

fn pow_bits_from_oid(oid: Oid) -> u32 {
    get_leading_zero_bits(oid.as_bytes()) as u32
}

fn accumulated_pow_entry(
    repo_path: &RepoPath,
    commit_id: Oid,
    notes_ref: Option<&str>,
    include_commit: bool,
    include_notes: bool,
) -> Result<AccumulatedPowEntry> {
    let commit_pow = if include_commit { pow_bits_from_oid(commit_id) } else { 0 };
    let note = if include_notes {
        show_note(repo_path, commit_id, notes_ref)?
    } else {
        None
    };
    let (note_id, note_pow) = if let Some(note) = note {
        let note_id = note.note_id;
        (Some(note_id), pow_bits_from_oid(note_id))
    } else {
        (None, 0)
    };

    Ok(AccumulatedPowEntry {
        commit_id,
        commit_pow,
        note_id,
        note_pow,
        total_pow: commit_pow + note_pow,
    })
}

fn accumulated_pow_range(
    repo_path: &RepoPath,
    range: &str,
    notes_ref: Option<&str>,
    include_notes: bool,
) -> Result<AccumulatedPowSummary> {
    let repo = repo(repo_path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.push_range(range)?;
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;

    let mut summary = AccumulatedPowSummary::default();

    for oid in revwalk {
        let commit_id = oid?;
        let mut entry = accumulated_pow_entry(repo_path, commit_id, notes_ref, true, include_notes)?;
        summary.commit_pow += entry.commit_pow;
        summary.note_pow += entry.note_pow;
        summary.total_pow += entry.total_pow;
        summary.entries.push(entry);
    }

    Ok(summary)
}

/// Sum commit and git-note proof-of-work over a revwalk range.
///
/// Note POW is included by default; use the narrower helpers below if you need
/// commit-only or note-only totals.
pub fn accumulated_pow(
    repo_path: &RepoPath,
    range: &str,
    notes_ref: Option<&str>,
) -> Result<AccumulatedPowSummary> {
    accumulated_pow_range(repo_path, range, notes_ref, true)
}

/// Sum commit proof-of-work over a revwalk range.
pub fn accumulated_commit_pow(repo_path: &RepoPath, range: &str) -> Result<AccumulatedPowSummary> {
    accumulated_pow_range(repo_path, range, None, false)
}

/// Sum git-note proof-of-work over a revwalk range.
pub fn accumulated_note_pow(
    repo_path: &RepoPath,
    range: &str,
    notes_ref: Option<&str>,
) -> Result<AccumulatedPowSummary> {
    accumulated_pow_range(repo_path, range, notes_ref, false)
}

/// Convenience helper for linear history depth from `HEAD`.
pub fn accumulated_pow_depth(
    repo_path: &RepoPath,
    depth: usize,
    notes_ref: Option<&str>,
) -> Result<AccumulatedPowSummary> {
    accumulated_pow(repo_path, &format!("HEAD...HEAD~{depth}"), notes_ref)
}

#[cfg(test)]
mod tests {
    use git2::Oid;

    use crate::{
        sync::{
            commit::commit,
            notes::{add_note, mine_note},
            stage_add_file,
            tests::repo_init_empty,
            RepoPath,
        },
        types::get_leading_zero_bits,
    };

    use super::*;

    fn oid_pow(oid: Oid) -> u32 {
        get_leading_zero_bits(oid.as_bytes()) as u32
    }

    #[test]
    fn accumulated_pow_sums_commit_and_note_pow() -> Result<()> {
        let (_td, repo) = repo_init_empty()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;

        std::fs::write(root.join("one.txt"), b"one")?;
        stage_add_file(repo_path, std::path::Path::new("one.txt"))?;
        let c1 = commit(repo_path, "c1")?;

        std::fs::write(root.join("two.txt"), b"two")?;
        stage_add_file(repo_path, std::path::Path::new("two.txt"))?;
        let c2 = commit(repo_path, "c2")?;

        std::fs::write(root.join("three.txt"), b"three")?;
        stage_add_file(repo_path, std::path::Path::new("three.txt"))?;
        let c3 = commit(repo_path, "c3")?;

        let note_id = mine_note(repo_path, c2, "note", None, 0, Some("0"))?;
        let summary = accumulated_pow(repo_path, "HEAD...HEAD~2", None)?;
        let commit_only = accumulated_commit_pow(repo_path, "HEAD...HEAD~2")?;
        let note_only = accumulated_note_pow(repo_path, "HEAD...HEAD~2", None)?;

        assert_eq!(summary.total_pow, summary.commit_pow + summary.note_pow);
        assert_eq!(commit_only.total_pow, commit_only.commit_pow);
        assert_eq!(note_only.note_pow, summary.note_pow);
        assert_eq!(summary.commit_pow, oid_pow(c3.into()) + oid_pow(c2.into()));
        assert!(summary.note_pow >= oid_pow(note_id));
        assert!(summary.entries.iter().any(|entry| entry.commit_id == c2.into()));
        assert!(summary.entries.iter().any(|entry| entry.commit_id == c3.into()));
        assert_eq!(summary.entries.len(), 2);

        let depth_summary = accumulated_pow_depth(repo_path, 2, None)?;
        assert_eq!(depth_summary.total_pow, summary.total_pow);
        assert_eq!(depth_summary.entries.len(), summary.entries.len());

        let _ = c1;
        Ok(())
    }

    #[test]
    fn accumulated_commit_pow_ignores_missing_notes() -> Result<()> {
        let (_td, repo) = repo_init_empty()?;
        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;

        std::fs::write(root.join("one.txt"), b"one")?;
        stage_add_file(repo_path, std::path::Path::new("one.txt"))?;
        let c1 = commit(repo_path, "c1")?;

        let summary = accumulated_commit_pow(repo_path, "HEAD")?;
        assert_eq!(summary.entries.len(), 1);
        assert_eq!(summary.entries[0].commit_id, c1.into());
        assert_eq!(summary.note_pow, 0);
        Ok(())
    }
}
