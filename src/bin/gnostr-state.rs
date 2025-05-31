use std::ffi::OsString;

use git2::Repository;

fn main() -> Result<(), git2::Error> {
    // Get path to git repo via command line args or assume current directory
    let repo_root: OsString = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| OsString::from("."));

    // Open git repo
    let repo = Repository::open(&repo_root).expect("Couldn't open repository");

    println!("{:?}", repo.state());

    Ok(())
}
