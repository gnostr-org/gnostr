use ascii::AsciiChar;
use git2::Repository;

pub fn ref_hash_list_padded() -> Result<(), git2::Error> {
    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(e) => panic!("Error opening repository: {}", e),
    };

    let mut revwalk = repo.revwalk()?;

    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    for rev in revwalk {
        let commit = repo.find_commit(rev?)?;
        println!("{:0>64}", commit.id());
    }
    Ok(())
} //end ref_hash_list_padded

pub fn ref_hash_list() -> Result<(), git2::Error> {
    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(e) => panic!("Error opening repository: {}", e),
    };

    let mut revwalk = repo.revwalk()?;

    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    for rev in revwalk {
        let commit = repo.find_commit(rev?)?;
        println!("{:}", commit.id());
    }
    Ok(())
} //end ref_hash_list

pub fn ref_hash_list_w_commit_message() -> Result<(), git2::Error> {
    let repo = match Repository::discover(".") {
        Ok(repo) => repo,
        Err(e) => panic!("Error opening repository: {}", e),
    };

    let mut revwalk = repo.revwalk()?;

    revwalk.push_head()?;
    revwalk.set_sorting(git2::Sort::TIME)?;

    for rev in revwalk {
        let commit = repo.find_commit(rev?)?;
        let message = commit
            .summary_bytes()
            .unwrap_or_else(|| commit.message_bytes());
        assert_eq!(AsciiChar::new('@'), AsciiChar::At);
        println!(
            "{:0}/commit/{}",
            commit.id(),
            String::from_utf8_lossy(message)
        );
    }
    Ok(())
} //end ref_hash_list_w_commit_message
