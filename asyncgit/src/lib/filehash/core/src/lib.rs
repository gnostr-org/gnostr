//! Core file-hash helpers.

use std::{path::PathBuf, process::Command};

pub fn should_remove_relay(error_msg: &str) -> bool {
    error_msg.contains("relay not connected")
        || error_msg.contains("not in web of trust")
        || error_msg.contains("blocked: not authorized")
        || error_msg.contains("timeout")
        || error_msg.contains("blocked: spam not permitted")
        || error_msg.contains("relay experienced an error trying to publish the latest event")
        || error_msg.contains("duplicate: event already broadcast")
}

pub fn get_git_tracked_files(dir: &PathBuf) -> Vec<String> {
    match Command::new("git")
        .arg("ls-files")
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
    {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(String::from)
                .collect()
        }
        Ok(output) => {
            println!(
                "cargo:warning=git ls-files failed or returned empty. Status: {:?}, Stderr: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            Vec::new()
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute git ls-files: {}", e);
            Vec::new()
        }
    }
}

#[macro_export]
macro_rules! get_file_hash {
    ($file_path:expr) => {{
        let bytes = include_bytes!($file_path);
        let mut hasher = sha2::Sha256::new();
        sha2::Digest::update(&mut hasher, bytes);
        let result = sha2::Digest::finalize(hasher);

        result
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{fs::File, io::Write};
    use tempfile::tempdir;

    #[test]
    fn test_get_file_hash() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_file.txt");
        File::create(&file_path)
            .unwrap()
            .write_all(b"Hello, world!")
            .unwrap();

        let macro_hash = get_file_hash!("lib.rs");
        let bytes = include_bytes!("lib.rs");
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let expected = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        assert_eq!(macro_hash, expected);
    }

    #[test]
    fn test_get_git_tracked_files() {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();

        let _ = Command::new("git")
            .arg("init")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to initialize git repo");

        let file1_path = repo_path.join("file1.txt");
        File::create(&file1_path).unwrap().write_all(b"content1").unwrap();
        let file2_path = repo_path.join("file2.txt");
        File::create(&file2_path).unwrap().write_all(b"content2").unwrap();

        let _ = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to git add files");
        let _ = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg("Initial commit")
            .current_dir(repo_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .expect("Failed to git commit");

        let tracked_files = get_git_tracked_files(&repo_path.to_path_buf());
        assert_eq!(tracked_files.len(), 2);
        assert!(tracked_files.contains(&"file1.txt".to_string()));
        assert!(tracked_files.contains(&"file2.txt".to_string()));
    }
}
