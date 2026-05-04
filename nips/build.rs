use std::env;
use std::path::PathBuf;
use std::process::Command;

use std::time::{SystemTime, UNIX_EPOCH};

fn _get_current_time_in_seconds() -> Result<u64, std::io::Error> {
    let now = SystemTime::now();
    let since_the_epoch = now.duration_since(UNIX_EPOCH).expect("0");
    Ok(since_the_epoch.as_secs())
}

fn _get_current_time_in_seconds_string() -> String {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}", duration.as_secs()),
        Err(e) => format!("Error getting time: {}", e),
    }
}

fn main() {
    println!("cargo:rerun-if-changed=."); // Re-run if any file in the project changes

    if !is_git_available() {
        eprintln!("Error: Git is not installed or not in the system's PATH.");
        std::process::exit(1);
    }

    let repo_path = match env::var("CARGO_MANIFEST_DIR") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            eprintln!("Error getting CARGO_MANIFEST_DIR");
            return;
        }
    };

    let remote_name = "upstream"; //.to_owned(); // + &get_current_time_in_seconds_string();
    let remote_url = "https://github.com/nostr-protocol/nips.git";

    //let remote_name = "upstream"; // The name of the remote you want to check

    if remote_exists(&repo_path, &remote_name) {
        println!("Remote '{}' already exists.", remote_name);
        // You can perform actions based on the remote existing here
        //

        println!(
            "fetching onto '{}' with URL '{}'...",
            remote_name.to_owned(),
            remote_url
        );
        if let Err(e) =
            git_fetch_upstream_master("git", &["fetch", &"-pf", &"upstream"], &repo_path)
        {
            eprintln!("Error adding Git remote '{}': {}", remote_name, e);
            // Depending on your needs, you might want to exit here or continue.
            // If this remote is not critical for the build, you might just warn.
            return;
        }
        println!("Git remote '{}' added successfully.", remote_name);

        println!(
            "rebasing onto '{}' with URL '{}'...",
            remote_name.to_owned(),
            remote_url
        );
        if let Err(e) = git_rebase_upstream_master(
            "git",
            &[
                "rebase",
                &format!("{}{}", remote_name, String::from("/master")),
            ],
        ) {
            eprintln!("Error adding Git remote '{}': {}", remote_name, e);
            // Depending on your needs, you might want to exit here or continue.
            // If this remote is not critical for the build, you might just warn.
            return;
        }
        println!("Git remote '{}' added successfully.", remote_name);
    } else {
        println!("Remote '{}' does not exist.", remote_name);

        if let Err(e) = git_remote_add_upstream(
            "git",
            &["remote", "add", &remote_name, remote_url],
            &repo_path,
        ) {
            println!("Error adding Git remote '{}': {}", remote_name, e);
            // Depending on your needs, you might want to exit here or continue.
            // If this remote is not critical for the build, you might just warn.
            return;
        }
        println!("Git remote '{}' added successfully.", remote_name);

        // You can perform actions based on the remote not existing here,
        // such as adding it.
    }
    //println!("Adding Git remote '{}' with URL '{}'...", remote_name, remote_url);
    //if let Err(e) = git_remote_add_upstream(
    //    "git",
    //    &["remote", "add", &remote_name, remote_url],
    //    &repo_path,
    //) {
    //    println!("Error adding Git remote '{}': {}", remote_name, e);
    //    // Depending on your needs, you might want to exit here or continue.
    //    // If this remote is not critical for the build, you might just warn.
    //    return;
    //}
    //println!("Git remote '{}' added successfully.", remote_name);

    //println!("rebasing onto '{}' with URL '{}'...", remote_name.to_owned(), remote_url);
    //if let Err(e) = git_rebase_upstream_master(
    //    "git",
    //    &["rebase", &remote_name],
    //) {
    //    eprintln!("Error adding Git remote '{}': {}", remote_name, e);
    //    // Depending on your needs, you might want to exit here or continue.
    //    // If this remote is not critical for the build, you might just warn.
    //    return;
    //}
    //println!("Git remote '{}' added successfully.", remote_name);
}

fn is_git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn git_remote_add_upstream(
    command: &str,
    args: &[&str],
    current_dir: &PathBuf,
) -> Result<(), String> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.current_dir(current_dir);

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => return Err(format!("Failed to execute command `{}`: {}", command, e)),
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command `{}` failed with exit code {:?}:\nStdout: {}\nStderr: {}",
            command,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
fn git_fetch_upstream_master(
    command: &str,
    args: &[&str],
    current_dir: &PathBuf,
) -> Result<(), String> {
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.current_dir(current_dir);

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => return Err(format!("Failed to execute command `{}`: {}", command, e)),
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command `{}` failed with exit code {:?}:\nStdout: {}\nStderr: {}",
            command,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}
fn git_rebase_upstream_master(command: &str, args: &[&str]) -> Result<(), String> {
    let mut cmd = Command::new(command);
    cmd.args(args);

    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => return Err(format!("Failed to execute command `{}`: {}", command, e)),
    };

    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command `{}` failed with exit code {:?}:\nStdout: {}\nStderr: {}",
            command,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

//fn main() {
//    println!("cargo:rerun-if-changed=."); // Re-run if any file in the project changes
//
//    if !is_git_available() {
//        eprintln!("Error: Git is not installed or not in the system's PATH.");
//        std::process::exit(1);
//    }
//
//    let repo_path = match env::var("CARGO_MANIFEST_DIR") {
//        Ok(path) => PathBuf::from(path),
//        Err(_) => {
//            eprintln!("Error getting CARGO_MANIFEST_DIR");
//            return;
//        }
//    };
//
//    let remote_name = "upstream"; // The name of the remote you want to check
//
//    if remote_exists(&repo_path, remote_name) {
//        println!("Remote '{}' already exists.", remote_name);
//        // You can perform actions based on the remote existing here
//    } else {
//        println!("Remote '{}' does not exist.", remote_name);
//        // You can perform actions based on the remote not existing here,
//        // such as adding it.
//    }
//}
//

fn remote_exists(repo_path: &PathBuf, remote_name: &str) -> bool {
    let output = Command::new("git")
        .args(&["remote"])
        .current_dir(repo_path)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let remotes = String::from_utf8_lossy(&output.stdout);
                remotes.lines().any(|line| line.trim() == remote_name)
            } else {
                eprintln!(
                    "Error checking remotes: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                false
            }
        }
        Err(e) => {
            eprintln!("Error executing git remote: {}", e);
            false
        }
    }
}
