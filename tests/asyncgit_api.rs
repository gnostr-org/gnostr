
use gnostr_asyncgit::sync::{
    self, create_branch, get_commit_details, get_head, get_stashes,
    stage_add_file, stash_save, checkout_branch, get_head_tuple, RepoPath,
};
use gnostr_asyncgit::sync::status::{get_status, StatusItemType};
use git2::{Repository, Signature};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

// Helper function to set up a temporary git repository for testing.
fn setup_test_repo() -> (TempDir, RepoPath) {
    let tmp_dir = TempDir::new().unwrap();
    let repo_path = tmp_dir.path().to_path_buf();
    let repo = Repository::init(&repo_path).unwrap();

    // Configure user name and email
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();

    // Create an initial commit
    let signature = Signature::now("Test User", "test@example.com").unwrap();
    let tree_id = {
        let mut index = repo.index().unwrap();
        let file_path = repo_path.join("README.md");
        File::create(&file_path)
            .unwrap()
            .write_all(b"Initial commit")
            .unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        let oid = index.write_tree().unwrap();
        repo.find_tree(oid).unwrap().id()
    };
    let tree = repo.find_tree(tree_id).unwrap();
    let _commit_id = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
    )
    .unwrap();

    (tmp_dir, RepoPath::Path(repo_path))
}

#[test]
fn test_get_head_commit() {
    let (_tmp_dir, repo_path) = setup_test_repo();

    let head_id = get_head(&repo_path).unwrap();
    let commit_details = get_commit_details(&repo_path, head_id).unwrap();

    assert_eq!(commit_details.message.unwrap().subject, "Initial commit");
}

#[test]
fn test_complex_git_workflow() {
    let (_tmp_dir, repo_path) = setup_test_repo();
    let repo_path_str = repo_path.gitpath().unwrap();

    // 1. Create a new branch
    create_branch(&repo_path, "feature-branch").unwrap();
    checkout_branch(&repo_path, "refs/heads/feature-branch").unwrap();

    // 2. Create a new file and stage it
    let file_path = repo_path_str.join("test.txt");
    fs::write(&file_path, "hello world").unwrap();
    stage_add_file(&repo_path, file_path.as_path()).unwrap();

    // 3. Verify the file is staged
    let status = get_status(&repo_path, Default::default()).unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path, "test.txt");
    assert_eq!(status[0].status, StatusItemType::New);

    // 4. Stash the changes
    let stash_hash = stash_save(&repo_path, Some("test stash"), false, false).unwrap();
    assert!(stash_hash.is_some());

    // 5. Verify the stash and that the working directory is clean
    let stashes = get_stashes(&repo_path).unwrap();
    assert_eq!(stashes.len(), 1);
    assert_eq!(stashes[0].message, "On feature-branch: test stash");

    let status_after_stash = get_status(&repo_path, Default::default()).unwrap();
    assert!(status_after_stash.is_empty());

    // 6. Check out the main branch again
    checkout_branch(&repo_path, "refs/heads/main").unwrap();

    // 7. Confirm that the HEAD is pointing to main
    let head = get_head_tuple(&repo_path).unwrap();
    assert_eq!(head.name, "refs/heads/main");
}

