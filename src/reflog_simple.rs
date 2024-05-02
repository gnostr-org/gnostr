extern crate getopts;
use std::env;
use getopts::Options;
use git2::{Repository, Revwalk, Commit};

use std::process;

pub fn reflog_simple() -> Result<(), git2::Error> {

  let repo = match Repository::open_bare(".") {
    Ok(repo) => repo,
    Err(e) => panic!("Error opening repository: {}", e),
};

let mut revwalk = repo.revwalk()?;

revwalk.push_head()?;
revwalk.set_sorting(git2::Sort::TIME)?;


for rev in revwalk {
    let commit = repo.find_commit(rev?)?;
    let message = commit.summary_bytes().unwrap_or_else(|| commit.message_bytes());
    println!("{}\t{}", commit.id(), String::from_utf8_lossy(message));
}


//let mut revwalk = match Revwalk::new(&repo) {
//    Ok(revwalk) => revwalk,
//    Err(e) => panic!("Error creating revwalk: {}", e),
//};

// Start at HEAD and show the last 10 commits
//revwalk.set_head(repo.find_reference("HEAD")?);
//revwalk.set_limit(10);

// Walk the commit history and process each commit
//for commit in revwalk {
//    let commit = commit?;
//    // Do something with the commit
//}

//process::exit(0);


//    // Collect and parse arguments
//    let args: Vec<String> = env::args().collect();
//    let mut opts = Options::new();
//    opts.optopt("r", "ref", "Specify the Git reference (default: HEAD)", "REF");
//    opts.optopt("n", "number", "Specify the maximum number of commits to show (default: 10)", "NUMBER");
//
//    let matches = opts.parse(&args[1..]).unwrap();
//
//    // Extract and validate arguments
//    let ref_name = matches.opt_str("r").unwrap_or("HEAD".to_string());
//    let num_commits = matches.opt_str("n")
//        .unwrap_or("10".to_string())
//        .parse::<i32>();
//        //.parse::<i32>()?;
//
//    println!("num_commits={:?}", num_commits);
//    //if num_commits <= "0" {
//    //    //return Err(git2::Error::from_str("0",git2::ErrorClass::Invalid, "Number of commits must be positive"));
//    //    return Err(git2::Error::from_str("0"));
//    //}
//
////    // Open the Git repository
////    let repo = Repository::open_bare(".")?;
////
////    // Create a Revwalk object
////    let mut revwalk = Revwalk::new(&repo)?;
////
////    // Set starting commit
////    revwalk.set_head(repo.find_reference_name(&ref_name)?);
////
////    // Limit the number of commits
////    revwalk.set_limit(num_commits as u32);
//
//    // Walk the commit history and print commit IDs
////    for commit in revwalk {
////        let commit = commit?;
////        println!("{}", commit.id());
////    }

    Ok(())
}

fn format_commit(commit: &Commit, format_str: &str, show_date: bool, show_message: bool) -> String {
    // This function is not implemented in this example, but provides a placeholder for future enhancements
    // It would allow more flexible output formatting based on the provided format string and boolean flags.
    panic!("Formatting not implemented.");
}

