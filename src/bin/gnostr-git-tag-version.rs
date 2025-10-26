use std::process::Command;
use anyhow::Result;
use std::env::args;

fn main() -> Result<()> {
    let weeble_output = Command::new("gnostr-weeble").output()?.stdout;
    let weeble = String::from_utf8_lossy(&weeble_output).trim().to_string();

    let blockheight_output = Command::new("gnostr-blockheight").output()?.stdout;
    let blockheight = String::from_utf8_lossy(&blockheight_output).trim().to_string();

    let wobble_output = Command::new("gnostr-wobble").output()?.stdout;
    let wobble = String::from_utf8_lossy(&wobble_output).trim().to_string();

    let args: Vec<String> = std::env::args().collect();
    let mut tag_name = if args.len() > 1 {
        format!("v{}-{}.{}.{}",
            args[1],
            if weeble.is_empty() { "0" } else { &weeble },
            if blockheight.is_empty() { "0" } else { &blockheight },
            if wobble.is_empty() { "0" } else { &wobble }
        )
    } else {
        format!("v{}.{}.{}",
            if weeble.is_empty() { "0" } else { &weeble },
            if blockheight.is_empty() { "0" } else { &blockheight },
            if wobble.is_empty() { "0" } else { &wobble }
        )
    };

    println!("Creating tag: {}", tag_name);
    let output = Command::new("git").arg("tag").arg(&tag_name).output()?;

    if !output.status.success() {
        eprintln!("Error creating tag: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create tag");
    }

    let grep_output = Command::new("git").arg("tag").output()?.stdout;
    let grep_output_str = String::from_utf8_lossy(&grep_output);
    let filtered_tags = grep_output_str
        .lines()
        .filter(|line| line.contains(&weeble))
        .collect::<Vec<&str>>();

    for tag in filtered_tags {
        println!("{}", tag);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_test_repo() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        let repo_path = dir.path();
        Command::new("git").arg("init").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("config").arg("user.email").arg("test@example.com").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("config").arg("user.name").arg("Test User").current_dir(repo_path).output().unwrap();
        fs::write(repo_path.join("file.txt"), "initial content").unwrap();
        Command::new("git").arg("add").arg(".").current_dir(repo_path).output().unwrap();
        Command::new("git").arg("commit").arg("-m").arg("Initial commit").current_dir(repo_path).output().unwrap();
        dir
    }

    #[test]
    fn test_git_tag_version_no_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        std::env::set_var("PATH", format!("{}:{}", std::env::var("PATH").unwrap(), repo_path.display()));
        fs::write(repo_path.join("gnostr-weeble"), "#!/bin/bash\necho 1").unwrap();
        fs::write(repo_path.join("gnostr-blockheight"), "#!/bin/bash\necho 2").unwrap();
        fs::write(repo_path.join("gnostr-wobble"), "#!/bin/bash\necho 3").unwrap();
        Command::new("chmod").arg("+x").arg(repo_path.join("gnostr-weeble")).output().unwrap();
        Command::new("chmod").arg("+x").arg(repo_path.join("gnostr-blockheight")).output().unwrap();
        Command::new("chmod").arg("+x").arg(repo_path.join("gnostr-wobble")).output().unwrap();

        let expected_tag_name = "v1.2.3".to_string();

        let result = Command::new(std::env::current_exe().unwrap())
            .arg("gnostr-git-tag-version")
            .current_dir(repo_path)
            .output();
        
        assert!(result.is_ok());
        assert!(result.unwrap().status.success());

        let tags_output = Command::new("git").arg("tag").current_dir(repo_path).output().unwrap().stdout;
        let tags = String::from_utf8_lossy(&tags_output).to_string();
        assert!(tags.contains(&expected_tag_name));

        Ok(())
    }

    #[test]
    fn test_git_tag_version_with_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        std::env::set_var("PATH", format!("{}:{}", std::env::var("PATH").unwrap(), repo_path.display()));
        fs::write(repo_path.join("gnostr-weeble"), "#!/bin/bash\necho 1").unwrap();
        fs::write(repo_path.join("gnostr-blockheight"), "#!/bin/bash\necho 2").unwrap();
        fs::write(repo_path.join("gnostr-wobble"), "#!/bin/bash\necho 3").unwrap();
        Command::new("chmod").arg("+x").arg(repo_path.join("gnostr-weeble")).output().unwrap();
        Command::new("chmod").arg("+x").arg(repo_path.join("gnostr-blockheight")).output().unwrap();
        Command::new("chmod").arg("+x").arg(repo_path.join("gnostr-wobble")).output().unwrap();

        let suffix = "beta";
        let expected_tag_name = format!("v{}-1.2.3", suffix);

        let result = Command::new(std::env::current_exe().unwrap())
            .arg("gnostr-git-tag-version")
            .arg(suffix)
            .current_dir(repo_path)
            .output();

        assert!(result.is_ok());
        assert!(result.unwrap().status.success());

        let tags_output = Command::new("git").arg("tag").current_dir(repo_path).output().unwrap().stdout;
        let tags = String::from_utf8_lossy(&tags_output).to_string();
        assert!(tags.contains(&expected_tag_name));

        Ok(())
    }
}
