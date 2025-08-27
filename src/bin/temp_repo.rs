use git2::{Repository, Signature};
use std::fs::File;
use std::io::prelude::*;
use tempfile::{tempdir, TempDir};

use gnostr::utils::temp_repo::*;
fn main() {
    match create_temp_repo() {
        Ok((_repo, _dir)) => {
            println!("Temporary repository created and used successfully!");
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("Temporary directory has been cleaned up.");
}
