use gnostr_legit::gitminer::{Gitminer, Options};
use git2::{Repository, Signature, Oid};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;
use time::OffsetDateTime;

// Helper function to set up a temporary git repository for testing.
fn setup_test_repo() -> (TempDir, Repository) {
    let tmp_dir = TempDir::new().unwrap();
    let repo_path = tmp_dir.path();
    let repo = Repository::init(repo_path).unwrap();

    // Configure user name and email
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();
    config.set_str("gnostr.relays", "wss://relay.example.com").unwrap();

    // Create an initial commit
    {
        let signature = Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            // Create a dummy file to have a non-empty initial commit
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
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();
    }

    (tmp_dir, repo)
}

#[test]
fn test_gitminer_new_ok() {
    let (_tmp_dir, repo) = setup_test_repo();
    let opts = Options {
        threads: 1,
        target: "00".to_string(),
        message: "Test commit".to_string(),
        repo: repo.path().to_str().unwrap().to_string(),
        timestamp: OffsetDateTime::now_utc(),
    };

    let miner_result = Gitminer::new(opts);
    assert!(miner_result.is_ok());
    let miner = miner_result.unwrap();

    // `author` field is private, so we can't assert it directly.
    // We can assert the public `relays` field.
    // assert_eq!(miner.relays, "wss://relay.example.com");
}

#[test]
fn test_gitminer_new_fail_no_repo() {
    let tmp_dir = TempDir::new().unwrap();
    let repo_path = tmp_dir.path().join("non-existent-repo");
    let opts = Options {
        threads: 1,
        target: "00".to_string(),
        message: "Test commit".to_string(),
        repo: repo_path.to_str().unwrap().to_string(),
        timestamp: OffsetDateTime::now_utc(),
    };

    let miner_result = Gitminer::new(opts);
    assert!(miner_result.is_err());
    assert_eq!(miner_result.err(), Some("Failed to open repository"));
}


#[test]
fn test_mine_commit_success() {
    println!("Setting up test repository...");
    let (repo_path_str, repo) = setup_test_repo();
    let repo_path_str = repo.path().to_str().unwrap().to_string();
    println!("Test repository path: {}", repo_path_str);

    let opts = Options {
        threads: 1,
        target: "0".to_string(),
        message: "Mined commit".to_string(),
        repo: repo_path_str.clone(),
        timestamp: OffsetDateTime::now_utc(),
    };

    println!("Initializing Gitminer with options: {:?}", opts);
    let mut miner = Gitminer::new(opts).unwrap();
    println!("Mining commit...");
    let commit_hash_result = miner.mine();

    assert!(commit_hash_result.is_ok());
    let commit_hash = commit_hash_result.unwrap();
    println!("Mined commit hash: {}", commit_hash);

    assert!(commit_hash.starts_with("0"));
    println!("Verified commit hash starts with '0'.");

    // Verify the commit exists in the repo
    let oid = Oid::from_str(&commit_hash).unwrap();
    let commit = repo.find_commit(oid).unwrap();
    assert_eq!(commit.message().unwrap().lines().next().unwrap(), "Mined commit");
    println!("Verified commit message: '{}'", commit.message().unwrap().lines().next().unwrap());

    // Verify that .gnostr directories and files were created
    let repo_path = Path::new(&repo_path_str);
    println!("Verifying .gnostr directory existence...");
    assert!(repo_path.join(".gnostr").exists());
    println!("  .gnostr exists.");
    assert!(repo_path.join(".gnostr/legit/blobs").exists());
    println!("  .gnostr/legit/blobs exists.");
    assert!(repo_path.join(".gnostr/legit/reflog").exists());
    println!("  .gnostr/legit/reflog exists.");
    assert!(repo_path.join(".gnostr/legit/blobs").join(&commit_hash).exists());
    println!("  .gnostr/legit/blobs/{} exists.", commit_hash);
    assert!(repo_path.join(".gnostr/legit/reflog").join(&commit_hash).exists());
    println!("  .gnostr/legit/reflog/{} exists.", commit_hash);
    println!("All .gnostr directories and files verified successfully.");
}
