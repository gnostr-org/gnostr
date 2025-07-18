//! Git Api for Commits
//use anyhow::anyhow;
use git2::{message_prettify, Commit, ErrorCode, ObjectType, Oid, Repository, Signature};

use serde::{Deserialize, Serialize};
use serde_json;
//?use nostr_sdk::serde_json;
//use serde_json::{Result as SerdeJsonResult, Value};
use log::debug;
use scopetime::scope_time;

use super::{CommitId, RepoPath};
use crate::{
    error::{Error, Result},
    sync::{
        repository::repo,
        sign::{SignBuilder, SignError},
        utils::get_head_repo,
    },
};

///
pub fn amend(repo_path: &RepoPath, id: CommitId, msg: &str) -> Result<CommitId> {
    scope_time!("amend");

    let repo = repo(repo_path)?;
    let config = repo.config()?;

    let commit = repo.find_commit(id.into())?;

    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    if config.get_bool("commit.gpgsign").unwrap_or(false) {
        // HACK: we undo the last commit and create a new one
        use crate::sync::utils::undo_last_commit;

        let head = get_head_repo(&repo)?;
        if head == commit.id().into() {
            undo_last_commit(repo_path)?;
            return self::commit(repo_path, msg);
        }

        return Err(Error::SignAmendNonLastCommit);
    }

    let new_id = commit.amend(Some("HEAD"), None, None, None, Some(msg), Some(&tree))?;

    Ok(CommitId::new(new_id))
}

/// Wrap `Repository::signature` to allow unknown user.name.
///
/// See <https://github.com/extrawurst/gitui/issues/79>.
#[allow(clippy::redundant_pub_crate)]
pub(crate) fn signature_allow_undefined_name(
    repo: &Repository,
) -> std::result::Result<Signature<'_>, git2::Error> {
    let signature = repo.signature();

    if let Err(ref e) = signature {
        if e.code() == ErrorCode::NotFound {
            let config = repo.config()?;

            if let (Err(_), Ok(email_entry)) = (
                config.get_entry("user.name"),
                config.get_entry("user.email"),
            ) {
                if let Some(email) = email_entry.value() {
                    return Signature::now("unknown", email);
                }
            };
        }
    }

    signature
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SerializableCommit {
    id: String,
    tree: String,
    parents: Vec<String>,
    author_name: String,
    author_email: String,
    committer_name: String,
    committer_email: String,
    message: String,
    time: i64,
}
///
pub fn serialize_commit(commit: &Commit) -> Result<String> {
    let id = commit.id().to_string();
    let tree = commit.tree_id().to_string();
    let parents = commit.parent_ids().map(|oid| oid.to_string()).collect();
    let author = commit.author();
    let committer = commit.committer();
    let message = commit
        .message()
        .ok_or(log::debug!("No commit message"))
        .expect("")
        .to_string();
    log::debug!("message:\n{:?}", message);
    let time = commit.time().seconds();
    debug!("time: {:?}", time);

    let serializable_commit = SerializableCommit {
        id,
        tree,
        parents,
        author_name: author.name().unwrap_or_default().to_string(),
        author_email: author.email().unwrap_or_default().to_string(),
        committer_name: committer.name().unwrap_or_default().to_string(),
        committer_email: committer.email().unwrap_or_default().to_string(),
        message,
        time,
    };

    let serialized = serde_json::to_string(&serializable_commit).expect("");
    debug!("serialized_commit: {:?}", serialized);
    Ok(serialized)
}
///
pub fn deserialize_commit<'a>(repo: &'a Repository, data: &'a str) -> Result<Commit<'a>> {
    //we serialize the commit data
    //easier to grab the commit.id
    let serializable_commit: SerializableCommit = serde_json::from_str(data).expect("");
    //grab the commit.id
    let oid = Oid::from_str(&serializable_commit.id)?;
    //oid used to search the repo
    let commit_obj = repo.find_object(oid, Some(ObjectType::Commit))?;
    //grab the commit
    let commit = commit_obj.peel_to_commit()?;
    //confirm we grabbed the correct commit
    //if commit.id().to_string() != serializable_commit.id {
    //    return Err(eprintln!("Commit ID mismatch during deserialization"));
    //}
    //return the commit
    Ok(commit)
}

