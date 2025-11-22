use clap::ArgAction;
use clap::Parser;

use crate::blockheight;
use crate::weeble;
use crate::wobble;

use env_logger::Env;
use anyhow::Result;
use gnostr_asyncgit::gitui;
use std::io::Error;
use std::path::Path;
use std::process::Command;
use which::which;

#[cfg(not(test))]
use crate::ssh::start;

#[cfg(test)]
mod mock_ssh {
    pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
        // In test environment, always return an error for now
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Mock SSH Start Error",
        )))
    }
}

#[cfg(test)]
use mock_ssh::start;

#[derive(Parser, Debug, Clone)]
pub struct GitSubCommand {
    /// Starts the gnostr git SSH server (gitweb).
    #[arg(long)]
    pub gitweb: bool,
    /// Creates a git tag with an optional suffix.
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    pub tag: Option<String>,
    /// Displays local git information (version, path).
    #[arg(long)]
    pub info: bool,
    /// Creates a git branch using gnostr-git-checkout-b.
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    pub checkout_branch: Option<String>,
    /// Creates a git PR branch using gnostr-git-checkout-pr.
    #[arg(long, num_args = 0..=1, default_missing_value = "")]
    pub checkout_pr: Option<String>,
    /// Opens the gitui terminal user interface.
    #[arg(long)]
    pub tui: bool,
}

pub async fn git(sub_command_args: &GitSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = std::env::current_dir()?;
    let repo_path = current_dir.as_path();

    if sub_command_args.gitweb {
        env_logger::init_from_env(Env::default().default_filter_or("info"));
        let res = start().await;
        if let Err(e) = &res {
            println!("{}", e);
            println!("EXAMPLE:server.toml\n{}", SERVER_TOML);
            println!("check the port in your server.toml is available!\n");
            println!("EXAMPLE:repo.toml\n{}", REPO_TOML);
        }
        return res.map_err(|e| e.into());
    } else if let Some(suffix) = &sub_command_args.tag {
        let owned_suffix = suffix.clone();
        let cloned_repo_path = current_dir.clone();
        let tag_name = tokio::task::spawn_blocking(move || {
            run_git_tag(owned_suffix, &cloned_repo_path)
        })
        .await??;
        println!("{}", tag_name);
        return Ok(());
    } else if let Some(suffix) = &sub_command_args.checkout_branch {
        let owned_suffix = suffix.clone();
        let cloned_repo_path = current_dir.clone();
        let branch_name = tokio::task::spawn_blocking(move || {
            run_git_checkout_b(owned_suffix, &cloned_repo_path)
        })
        .await??;
        println!("{}", branch_name);
        return Ok(());
    } else if let Some(suffix) = &sub_command_args.checkout_pr {
        let owned_suffix = suffix.clone();
        let cloned_repo_path = current_dir.clone();
        let pr_branch_name = tokio::task::spawn_blocking(move || {
            run_git_checkout_pr(owned_suffix, &cloned_repo_path)
        })
        .await??;
        println!("{}", pr_branch_name);
        return Ok(());
    } else if sub_command_args.info {
        println!("{}", get_git_info());
        return Ok(());
    } else if sub_command_args.tui {
        // This will run the gitui TUI
        let term = gnostr_asyncgit::gitui::term::backend();
        let mut terminal = gnostr_asyncgit::gitui::term::Term::new(term)?;
        gnostr_asyncgit::gitui::run(&gnostr_asyncgit::gitui::cli::Args::default(), &mut terminal).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
        return Ok(());
    } else {
        let git_info = get_git_info();
        println!("The 'git' subcommand requires a flag to specify functionality.");
        println!("For example, use '--gitweb' to start the SSH server.");
        println!("Or, use '--tag [SUFFIX]' to create a git tag.");
        println!("Or, use '--info' to display local git information.");
        println!("Or, use '--checkout-branch [SUFFIX]' to create a git branch.");
        println!("Or, use '--checkout-pr [SUFFIX]' to create a git PR branch.");
        println!("Or, use '--tui' to open the git terminal user interface.");
        println!("{}", git_info);
        Ok(())
    }
}

fn get_git_info() -> String {
    let git_path = match which("git") {
        Ok(path) => format!("Git path: {}", path.display()),
        Err(_) => "Git not found in PATH.".to_string(),
    };

    let git_version = match Command::new("git").arg("--version").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        },
        _ => "Git version: Not available.".to_string(),
    };

    format!("\nLocal Git Info:\n{}\n{}", git_path, git_version)
}

fn run_git_tag(suffix: String, repo_path: &Path) -> Result<String> {
    let weeble = weeble::weeble().unwrap_or(0.0).to_string();

    let blockheight = blockheight::blockheight().unwrap_or(0.0).to_string();

    let wobble = wobble::wobble().unwrap_or(0.0).to_string();

    let weeble = weeble;
    let blockheight = blockheight;
    let wobble = wobble;

    let mut tag_name = format!("{}.{}.{}",
        if weeble.is_empty() { "0" } else { &weeble },
        if blockheight.is_empty() { "0" } else { &blockheight },
        if wobble.is_empty() { "0" } else { &wobble },
    );

    if !suffix.is_empty() {
        tag_name = format!("{}-{}", tag_name, suffix);
    }

	//USE git2 instead of system git 
    let output = Command::new("git").arg("tag").arg("-f").arg(&tag_name).current_dir(repo_path).output()?;

    if !output.status.success() {
        eprintln!("Error creating tag: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create tag");
    }
    Ok(tag_name)
}

