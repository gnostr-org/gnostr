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
