use gnostr_asyncgit::git2::Repository;
use gnostr_asyncgit::git2::RepositoryState;

pub fn state(repo_root: &str) -> RepositoryState {
    debug!("Discovering repository from {}", repo_root);
    let repo = Repository::discover(repo_root).expect("Couldn't open repository");
    let state = repo.state();
    debug!(
        "Discovered repository: path={}, state={:?}",
        repo.path().display(),
        state
    );
    state
}
