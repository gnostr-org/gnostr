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

    run(std::env::args().skip(1).collect(), &weeble_cmd_output, &blockheight_cmd_output, &wobble_cmd_output)
}

fn run(args: Vec<String>, weeble: &str, blockheight: &str, wobble: &str) -> Result<()> {
    let weeble = weeble.trim().to_string();
    let blockheight = blockheight.trim().to_string();
    let wobble = wobble.trim().to_string();

    let mut tag_name = format!("v{}.{}.{}",
        if weeble.is_empty() { "0" } else { &weeble },
        if blockheight.is_empty() { "0" } else { &blockheight },
        if wobble.is_empty() { "0" } else { &wobble },
    );

    if args.len() > 0 {
        tag_name = format!("{}-{}", tag_name, args[0]);
    }

    let output = Command::new("git").arg("tag").arg("-f").arg(&tag_name).output()?;

    if !output.status.success() {
        eprintln!("Error creating branch: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create branch");
    }

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
    #[ignore]
    fn test_git_tag_version_no_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let weeble = gnostr::weeble::weeble().unwrap_or(0.0).to_string();
        let blockheight = gnostr::blockheight::blockheight().unwrap_or(0.0).to_string();
        let wobble = gnostr::wobble::wobble().unwrap_or(0.0).to_string();
        let expected_tag_name = format!("v{}.{}.{}", weeble.clone(), blockheight.clone(), wobble.clone());
        println!("expected_tag_name={}", expected_tag_name);

        let _ = run(vec![], &weeble, &blockheight, &wobble);

        let current_tag_output = Command::new("git").arg("describe").arg("--tags").output().unwrap().stdout;
        let current_tag = String::from_utf8_lossy(&current_tag_output).trim().to_string();
        assert_eq!(current_tag, expected_tag_name);

        // Clean up the tag
        Command::new("git").arg("tag").arg("-d").arg(&expected_tag_name).output().unwrap();

        Ok(())
    }

    #[test]
    #[ignore]
    fn test_git_tag_version_with_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let weeble = gnostr::weeble::weeble().unwrap().to_string();
        let blockheight = gnostr::blockheight::blockheight().unwrap().to_string();
        let wobble = gnostr::wobble::wobble().unwrap().to_string();

        let suffix = "test_suffix";
        std::env::set_current_dir(repo_path)?;

        let expected_tag_name = format!("v{}.{}.{}-{}", weeble, blockheight, wobble, suffix);
        println!("expected_tag_name={}", expected_tag_name);
        let _ = run(vec![suffix.to_string()], &weeble, &blockheight, &wobble);

        // Verify the tag was created
        let tag_list_output = Command::new("git").arg("tag").arg("-l").arg(&expected_tag_name).output().unwrap().stdout;
        let tag_exists = String::from_utf8_lossy(&tag_list_output).trim().to_string();
        assert_eq!(tag_exists, expected_tag_name);

        // Clean up the tag
        Command::new("git").arg("tag").arg("-d").arg(&expected_tag_name).output().unwrap();

        Ok(())
    }
}
