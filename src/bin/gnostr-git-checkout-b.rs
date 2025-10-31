use std::process::Command;
use anyhow::Result;
use std::path::Path;
use gnostr::weeble::weeble;
use gnostr::wobble::wobble;
use gnostr::blockheight::blockheight;


fn main() -> Result<()> {
    let weeble_output = Command::new("gnostr-weeble").output()?.stdout;
    let weeble_cmd_output = String::from_utf8_lossy(&weeble_output).trim().to_string();

    let blockheight_output = Command::new("gnostr-blockheight").output()?.stdout;
    let blockheight_cmd_output = String::from_utf8_lossy(&blockheight_output).trim().to_string();

    let wobble_output = Command::new("gnostr-wobble").output()?.stdout;
    let wobble_cmd_output = String::from_utf8_lossy(&wobble_output).trim().to_string();

    run(std::env::args().collect(), &weeble_cmd_output, &blockheight_cmd_output, &wobble_cmd_output, &std::env::current_dir()?)
}

fn run(args: Vec<String>, weeble_output: &str, blockheight_output: &str, wobble_output: &str, repo_path: &Path) -> Result<()> {
    let weeble = weeble_output.trim().to_string();
    let blockheight = blockheight_output.trim().to_string();
    let wobble = wobble_output.trim().to_string();

    let head_parent_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").output()?.stdout;
    let head_parent = String::from_utf8_lossy(&head_parent_output).trim().to_string();

    let head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").output()?.stdout;
    let head = String::from_utf8_lossy(&head_output).trim().to_string();

    let mut branch_name = format!("{}/{}/{}/{}/{}",
        if weeble.is_empty() { "0" } else { &weeble },
        if blockheight.is_empty() { "0" } else { &blockheight },
        if wobble.is_empty() { "0" } else { &wobble },
        head_parent,
        head
    );

    if args.len() > 1 {
        branch_name = format!("{}-{}", branch_name, args[1]);
    }

    let output = Command::new("git").arg("checkout").arg("-b").arg(&branch_name).current_dir(repo_path).output()?;

    if !output.status.success() {
        eprintln!("Error creating branch: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create branch");
    }
    print!("{}", branch_name);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    // Helper to create a dummy git repo for testing
    fn setup_test_repo() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        Command::new("git").arg("init").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("config").arg("user.email").arg("test@example.com").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("config").arg("user.name").arg("Test User").current_dir(repo_path).output().unwrap();
        fs::write(repo_path.join("file.txt"), "initial content").unwrap();
        Command::new("git").arg("add").arg(".").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("commit").arg("-m").arg("Initial commit").current_dir(repo_path).output().unwrap();
        fs::write(repo_path.join("file.txt"), "second content").unwrap();
        Command::new("git").arg("add").arg(".").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("commit").arg("-m").arg("Second commit").current_dir(repo_path).output().unwrap();
        dir
    }

    #[test]
    //#[ignore]
    fn test_git_checkout_b_no_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let current_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_head = String::from_utf8_lossy(&current_head_output).trim().to_string();
        let parent_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output().unwrap().stdout;
        let parent_head = String::from_utf8_lossy(&parent_head_output).trim().to_string();

        let weeble = gnostr::weeble::weeble().unwrap().to_string();
        let blockheight = gnostr::blockheight::blockheight().unwrap().to_string();
        let wobble = gnostr::wobble::wobble().unwrap().to_string();
        let expected_branch_name = format!("{}/{}/{}/{}/{}", weeble, blockheight, wobble, parent_head, current_head);

        std::env::set_current_dir(repo_path)?;
        let args_vector: Vec<String> = vec![String::from("")];
        let _ = run(args_vector, &weeble, &blockheight, &wobble, &std::env::current_dir()?);

        // Verify the branch was created and checked out
        let current_branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_branch = String::from_utf8_lossy(&current_branch_output).trim().to_string();
        assert_eq!(current_branch, expected_branch_name);

        Ok(())
    }

    #[test]
    fn test_git_checkout_b_with_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let current_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_head = String::from_utf8_lossy(&current_head_output).trim().to_string();
        let parent_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output().unwrap().stdout;
        let parent_head = String::from_utf8_lossy(&parent_head_output).trim().to_string();

        let suffix = "feature";
        let expected_branch_name = format!("1/1/1/{}/{}-{}", parent_head, current_head, suffix);

        std::env::set_current_dir(repo_path)?;
        let args_vector: Vec<String> = vec![String::from("test_arg")];
        let _ = run(args_vector, &"1", &"1", &"1", &std::env::current_dir()?);
        let current_branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_branch = String::from_utf8_lossy(&current_branch_output).trim().to_string();
        assert_eq!(current_branch+"-"+suffix, expected_branch_name);

        Ok(())
    }
}
