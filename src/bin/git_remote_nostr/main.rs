#![cfg_attr(not(test), warn(clippy::pedantic))]
#![allow(clippy::large_futures, clippy::module_name_repetitions)]
// better solution to dead_code error on multiple binaries than https://stackoverflow.com/a/66196291
#![allow(dead_code)]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use core::str;
use std::{
    collections::HashSet,
    env, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{bail, Context, Result};
use client::{consolidate_fetch_reports, get_repo_ref_from_cache, Connect};
use git::{nostr_url::NostrUrlDecoded, RepoActions};
use gnostr::{client, git};
use nostr_0_34_1::nips::nip01::Coordinate;
use utils::read_line;

use crate::{client::Client, git::Repo};

mod fetch;
mod list;
mod push;
mod utils;

use tempfile::Builder;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let args = env::args();
    let args = args.skip(1).take(2).collect::<Vec<_>>();

    let ([_, nostr_remote_url] | [nostr_remote_url]) = args.as_slice() else {
        bail!("invalid arguments - no url");
    };
    if env::args().nth(1).as_deref() == Some("--version") {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("v{VERSION}");
        return Ok(());
    }

    let git_repo = Repo::from_path(&PathBuf::from(
        std::env::var("GIT_DIR").context("git should set GIT_DIR when remote helper is called")?,
    ))?;
    let git_repo_path = git_repo.get_path()?;

    let client = Client::default();

    let decoded_nostr_url =
        NostrUrlDecoded::from_str(nostr_remote_url).context("invalid nostr url")?;

    fetching_with_report_for_helper(git_repo_path, &client, &decoded_nostr_url.coordinates).await?;

    let repo_ref = get_repo_ref_from_cache(git_repo_path, &decoded_nostr_url.coordinates).await?;

    let stdin = io::stdin();
    let mut line = String::new();

    let mut list_outputs = None;
    loop {
        let tokens = read_line(&stdin, &mut line)?;

        match tokens.as_slice() {
            ["capabilities"] => {
                println!("option");
                println!("push");
                println!("fetch");
                println!();
            }
            ["option", "verbosity"] => {
                println!("ok");
            }
            ["option", ..] => {
                println!("unsupported");
            }
            ["fetch", oid, refstr] => {
                fetch::run_fetch(
                    &git_repo,
                    &repo_ref,
                    &decoded_nostr_url,
                    &stdin,
                    oid,
                    refstr,
                )
                .await?;
            }
            ["push", refspec] => {
                push::run_push(
                    &git_repo,
                    &repo_ref,
                    &decoded_nostr_url,
                    &stdin,
                    refspec,
                    &client,
                    list_outputs.clone(),
                )
                .await?;
            }
            ["list"] => {
                list_outputs =
                    Some(list::run_list(&git_repo, &repo_ref, &decoded_nostr_url, false).await?);
            }
            ["list", "for-push"] => {
                list_outputs =
                    Some(list::run_list(&git_repo, &repo_ref, &decoded_nostr_url, true).await?);
            }
            [] => {
                return Ok(());
            }
            _ => {
                bail!(format!("unknown command: {}", line.trim().to_owned()));
            }
        }
    }
}

async fn fetching_with_report_for_helper(
    git_repo_path: &Path,
    client: &Client,
    repo_coordinates: &HashSet<Coordinate>,
) -> Result<()> {
    let term = console::Term::stderr();
    term.write_line("nostr: fetching...")?;
    let (relay_reports, progress_reporter) = client
        .fetch_all(git_repo_path, repo_coordinates, &HashSet::new())
        .await?;
    if !relay_reports.iter().any(std::result::Result::is_err) {
        let _ = progress_reporter.clear();
        term.clear_last_lines(1)?;
    }
    let report = consolidate_fetch_reports(relay_reports);
    if report.to_string().is_empty() {
        term.write_line("nostr: no updates")?;
    } else {
        term.write_line(&format!("nostr updates: {report}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        io::{self, Write},
        path::PathBuf,
        process::Stdio,
    };
    use tempfile::Builder;
    use tokio::process::Command;

    // Helper to get the path to the built binary.
    // In a real scenario, this might involve building the binary first.
    // For this example, we'll assume 'git_remote_nostr' is in the PATH or current dir.
    fn get_binary_path() -> PathBuf {
        // This is a placeholder. In a real test setup, you'd ensure the binary is built and accessible.
        // For example, you might run `cargo build --bin git-remote-nostr` and then use its path.
        // For now, let's assume it's in the PATH.
        PathBuf::from("git_remote_nostr")
    }

    #[tokio::test]
    async fn test_git_remote_nostr_capabilities() -> Result<()> {
        let temp_dir = Builder::new().prefix("gnostr-test").tempdir()?;
        let repo_path = temp_dir.path();

        // Initialize a Git repository
        let mut init_cmd = Command::new("git");
        init_cmd.current_dir(repo_path);
        init_cmd.arg("init");
        init_cmd.stdout(Stdio::piped());
        init_cmd.stderr(Stdio::piped());
        let init_output = init_cmd.output().await?;
        assert!(init_output.status.success(), "git init failed: {:?}", init_output);

        // Set GIT_DIR environment variable to the temporary repo path
        let original_git_dir = env::var("GIT_DIR").ok();
        env::set_var("GIT_DIR", repo_path);

        let nostr_url = "nostr://npub1ahaz04ya9tehace3uy39hdhdryfvdkve9qdndkqp3tvehs6h8s5slq45hy/nostr.cro.social/gnostr";

        // Prepare stdin for capabilities command
        let mut child = Command::new(get_binary_path())
            .arg(nostr_url)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        tokio::task::spawn(async move {
            // Send capabilities command and an empty line to signal end of input
            let mut buffer = io::Cursor::new(b"capabilities\n\n"); // Escaped for string literal
            tokio::io::copy(&mut buffer, &mut stdin).await.unwrap();
        });

        let output = child.wait_with_output().await?;

        // Restore original GIT_DIR environment variable
        if let Some(original) = original_git_dir {
            env::set_var("GIT_DIR", original);
        } else {
            env::remove_var("GIT_DIR");
        }
        // temp_dir is dropped automatically when it goes out of scope, cleaning up the directory.

        assert!(output.status.success(), "git_remote_nostr failed: stdout='{}', stderr='{}'", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));

        let stdout = String::from_utf8(output.stdout)?;
        let stderr = String::from_utf8(output.stderr)?;

        println!("Stdout:\n{}", stdout);
        println!("Stderr:\n{}", stderr);

        // Check for expected output from capabilities command
        assert!(stdout.contains("option"));
        assert!(stdout.contains("push"));
        assert!(stdout.contains("fetch"));

        Ok(())
    }
}