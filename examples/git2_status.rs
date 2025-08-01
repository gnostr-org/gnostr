use git2::{Repository, Status, StatusOptions};
use std::path::Path;

fn main() -> Result<(), git2::Error> {
    // Open an existing repository (replace "." with your repo path if needed)
    let repo = Repository::open(".")?;

    let mut opts = StatusOptions::new();
    // You can configure what types of statuses you want to include
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .exclude_submodules(true); // Or false, depending on your needs

    let statuses = repo.statuses(Some(&mut opts))?;

    for entry in statuses.iter() {
        let status = entry.status();

        // Get the path of the file
        let path = if let Some(path_str) = entry.path() {
            Path::new(path_str)
        } else {
            // This might happen for certain status types or unusual cases
            continue;
        };

        // Determine the status characters (like 'M', 'A', 'D', '??')
        let (index_status, workdir_status) = {
            let mut i = ' ';
            let mut w = ' ';

            if status.contains(Status::INDEX_NEW) {
                i = 'A';
            } else if status.contains(Status::INDEX_MODIFIED) {
                i = 'M';
            } else if status.contains(Status::INDEX_DELETED) {
                i = 'D';
            } else if status.contains(Status::INDEX_RENAMED) {
                i = 'R';
            } else if status.contains(Status::INDEX_TYPECHANGE) {
                i = 'T';
            }

            if status.contains(Status::WT_NEW) {
                w = '?';
            }
            // Untracked
            else if status.contains(Status::WT_MODIFIED) {
                w = 'M';
            } else if status.contains(Status::WT_DELETED) {
                w = 'D';
            } else if status.contains(Status::WT_RENAMED) {
                w = 'R';
            } else if status.contains(Status::WT_TYPECHANGE) {
                w = 'T';
            }

            (i, w)
        };

        // Print the status and path
        println!("{}{} {}", index_status, workdir_status, path.display());

        // You can also get more detailed information
        if let Some(h2i) = entry.head_to_index() {
            // Changes between HEAD and index
            let old_file = h2i.old_file().path().unwrap_or(Path::new(""));
            let new_file = h2i.new_file().path().unwrap_or(Path::new(""));
            if old_file != new_file {
                println!(
                    "    (Indexed path changed from {} to {})",
                    old_file.display(),
                    new_file.display()
                );
            }
        }

        if let Some(i2w) = entry.index_to_workdir() {
            // Changes between index and working directory
            let old_file = i2w.old_file().path().unwrap_or(Path::new(""));
            let new_file = i2w.new_file().path().unwrap_or(Path::new(""));
            if old_file != new_file {
                println!(
                    "    (Workdir path changed from {} to {})",
                    old_file.display(),
                    new_file.display()
                );
            }
        }
    }

    Ok(())
}
