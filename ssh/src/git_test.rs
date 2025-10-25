#[cfg(test)]
mod tests {
    use crate::git::Repo;
    use anyhow::Context;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_git_operations() -> anyhow::Result<()> {
        // 1. Setup: Create a bare server repository
        let server_dir = tempdir().context("Failed to create server tempdir")?;
        let server_repo_path = server_dir.path().join("server.git");
        Repo::create_bare(&server_repo_path)
            .await
            .context("Failed to create bare server repo")?;

        // 2. Client 1: Clone, commit, and push
        let client1_dir = tempdir().context("Failed to create client1 tempdir")?;
        let client1_repo_path = client1_dir.path().join("client1");
        Repo::clone(&server_repo_path, &client1_repo_path)
            .await
            .context("Failed to clone server repo to client1")?;

        let mut file1 = File::create(client1_repo_path.join("file1.txt"))
            .context("Failed to create file1.txt in client1")?;
        writeln!(file1, "Hello from client 1").context("Failed to write to file1.txt")?;
        drop(file1);

        let client1_repo = Repo::new(&client1_repo_path)
            .await
            .context("Failed to open client1 repo")?;
        client1_repo
            .add_and_commit("file1.txt", "Add file1 from client1")
            .await
            .context("Failed to commit file1 in client1")?;
        client1_repo
            .push_changes("main")
            .await
            .context("Failed to push from client1")?;

        // 3. Client 2: Clone, commit, and push
        let client2_dir = tempdir().context("Failed to create client2 tempdir")?;
        let client2_repo_path = client2_dir.path().join("client2");
        Repo::clone(&server_repo_path, &client2_repo_path)
            .await
            .context("Failed to clone server repo to client2")?;

        let mut file2 = File::create(client2_repo_path.join("file2.txt"))
            .context("Failed to create file2.txt in client2")?;
        writeln!(file2, "Hello from client 2").context("Failed to write to file2.txt")?;
        drop(file2);

        let client2_repo = Repo::new(&client2_repo_path)
            .await
            .context("Failed to open client2 repo")?;
        client2_repo
            .add_and_commit("file2.txt", "Add file2 from client2")
            .await
            .context("Failed to commit file2 in client2")?;
        client2_repo
            .push_changes("main")
            .await
            .context("Failed to push from client2")?;

        // 4. Client 1: Fetch and verify changes from Client 2
        client1_repo
            .fetch_changes("origin", "main")
            .await
            .context("Failed to fetch changes in client1")?;
        client1_repo
            .reset_hard("origin/main")
            .await
            .context("Failed to reset client1 to origin/main")?;

        // Verify file2.txt exists in client1
        assert!(client1_repo_path.join("file2.txt").exists());
        let content = fs::read_to_string(client1_repo_path.join("file2.txt"))
            .context("Failed to read file2.txt in client1")?;
        assert_eq!(content.trim(), "Hello from client 2");

        Ok(())
    }
}
