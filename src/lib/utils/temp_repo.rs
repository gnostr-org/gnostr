use git2::{Repository, Signature};
use std::fs::File;
use std::io::prelude::*;
use tempfile::{tempdir, TempDir};

pub fn create_temp_repo() -> Result<(Repository, TempDir), git2::Error> {
    let dir = tempdir().expect("Failed to create temporary directory");
    let repo_path = dir.path();

    let repo = Repository::init(repo_path)?;
    println!(
        "Successfully initialized a new Git repository at: {}",
        repo_path.display()
    );

    let file_path = repo_path.join("README.md");
    let mut file = File::create(&file_path).expect("Failed to create file");
    writeln!(file, "# My Temporary Git Repository").expect("Failed to write to file");

    let mut index = repo.index()?;

    let relative_path = file_path.strip_prefix(repo_path).expect("Path error");
    index.add_path(relative_path)?;

    let oid = index.write_tree()?;

    let signature = Signature::now("John Doe", "john.doe@example.com")?;

    // Create a new scope block to control the lifetime of the `tree` variable.
    {
        // The `tree` variable is created here and borrows `repo`.
        let tree = repo.find_tree(oid)?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )?;
        // `tree` is dropped here at the end of the scope block,
        // releasing its borrow on `repo`.
    } // End of the scope block.

    println!("Successfully made an initial commit.");

    // Now it is safe to move `repo` out of the function.
    Ok((repo, dir))
}

//fn main() {
//    match create_temp_repo() {
//        Ok((_repo, _dir)) => {
//            println!("Temporary repository created and used successfully!");
//        }
//        Err(e) => eprintln!("Error: {}", e),
//    }
//
//    println!("Temporary directory has been cleaned up.");
//}
