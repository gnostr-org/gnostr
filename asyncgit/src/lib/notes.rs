use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossbeam_channel::Sender;

use crate::{
    asyncjob::{AsyncJob, AsyncSingleJob, RunParams},
    error::Result,
    hash,
    sync::{self, NoteInfo, RepoPath},
    AsyncGitNotification,
};

///
#[derive(Default, Clone)]
pub struct NotesResult {
    hash: u64,
    notes: Vec<NoteInfo>,
    notes_ref: Option<String>,
}

///
pub struct AsyncNotes {
    last: Option<(Instant, NotesResult)>,
    sender: Sender<AsyncGitNotification>,
    job: AsyncSingleJob<AsyncNotesJob>,
    repo: RepoPath,
}

impl AsyncNotes {
    ///
    pub fn new(repo: RepoPath, sender: &Sender<AsyncGitNotification>) -> Self {
        Self {
            repo,
            last: None,
            sender: sender.clone(),
            job: AsyncSingleJob::new(sender.clone()),
        }
    }

    ///
    pub fn last(&self) -> Result<Option<Vec<NoteInfo>>> {
        Ok(self.last.as_ref().map(|result| result.1.notes.clone()))
    }

    ///
    pub fn is_pending(&self) -> bool {
        self.job.is_pending()
    }

    fn is_outdated(&self, dur: Duration, notes_ref: Option<&str>) -> bool {
        self.last.as_ref().is_none_or(|(last_time, result)| {
            last_time.elapsed() > dur || result.notes_ref.as_deref() != notes_ref
        })
    }

    ///
    pub fn request(&mut self, dur: Duration, force: bool, notes_ref: Option<&str>) -> Result<()> {
        log::trace!("request");

        // Spawn the note fetch on the async job queue; callers apply the result
        // later from their update/update_async loop when Notes arrives.
        if !force && self.job.is_pending() {
            return Ok(());
        }

        let outdated = self.is_outdated(dur, notes_ref);

        if !force && !outdated {
            self.sender.send(AsyncGitNotification::FinishUnchanged)?;
            return Ok(());
        }

        let repo = self.repo.clone();

        self.job.spawn(AsyncNotesJob::new(
            self.last.as_ref().map_or(0, |(_, result)| result.hash),
            repo,
            notes_ref.map(str::to_string),
        ));

        let _ = self.refresh()?;

        Ok(())
    }

    ///
    pub fn refresh(&mut self) -> Result<bool> {
        // Drain the completed job result without blocking. This is the only
        // place the cached note snapshot should be updated.
        if let Some(job) = self.job.take_last() {
            if let Some(Ok(result)) = job.result() {
                self.last = Some(result);
                return Ok(true);
            }
        }

        Ok(false)
    }
}

enum JobState {
    Request(u64, RepoPath, Option<String>),
    Response(Result<(Instant, NotesResult)>),
}

///
#[derive(Clone, Default)]
pub struct AsyncNotesJob {
    state: Arc<Mutex<Option<JobState>>>,
}

///
impl AsyncNotesJob {
    ///
    pub fn new(last_hash: u64, repo: RepoPath, notes_ref: Option<String>) -> Self {
        Self {
            state: Arc::new(Mutex::new(Some(JobState::Request(last_hash, repo, notes_ref)))),
        }
    }

    ///
    pub fn result(&self) -> Option<Result<(Instant, NotesResult)>> {
        if let Ok(mut state) = self.state.lock() {
            if let Some(state) = state.take() {
                return match state {
                    JobState::Request(_, _, _) => None,
                    JobState::Response(result) => Some(result),
                };
            }
        }

        None
    }
}

impl AsyncJob for AsyncNotesJob {
    type Notification = AsyncGitNotification;
    type Progress = ();

    fn run(
        &mut self,
        _params: RunParams<Self::Notification, Self::Progress>,
    ) -> Result<Self::Notification> {
        let mut notification = AsyncGitNotification::FinishUnchanged;
        if let Ok(mut state) = self.state.lock() {
            *state = state.take().map(|state| match state {
                JobState::Request(last_hash, repo, notes_ref) => {
                    let notes = sync::list_notes(&repo, notes_ref.as_deref());

                    JobState::Response(notes.map(|notes| {
                        let hash = hash(&notes);
                        if last_hash != hash {
                            notification = AsyncGitNotification::Notes;
                        }

                        (
                            Instant::now(),
                            NotesResult {
                                hash,
                                notes,
                                notes_ref,
                            },
                        )
                    }))
                }
                JobState::Response(result) => JobState::Response(result),
            });
        }

        Ok(notification)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::Path, time::Duration};

    use crossbeam_channel::unbounded;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn async_notes_roundtrip() -> Result<()> {
        let td = TempDir::new()?;
        let repo = git2::Repository::init(td.path())?;
        {
            let mut config = repo.config()?;
            config.set_str("user.name", "name")?;
            config.set_str("user.email", "email")?;
        }

        let file_path = Path::new("foo");
        File::create(td.path().join(file_path))?.write_all(b"a")?;
        let mut index = repo.index()?;
        index.add_path(file_path)?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let sig = repo.signature()?;
        repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])?;

        let root = repo.path().parent().unwrap();
        let repo_path_owned: RepoPath = root.as_os_str().to_str().unwrap().into();
        let repo_path: &RepoPath = &repo_path_owned;
        let head = repo.head()?.target().unwrap();

        sync::add_note(repo_path, head, "hello async notes", None, false)?;

        let (sender, receiver) = unbounded();
        let mut async_notes = AsyncNotes::new(repo_path_owned, &sender);
        async_notes.request(Duration::from_secs(0), true, None)?;

        let mut saw_notes = false;
        for _ in 0..10 {
            if let Ok(notification) = receiver.recv_timeout(Duration::from_secs(1)) {
                if notification == crate::AsyncGitNotification::Notes {
                    saw_notes = true;
                    break;
                }
            }
        }

        assert!(saw_notes);
        assert!(async_notes.refresh()?);
        assert_eq!(async_notes.last()?.unwrap()[0].message, "hello async notes");

        Ok(())
    }
}