fn run_git_checkout_b(suffix: String, repo_path: &Path) -> Result<String> {
    let weeble = crate::weeble::weeble().unwrap_or(0.0).to_string();
    let blockheight = crate::blockheight::blockheight().unwrap_or(0.0).to_string();
    let wobble = crate::wobble::wobble().unwrap_or(0.0).to_string();

    let head_parent_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output()?.stdout;
    let head_parent = String::from_utf8_lossy(&head_parent_output).trim().to_string();

    let head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output()?.stdout;
    let head = String::from_utf8_lossy(&head_output).trim().to_string();

    let mut branch_name = format!("{}/{}/{}/{}/{}",
        if weeble.is_empty() { "0" } else { &weeble },
        if blockheight.is_empty() { "0" } else { &blockheight },
        if wobble.is_empty() { "0" } else { &wobble },
        head_parent,
        head
    );

    if !suffix.is_empty() {
        branch_name = format!("{}-{}", branch_name, suffix);
    }

    let output = Command::new("git").arg("checkout").arg("-b").arg(&branch_name).current_dir(repo_path).output()?;

    if !output.status.success() {
        eprintln!("Error creating branch: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create branch");
    }
    Ok(branch_name)
}
fn run_git_checkout_pr(suffix: String, repo_path: &Path) -> Result<String> {
    let weeble = crate::weeble::weeble().unwrap_or(0.0).to_string();
    let blockheight = crate::blockheight::blockheight().unwrap_or(0.0).to_string();
    let wobble = crate::wobble::wobble().unwrap_or(0.0).to_string();

    let head_parent_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output()?.stdout;
    let head_parent = String::from_utf8_lossy(&head_parent_output).trim().to_string();

    let head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output()?.stdout;
    let head = String::from_utf8_lossy(&head_output).trim().to_string();

    let mut branch_name = format!("pr/{}/{}/{}/{}/{}",
        if weeble.is_empty() { "0" } else { &weeble },
        if blockheight.is_empty() { "0" } else { &blockheight },
        if wobble.is_empty() { "0" } else { &wobble },
        head_parent,
        head
    );

    if !suffix.is_empty() {
        branch_name = format!("{}-{}", branch_name, suffix);
    }

    let output = Command::new("git").arg("checkout").arg("-b").arg(&branch_name).current_dir(repo_path).output()?;

    if !output.status.success() {
        eprintln!("Error creating PR branch: {}", String::from_utf8_lossy(&output.stderr));
        anyhow::bail!("Failed to create PR branch");
    }
    Ok(branch_name)
}

static REPO_TOML: &str = r###"#'''
name = "gnostr-gnit-server"
public = true
members = ["gnostr", "gnostr-user"]
failed_push_message = "Issues and patches can be emailed to admin@gnostr.org"
"###;

static SERVER_TOML: &str = r###"#

name = "gnostr.org"

port = 2222



hostname = "gnostr.org"



[users.gnostr]

is_admin = true

public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHwG6aJjwv3Zky9jR/ey2rSgKLMcTOLrMeAyop6fYhjIHqp0dTagHo1j+XHAbVsrjw6oxC0ohTkp8rzH6cYJyjK4TOKApEgCALJUOA2rbHNxr68wAIe2RS36dRQobD3ops2+HoOGk7pkBQazBAlZp/H4monWRrq7tTEw8FkGMX5udZQX6BNEI0vJZqtdkSpG7jSS3aL7GXcuOYKpsTKxuGm5BWsrRPiphsc25U02oe/y3+qM0ceP/njJp3ZvXQ/a2QGPU4+P8WSD+J0oKS+TiRKrpiTR4ChJk8zWupg4PI5zflN3yyK7MrGXg1n0DsvHxPXcqpvVRz4i8ORt6IlKGkve1tC0Wd9pVy4044LDethMORRZFjWAdS/caN1EMgTrrGMxi0DLVw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com"



[users.gnostr-user]

can_create_repos = true