/// this does not run any git hooks, git-hooks have to be executed
/// manually, checkout `hooks_commit_msg` for example
pub fn commit(repo_path: &RepoPath, msg: &str) -> Result<CommitId> {
    scope_time!("commit");

    let repo = repo(repo_path)?;
    let config = repo.config()?;
    let signature = signature_allow_undefined_name(&repo)?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;

    let parents = if let Ok(id) = get_head_repo(&repo) {
        vec![repo.find_commit(id.into())?]
    } else {
        Vec::new()
    };

    let parents = parents.iter().collect::<Vec<_>>();

    let commit_id = if config.get_bool("commit.gpgsign").unwrap_or(false) {
        let buffer =
            repo.commit_create_buffer(&signature, &signature, msg, &tree, parents.as_slice())?;

        let commit = std::str::from_utf8(&buffer)
            .map_err(|_e| SignError::Shellout("utf8 conversion error".to_string()))?;

        let signer = SignBuilder::from_gitconfig(&repo, &config)?;
        let (signature, signature_field) = signer.sign(&buffer)?;
        let commit_id = repo.commit_signed(commit, &signature, signature_field.as_deref())?;

        // manually advance to the new commit ID
        // repo.commit does that on its own, repo.commit_signed does
        // not if there is no head, read default branch or default
        // to "master"
        if let Ok(mut head) = repo.head() {
            head.set_target(commit_id, msg)?;
        } else {
            let default_branch_name = config.get_str("init.defaultBranch").unwrap_or("master");
            repo.reference(
                &format!("refs/heads/{default_branch_name}"),
                commit_id,
                true,
                msg,
            )?;
        }

        commit_id
    } else {
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            msg,
            &tree,
            parents.as_slice(),
        )?
    };

    Ok(commit_id.into())
}

/// Tag a commit.
///
/// This function will return an `Err(…)` variant if the tag’s name is
/// refused by git or if the tag already exists.
pub fn tag_commit(
    repo_path: &RepoPath,
    commit_id: &CommitId,
    tag: &str,
    message: Option<&str>,
) -> Result<CommitId> {
    scope_time!("tag_commit");

    let repo = repo(repo_path)?;

    let object_id = commit_id.get_oid();
    let target = repo.find_object(object_id, Some(ObjectType::Commit))?;

    let c = if let Some(message) = message {
        let signature = signature_allow_undefined_name(&repo)?;
        repo.tag(tag, &target, &signature, message, false)?.into()
    } else {
        repo.tag_lightweight(tag, &target, false)?.into()
    };

    Ok(c)
}

