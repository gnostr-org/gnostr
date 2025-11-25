
#[cfg(test)]
mod tests {
    use gnostr_asyncgit::sync::{self, RepoPath};
    use git2::{Repository, Signature};
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    // Helper function to set up a temporary git repository for testing.
    fn setup_test_repo() -> (TempDir, RepoPath) {
        let tmp_dir = TempDir::new().unwrap();
        let repo_path = tmp_dir.path();
        let repo = Repository::init(repo_path).unwrap();

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
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();

        (tmp_dir, RepoPath::Path(repo_path.to_path_buf()))
    }

    #[test]
    fn test_get_head_commit() {
        let (_tmp_dir, repo_path) = setup_test_repo();

        let head_commit = sync::get_head_commit(&repo_path);
        assert!(head_commit.is_ok());

        let commit = head_commit.unwrap();
        assert_eq!(commit.message.subject, "Initial commit");
    }
}
