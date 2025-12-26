#[cfg(test)]
mod tests {
    use crate::git::Repo;
    use anyhow::Context;
    use git2::{self, RepositoryInitOptions};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore]
    async fn test_git_operations() -> anyhow::Result<()> {
        // 1. Setup: Create a bare server repository
        let _server_dir = tempdir().context("Failed to create server tempdir")?;
        let mut initial_repo = crate::test_utils::git::GitTestRepo::new("main")?;
        File::create(initial_repo.dir.join("initial.txt"))
            .context("Failed to create initial.txt")?;
        initial_repo
            .stage_and_commit("Initial commit")
            .context("Failed to make initial commit")?;

        let server_repo = crate::test_utils::git::GitTestRepo::recreate_as_bare(&initial_repo)?;
        server_repo.git_repo.set_head("refs/heads/main")?;

        // 2. Client 1: Clone, commit, and push
        let client1_dir = tempdir().context("Failed to create client1 tempdir")?;
        let _client1_repo_path = client1_dir.path().join("client1");
        let mut client1_test_repo = crate::test_utils::git::GitTestRepo::duplicate(&server_repo)
            .context("Failed to clone server repo to client1")?;

        let mut file1 = File::create(client1_test_repo.dir.join("file1.txt"))
            .context("Failed to create file1.txt in client1")?;
        writeln!(file1, "Hello from client 1").context("Failed to write to file1.txt")?;
        drop(file1);

        client1_test_repo
            .stage_and_commit("Add file1 from client1")
            .context("Failed to commit file1 in client1")?;
        client1_test_repo
            .push_changes("origin", "main")
            .context("Failed to push from client1")?;

        // 3. Client 2: Clone, commit, and push
        let client2_dir = tempdir().context("Failed to create client2 tempdir")?;
        let _client2_repo_path = client2_dir.path().join("client2");
        let mut client2_test_repo = crate::test_utils::git::GitTestRepo::duplicate(&server_repo)
            .context("Failed to clone server repo to client2")?;

        let mut file2 = File::create(client2_test_repo.dir.join("file2.txt"))
            .context("Failed to create file2.txt in client2")?;
        writeln!(file2, "Hello from client 2").context("Failed to write to file2.txt")?;
        drop(file2);

        client2_test_repo
            .stage_and_commit("Add file2 from client2")
            .context("Failed to commit file2 in client2")?;
        client2_test_repo
            .push_changes("origin", "main")
            .context("Failed to push from client2")?;

        // 4. Client 1: Fetch and verify changes from Client 2
        client1_test_repo
            .fetch_changes("origin", "main")
            .context("Failed to fetch changes in client1")?;
        client1_test_repo
            .reset_hard("origin/main")
            .context("Failed to reset client1 to origin/main")?;

        // Verify file2.txt exists in client1
        assert!(client1_test_repo.dir.join("file2.txt").exists());
        let content = fs::read_to_string(client1_test_repo.dir.join("file2.txt"))
            .context("Failed to read file2.txt in client1")?;
        assert_eq!(content.trim(), "Hello from client 2");

        Ok(())
    }
}