public_key = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQDaBogLsfsOkKIpZEZYa3Ee+wFaaxeJuHps05sH2rZLf+KEE6pWX5MT2iWMgP7ihmm6OqbAPkWoBUGEO5m+m/K1S0MgQXUvaTsTI0II3MDqJT/RXA6Z9c+ZIDROEAkNIDrfeU2n8hQXfMHwG6aJjwv3Zky9jR/ey2rSgKLMcTOLrMeAyop6fYhjIHqp0dTagHo1j+XHAbVsrjw6oxC0ohTkp8rzH6cYJyjK4TOKApEgCALJUOA2rbHNxr68wAIe2RS36dRQobD3ops2+HoOGk7pkBQazBAlZp/H4monWRrq7tTEw8FkGMX5udZQX6BNEI0vJZqtdkSpG7jSS3aL7GXcuOYKpsTKxuGm5BWsrRPiphsc25U02oe/y3+qM0ceP/njJp3ZvXQ/a2QGPU4+P8WSD+J0oKS+TiRKrpiTR4ChJk8zWupg4PI5zflN3yyK7MrGXg1n0DsvHxPXcqpvVRz4i8ORt6IlKGkve1tC0Wd9pVy4044LDethMORRZFjWAdS/caN1EMgTrrGMxi0DLVw6ahedGUgZj2WYWfsrEg8Kzbfk3fn32sO/lMnNyz5hmavMBiNORGlIi2Qe2RjQEtcJHn89B7UtyEfnj87V+jZYcFf4nnNQigT2eQ3NlB1YzZS4Zk/OxQeYypclzYFaiYc7RZv2yxKVOy0KvEpldyUKeQ== randy.lee.mcmillan@gmail.com"



// Optional.

welcome_message = "welcome to gnostr.org!"

"###;



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
    fn test_run_git_checkout_b_no_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        std::env::set_current_dir(repo_path)?;
        let created_branch_name = run_git_checkout_b(String::from(""), repo_path)?;

        // Verify the branch was created and checked out
        let current_branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_branch = String::from_utf8_lossy(&current_branch_output).trim().to_string();
        assert_eq!(current_branch, created_branch_name);

        Ok(())
    }

    #[test]
    fn test_run_git_checkout_b_with_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let current_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_head = String::from_utf8_lossy(&current_head_output).trim().to_string();
        let parent_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output().unwrap().stdout;
        let parent_head = String::from_utf8_lossy(&parent_head_output).trim().to_string();

        let weeble = crate::weeble::weeble().unwrap_or(0.0).to_string();
        let blockheight = crate::blockheight::blockheight().unwrap_or(0.0).to_string();
        let wobble = crate::wobble::wobble().unwrap_or(0.0).to_string();
        let suffix = "feature";
        let expected_branch_name = format!("{}/{}/{}/{}/{}-{}", weeble, blockheight, wobble, parent_head, current_head, suffix);

        std::env::set_current_dir(repo_path)?;
        let created_branch_name = run_git_checkout_b(suffix.to_string(), repo_path)?;

        // Verify the branch was created and checked out
        let current_branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_branch = String::from_utf8_lossy(&current_branch_output).trim().to_string();
        assert_eq!(current_branch, created_branch_name);

        Ok(())
    }

    #[test]
    fn test_run_git_checkout_pr_no_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let current_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_head = String::from_utf8_lossy(&current_head_output).trim().to_string();
        let parent_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output().unwrap().stdout;
        let parent_head = String::from_utf8_lossy(&parent_head_output).trim().to_string();

        let weeble = crate::weeble::weeble().unwrap_or(0.0).to_string();
        let blockheight = crate::blockheight::blockheight().unwrap_or(0.0).to_string();
        let wobble = crate::wobble::wobble().unwrap_or(0.0).to_string();
        let expected_branch_name = format!("pr/{}/{}/{}/{}/{}", weeble, blockheight, wobble, parent_head, current_head);

        std::env::set_current_dir(repo_path)?;
        let created_branch_name = run_git_checkout_pr(String::from(""), repo_path)?;

        // Verify the branch was created and checked out
        let current_branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_branch = String::from_utf8_lossy(&current_branch_output).trim().to_string();
        assert_eq!(current_branch, created_branch_name);

        Ok(())
    }

    #[test]
    fn test_run_git_checkout_pr_with_arg() -> Result<()> {
        let dir = setup_test_repo();
        let repo_path = dir.path();

        let current_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_head = String::from_utf8_lossy(&current_head_output).trim().to_string();
        let parent_head_output = Command::new("git").arg("rev-parse").arg("--short").arg("HEAD^1").current_dir(repo_path).output().unwrap().stdout;
        let parent_head = String::from_utf8_lossy(&parent_head_output).trim().to_string();

        let weeble = crate::weeble::weeble().unwrap_or(0.0).to_string();
        let blockheight = crate::blockheight::blockheight().unwrap_or(0.0).to_string();
        let wobble = crate::wobble::wobble().unwrap_or(0.0).to_string();
        let suffix = "fix";
        let expected_branch_name = format!("pr/{}/{}/{}/{}/{}-{}", weeble, blockheight, wobble, parent_head, current_head, suffix);

        std::env::set_current_dir(repo_path)?;
        let created_branch_name = run_git_checkout_pr(suffix.to_string(), repo_path)?;

        // Verify the branch was created and checked out
        let current_branch_output = Command::new("git").arg("rev-parse").arg("--abbrev-ref").arg("HEAD").current_dir(repo_path).output().unwrap().stdout;
        let current_branch = String::from_utf8_lossy(&current_branch_output).trim().to_string();
        assert_eq!(current_branch, created_branch_name);

        Ok(())
    }
}


