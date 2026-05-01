use std::cell::Ref;

pub use git2_hooks::PrepareCommitMsgSource;
use scopetime::scope_time;

use super::{repository::repo, RepoPath};
use crate::error::{Error, Result};
//use crate::sync::utils;

///
#[derive(Debug, PartialEq, Eq)]
pub enum HookResult {
    /// Everything went fine
    Ok,
    /// Hook returned error
    NotOk(String),
}

///
#[derive(Debug, PartialEq, Eq)]
pub enum PrePushTarget<'a> {
    /// Push a single branch.
    Branch {
        /// Local branch name being pushed.
        branch: &'a str,
        /// Whether this is a delete push.
        delete: bool,
    },
    /// Push tags.
    Tags,
}

impl From<git2_hooks::HookResult> for HookResult {
    fn from(v: git2_hooks::HookResult) -> Self {
        match v {
            git2_hooks::HookResult::NoHookFound => Self::Ok,
            git2_hooks::HookResult::Run(response) if response.is_successful() => Self::Ok,
            git2_hooks::HookResult::Run(response) => {
                Self::NotOk(format!("{}{}", response.stdout, response.stderr))
            }
        }
    }
}

impl From<crate::upstream_sync::HookResult> for HookResult {
    fn from(v: crate::upstream_sync::HookResult) -> Self {
        match v {
            crate::upstream_sync::HookResult::Ok => Self::Ok,
            crate::upstream_sync::HookResult::NotOk(msg) => Self::NotOk(msg),
        }
    }
}

fn to_upstream_repo_path(repo_path: &RepoPath) -> crate::upstream_sync::RepoPath {
    match repo_path {
        RepoPath::Path(path) => crate::upstream_sync::RepoPath::Path(path.clone()),
        RepoPath::Workdir { gitdir, workdir } => {
            crate::upstream_sync::RepoPath::Workdir {
                gitdir: gitdir.clone(),
                workdir: workdir.clone(),
            }
        }
    }
}

fn to_upstream_target<'a>(
    push: &'a PrePushTarget<'a>,
) -> crate::upstream_sync::PrePushTarget<'a> {
    match push {
        PrePushTarget::Branch { branch, delete } => {
            crate::upstream_sync::PrePushTarget::Branch {
                branch,
                delete: *delete,
            }
        }
        PrePushTarget::Tags => crate::upstream_sync::PrePushTarget::Tags,
    }
}

///
pub fn hooks_pre_push(
    repo_path: &Ref<'_, RepoPath>,
    remote: &str,
    push: &PrePushTarget<'_>,
    basic_credential: Option<crate::sync::cred::BasicAuthCredential>,
) -> Result<HookResult> {
    scope_time!("hooks_pre_push");

    let repo_path = &**repo_path;
    let upstream_repo_path = to_upstream_repo_path(repo_path);
    let upstream_cred = basic_credential.map(|cred| {
        crate::upstream_sync::cred::BasicAuthCredential::new(
            cred.username,
            cred.password,
        )
    });

    let result = crate::upstream_sync::hooks_pre_push(
        &upstream_repo_path,
        remote,
        &to_upstream_target(push),
        upstream_cred,
    )
    .map_err(|e| Error::Generic(e.to_string()))?;

    Ok(result.into())
}

/// this hook is documented here <https://git-scm.com/docs/githooks#_commit_msg>
/// we use the same convention as other git clients to create a temp
/// file containing the commit message at
/// `<.git|hooksPath>/COMMIT_EDITMSG` and pass it's relative path as
/// the only parameter to the hook script.
pub fn hooks_commit_msg(repo_path: &RepoPath, msg: &mut String) -> Result<HookResult> {
    scope_time!("hooks_commit_msg");

    let repo = repo(repo_path)?;

    Ok(git2_hooks::hooks_commit_msg(&repo, None, msg)?.into())
}

/// this hook is documented here <https://git-scm.com/docs/githooks#_pre_commit>
pub fn hooks_pre_commit(repo_path: &RepoPath) -> Result<HookResult> {
    scope_time!("hooks_pre_commit");

    let repo = repo(repo_path)?;

    Ok(git2_hooks::hooks_pre_commit(&repo, None)?.into())
}

///
pub fn hooks_post_commit(repo_path: &RepoPath) -> Result<HookResult> {
    scope_time!("hooks_post_commit");

    let repo = repo(repo_path)?;

    Ok(git2_hooks::hooks_post_commit(&repo, None)?.into())
}

///
pub fn hooks_prepare_commit_msg(
    repo_path: &RepoPath,
    source: PrepareCommitMsgSource,
    msg: &mut String,
) -> Result<HookResult> {
    scope_time!("hooks_prepare_commit_msg");

    let repo = repo(repo_path)?;

    Ok(git2_hooks::hooks_prepare_commit_msg(&repo, None, source, msg)?.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::tests::repo_init;

    #[test]
    fn test_post_commit_hook_reject_in_subfolder() {
        let (_td, repo) = repo_init().unwrap();
        let root = repo.path().parent().unwrap();

        let hook = b"#!/bin/sh
	echo 'rejected'
	exit 1
	        ";

        git2_hooks::create_hook(&repo, git2_hooks::HOOK_POST_COMMIT, hook);

        let subfolder = root.join("foo/");
        std::fs::create_dir_all(&subfolder).unwrap();

        let res = hooks_post_commit(&subfolder.to_str().unwrap().into()).unwrap();

        assert_eq!(res, HookResult::NotOk(String::from("rejected\n")));
    }

    // make sure we run the hooks with the correct pwd.
    // for non-bare repos this is the dir of the worktree
    // unfortunately does not work on windows
    #[test]
    #[cfg(unix)]
    fn test_pre_commit_workdir() {
        let (_td, repo) = repo_init().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();
        let workdir = crate::sync::utils::repo_work_dir(repo_path).unwrap();

        let hook = b"#!/bin/sh
	echo $(pwd)
	exit 1
	        ";

        git2_hooks::create_hook(&repo, git2_hooks::HOOK_PRE_COMMIT, hook);
        let res = hooks_pre_commit(repo_path).unwrap();
        if let HookResult::NotOk(res) = res {
            assert_eq!(
                std::path::Path::new(res.trim_end()),
                std::path::Path::new(&workdir)
            );
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_hooks_commit_msg_reject_in_subfolder() {
        let (_td, repo) = repo_init().unwrap();
        let root = repo.path().parent().unwrap();

        let hook = b"#!/bin/sh
	echo 'msg' > $1
	echo 'rejected'
	exit 1
	        ";

        git2_hooks::create_hook(&repo, git2_hooks::HOOK_COMMIT_MSG, hook);

        let subfolder = root.join("foo/");
        std::fs::create_dir_all(&subfolder).unwrap();

        let mut msg = String::from("test");
        let res = hooks_commit_msg(&subfolder.to_str().unwrap().into(), &mut msg).unwrap();

        assert_eq!(res, HookResult::NotOk(String::from("rejected\n")));

        assert_eq!(msg, String::from("msg\n"));
    }
}
