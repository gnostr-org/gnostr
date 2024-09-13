use std::process::Command;

use ascii::AsciiChar;
use git2::Repository;

//pub fn strip_trailing_nl(input: &mut String) {
//    let new_len = input
//        .char_indices()
//        .rev()
//        .find(|(_, c)| !matches!(c, '\n' | '\r'))
//        .map_or(0, |(i, _)| i + 1);
//    if new_len != input.len() {
//        input.truncate(new_len);
//    }
//}

pub fn pwd() -> Result<String, &'static str> {
    let get_pwd = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo %cd%"])
            .output()
            .expect("failed to execute process")
    } else if cfg!(target_os = "macos") {
        Command::new("sh")
            .arg("-c")
            .arg("echo ${PWD##*/}")
            .output()
            .expect("failed to execute process")
    } else if cfg!(target_os = "linux") {
        Command::new("sh")
            .arg("-c")
            .arg("echo ${PWD##*/}")
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo ${PWD##*/}")
            .output()
            .expect("failed to execute process")
    };

    let mut _pwd = String::from_utf8(get_pwd.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    let _mutable_string = String::new();
    let mutable_string = _pwd.clone();
    Ok(format!("{}", mutable_string))
} //end pwd()

pub fn ref_hash_list_padded() -> Result<(), git2::Error> {
    let repo = match Repository::open(".") {
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
    let repo = match Repository::open(".") {
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
    //let brief = format!("Usage: {} FILE [options]", _program);
    //print!("ref_hash_list_commit_message:\n{}", _opts.usage(&brief));
    let repo = match Repository::open(".") {
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
