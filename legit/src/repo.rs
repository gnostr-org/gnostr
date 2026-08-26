use gnostr_asyncgit::git2::Repository;
use gnostr_asyncgit::git2::RepositoryState;

pub fn state(repo_root: &str) -> RepositoryState {
    let repo = Repository::discover(repo_root).expect("Couldn't open repository");
    //println!("{} state={:?}", repo.path().display(), repo.state());
    //println!("state={:?}", repo.state());
    if repo.state() == RepositoryState::Clean {
        //println!("clean {:?}", repo.state());
    }
    return repo.state();
}
