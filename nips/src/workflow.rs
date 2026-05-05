use std::{
    env,
    error::Error,
    ffi::OsStr,
    path::{Path, PathBuf},
    process::Command,
};

use gnostr_asyncgit::{
    filehash::{get_relay_urls, publish_patch_event},
    tui::{
        git::{self, cli::Args as GitTuiArgs},
        shared::term::{backend as asyncgit_backend, Term},
    },
    sync::{commit::commit, stage_add_file, RepoPath},
    types::Keys,
};

fn repo_path(repo_dir: &Path) -> Result<RepoPath, Box<dyn Error>> {
    let repo_dir = repo_dir
        .to_str()
        .ok_or_else(|| std::io::Error::other("invalid repository path"))?;
    Ok(repo_dir.into())
}

pub fn launch_editor(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let args: Vec<&str> = parts.collect();

    let status = Command::new(program)
        .args(args)
        .arg(file_path)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "editor exited with status {status}"
        ))
        .into())
    }
}

pub fn launch_git_tui(repo_dir: &Path) -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir()?;
    env::set_current_dir(repo_dir)?;

    let result = (|| -> Result<(), Box<dyn Error>> {
        let mut term = Term::new(asyncgit_backend())?;
        let args = GitTuiArgs::default();
        git::run(&args, &mut term)?;
        Ok(())
    })();

    env::set_current_dir(current_dir)?;
    result
}

fn patch_content(repo_dir: &Path, commit_id: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args([
            "show",
            "--format=medium",
            "--patch",
            "--no-ext-diff",
            "--no-color",
            commit_id,
        ])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other("failed to generate patch content").into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn submit_proposal(repo_dir: &Path, file_path: &Path) -> Result<String, Box<dyn Error>> {
    let repo = repo_path(repo_dir)?;
    let rel_path = file_path.strip_prefix(repo_dir)?;
    stage_add_file(&repo, rel_path)?;

    let subject = rel_path
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("nips update");
    let commit_id = commit(&repo, &format!("update {subject}"))?;
    let commit_hash = commit_id.to_string();
    let keys = Keys::parse(format!("{:0>64}", commit_hash))
        .ok_or_else(|| std::io::Error::other("failed to derive keys from commit hash"))?;
    let relay_urls = get_relay_urls();
    let patch = patch_content(repo_dir, &commit_hash)?;
    let repo_name = repo_dir
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("nips")
        .to_string();

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        publish_patch_event(&keys, &relay_urls, &repo_name, &commit_hash, &patch, None).await
    })?;

    Ok(commit_hash)
}