/// Loads the comment prefix from config & uses it to prettify commit
/// messages
pub fn commit_message_prettify(repo_path: &RepoPath, message: String) -> Result<String> {
    let comment_char = repo(repo_path)?
        .config()?
        .get_string("core.commentChar")
        .ok()
        .and_then(|char_string| char_string.chars().next())
        .unwrap_or('#') as u8;

    Ok(message_prettify(message, Some(comment_char))?)
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Write, path::Path};

    use commit::{amend, commit_message_prettify, tag_commit};
    use git2::Repository;

    use crate::{
        error::Result,
        sync::{
            commit, get_commit_details, get_commit_files, stage_add_file,
            tags::{get_tags, Tag},
            tests::{get_statuses, repo_init, repo_init_empty},
            utils::get_head,
            LogWalker, RepoPath,
        },
    };

    fn count_commits(repo: &Repository, max: usize) -> usize {
        let mut items = Vec::new();
        let mut walk = LogWalker::new(repo, max).unwrap();
        walk.read(&mut items).unwrap();
        items.len()
    }

    #[test]
    fn test_commit() {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        File::create(root.join(file_path))
            .unwrap()
            .write_all(b"test\nfoo")
            .unwrap();

        assert_eq!(get_statuses(repo_path), (1, 0));

        stage_add_file(repo_path, file_path).unwrap();

        assert_eq!(get_statuses(repo_path), (0, 1));

        commit(repo_path, "commit msg").unwrap();

        assert_eq!(get_statuses(repo_path), (0, 0));
    }

    #[test]
    fn test_commit_in_empty_repo() {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init_empty().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        assert_eq!(get_statuses(repo_path), (0, 0));

        File::create(root.join(file_path))
            .unwrap()
            .write_all(b"test\nfoo")
            .unwrap();

        assert_eq!(get_statuses(repo_path), (1, 0));

        stage_add_file(repo_path, file_path).unwrap();

        assert_eq!(get_statuses(repo_path), (0, 1));

        commit(repo_path, "commit msg").unwrap();

        assert_eq!(get_statuses(repo_path), (0, 0));
    }

    #[test]
    fn test_amend() -> Result<()> {
        let file_path1 = Path::new("foo");
        let file_path2 = Path::new("foo2");
        let (_td, repo) = repo_init_empty()?;
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        File::create(root.join(file_path1))?.write_all(b"test1")?;

        stage_add_file(repo_path, file_path1)?;
        let id = commit(repo_path, "commit msg")?;

        assert_eq!(count_commits(&repo, 10), 1);

        File::create(root.join(file_path2))?.write_all(b"test2")?;

        stage_add_file(repo_path, file_path2)?;

        let new_id = amend(repo_path, id, "amended")?;

        assert_eq!(count_commits(&repo, 10), 1);

        let details = get_commit_details(repo_path, new_id)?;
        assert_eq!(details.message.unwrap().subject, "amended");

        let files = get_commit_files(repo_path, new_id, None)?;

        assert_eq!(files.len(), 2);

        let head = get_head(repo_path)?;

        assert_eq!(head, new_id);

        Ok(())
    }

    #[test]
    fn test_tag() -> Result<()> {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init_empty().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        File::create(root.join(file_path))?.write_all(b"test\nfoo")?;

        stage_add_file(repo_path, file_path)?;

        let new_id = commit(repo_path, "commit msg")?;

        tag_commit(repo_path, &new_id, "tag", None)?;

        assert_eq!(get_tags(repo_path).unwrap()[&new_id], vec![Tag::new("tag")]);

        assert!(matches!(
            tag_commit(repo_path, &new_id, "tag", None),
            Err(_)
        ));

        assert_eq!(get_tags(repo_path).unwrap()[&new_id], vec![Tag::new("tag")]);

        tag_commit(repo_path, &new_id, "second-tag", None)?;

        assert_eq!(
            get_tags(repo_path).unwrap()[&new_id],
            vec![Tag::new("second-tag"), Tag::new("tag")]
        );

        Ok(())
    }

    #[test]
    fn test_tag_with_message() -> Result<()> {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init_empty().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        File::create(root.join(file_path))?.write_all(b"test\nfoo")?;

        stage_add_file(repo_path, file_path)?;

        let new_id = commit(repo_path, "commit msg")?;

        tag_commit(repo_path, &new_id, "tag", Some("tag-message"))?;

        assert_eq!(
            get_tags(repo_path).unwrap()[&new_id][0]
                .annotation
                .as_ref()
                .unwrap(),
            "tag-message"
        );

        Ok(())
    }

    /// Beware: this test has to be run with a `$HOME/.gitconfig` that
    /// has `user.email` not set. Otherwise, git falls back to the
    /// value of `user.email` in `$HOME/.gitconfig` and this test
    /// fails.
    ///
    /// As of February 2021, `repo_init_empty` sets all git config
    /// locations to an empty temporary directory, so this constraint
    /// is met.
    #[test]
    fn test_empty_email() -> Result<()> {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init_empty().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        File::create(root.join(file_path))?.write_all(b"test\nfoo")?;

        stage_add_file(repo_path, file_path)?;

        repo.config()?.remove("user.email")?;

        let error = commit(repo_path, "commit msg");

        assert!(matches!(error, Err(_)));

        repo.config()?.set_str("user.email", "email")?;

        let success = commit(repo_path, "commit msg");

        assert!(matches!(success, Ok(_)));
        assert_eq!(count_commits(&repo, 10), 1);

        let details = get_commit_details(repo_path, success.unwrap()).unwrap();

        assert_eq!(details.author.name, "name");
        assert_eq!(details.author.email, "email");

        Ok(())
    }

    /// See comment to `test_empty_email`.
    #[test]
    fn test_empty_name() -> Result<()> {
        let file_path = Path::new("foo");
        let (_td, repo) = repo_init_empty().unwrap();
        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        File::create(root.join(file_path))?.write_all(b"test\nfoo")?;

        stage_add_file(repo_path, file_path)?;

        repo.config()?.remove("user.name")?;

        let mut success = commit(repo_path, "commit msg");

        assert!(matches!(success, Ok(_)));
        assert_eq!(count_commits(&repo, 10), 1);

        let mut details = get_commit_details(repo_path, success.unwrap()).unwrap();

        assert_eq!(details.author.name, "unknown");
        assert_eq!(details.author.email, "email");

        repo.config()?.set_str("user.name", "name")?;

        success = commit(repo_path, "commit msg");

        assert!(matches!(success, Ok(_)));
        assert_eq!(count_commits(&repo, 10), 2);

        details = get_commit_details(repo_path, success.unwrap()).unwrap();

        assert_eq!(details.author.name, "name");
        assert_eq!(details.author.email, "email");

        Ok(())
    }

    #[test]
    fn test_empty_comment_char() -> Result<()> {
        let (_td, repo) = repo_init_empty().unwrap();

        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        let message =
            commit_message_prettify(repo_path, "#This is a test message\nTest".to_owned())?;

        assert_eq!(message, "Test\n");
        Ok(())
    }

    #[test]
    fn test_with_comment_char() -> Result<()> {
        let (_td, repo) = repo_init_empty().unwrap();

        let root = repo.path().parent().unwrap();
        let repo_path: &RepoPath = &root.as_os_str().to_str().unwrap().into();

        repo.config()?.set_str("core.commentChar", ";")?;

        let message =
            commit_message_prettify(repo_path, ";This is a test message\nTest".to_owned())?;

        assert_eq!(message, "Test\n");

        Ok(())
    }
}
