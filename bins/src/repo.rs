extern crate git2;
use std::process::Command;

use git2::{Repository, RepositoryState};

pub trait ToString {
    fn to_string(&self) -> String;
}

pub fn state() -> RepositoryState {
    let get_pwd = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", "echo %cd%"])
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("echo `pwd`")
            .output()
            .expect("failed to execute process")
    };

    let _pwd = String::from_utf8(get_pwd.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();
    //println!("pwd={:?}", pwd);

    //let repo_root = std::env::args().nth(1).unwrap_or(pwd);

    let repo_root = std::env::args().nth(1).unwrap_or(".".to_string());
    //println!("repo_root={:?}", repo_root.as_str());
    let repo = Repository::open(repo_root.as_str()).expect("Couldn't open repository");
    //println!("{} state={:?}", repo.path().display(), repo.state());
    //println!("state={:?}", repo.state());
    if repo.state() == RepositoryState::Clean {
        //println!("clean {:?}", repo.state());

        let _repo_state = if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/C", "git status"])
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("sh")
                .arg("-c")
                .arg("git status")
                .output()
                .expect("failed to execute process")
        };

        //let state = String::from_utf8(repo_state.stdout)
        //    .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        //    .unwrap();
        ////println!("state={:?}", state);
    }
    repo.state()
}
