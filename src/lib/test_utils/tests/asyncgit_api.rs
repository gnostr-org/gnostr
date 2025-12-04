
use gnostr_asyncgit::sync::{
    self, create_branch, get_commit_details, get_head,
    stage_add_file, checkout_branch, get_head_tuple, RepoPath,
};
use gnostr_asyncgit::sync::status::{get_status, StatusItemType, StatusType};
use git2::{Repository, Signature};
use serial_test::serial;
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

    // Set the initial branch to main
    let head = repo.head().unwrap();
    repo.branch("main", &repo.find_commit(head.target().unwrap()).unwrap(), true).unwrap();
    repo.set_head("refs/heads/main").unwrap();

    (tmp_dir, RepoPath::Path(repo_path))
}

#[test]
#[serial]
fn test_get_head_commit() {
    let (_tmp_dir, repo_path) = setup_test_repo();

    let head_id = get_head(&repo_path).unwrap();
    let commit_details = get_commit_details(&repo_path, head_id).unwrap();

    assert_eq!(commit_details.message.unwrap().subject, "Initial commit");
}

#[test]
#[serial]
fn test_complex_git_workflow() {
    let (_tmp_dir, repo_path) = setup_test_repo();
    let repo_path_str = repo_path.gitpath();

    // 1. Create a new branch
    create_branch(&repo_path, "feature-branch").unwrap();
    checkout_branch(&repo_path, "feature-branch", true).unwrap();

    // 2. Create a new file and stage it
    let file_path = repo_path_str.join("test.txt");
    fs::write(&file_path, "hello world").unwrap();
    stage_add_file(&repo_path, Path::new("test.txt")).unwrap();

    // 3. Verify the file is staged
    let status = get_status(&repo_path, StatusType::Both, None).unwrap();
    assert_eq!(status.len(), 1);
    assert_eq!(status[0].path, "test.txt");
    assert_eq!(status[0].status, StatusItemType::New);


    //INSERT gnostr legit commit creation here
    let mut cmd = std::process::Command::new("gnostr");
    cmd.arg("legit")
        .arg("-m")
        .arg("feat: add test file")
        .env("GIT_DIR", repo_path.gitpath().join(".git"))
        .env("GIT_WORK_TREE", repo_path.gitpath())
        .current_dir(repo_path.gitpath());
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // we will fix stashing and popping later


    // 4. Unstage the file and then stash the changes
    sync::reset_stage(&repo_path, "test.txt").unwrap();
    //let stash_result = stash_save(&repo_path, Some("test stash"), false, false);
    //assert!(stash_result.is_ok());

    //// 5. Verify the stash and that the working directory is clean
    //let stashes = get_stashes(&repo_path).unwrap();
    //assert_eq!(stashes.len(), 1);
    //let stash_commit_details = get_commit_details(&repo_path, stashes[0]).unwrap();
    //assert_eq!(
    //    stash_commit_details.message.unwrap().subject,
    //    "On feature-branch: test stash"
    //);

    //let status_after_stash = get_status(&repo_path, StatusType::Both, None).unwrap();
    //assert!(status_after_stash.is_empty());

    // 6. Check out the main branch again
    checkout_branch(&repo_path, "main", true).unwrap();

    // 7. Pop the stash
    //sync::stash_pop(&repo_path, stashes[0]).unwrap();
    // TODO: verify popped content

    // 8. Confirm that the HEAD is pointing to main
    let head = get_head_tuple(&repo_path).unwrap();
    assert_eq!(head.name, "refs/heads/main");
}

