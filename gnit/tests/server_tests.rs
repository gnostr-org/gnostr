use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_commit_page_does_not_panic() {
    let db_dir = tempdir().unwrap();
    let db_path = db_dir.path().to_str().unwrap();

    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let binary_path = format!("{}/target/debug/gnostr-gnit", crate_dir);
    
    let mut child = tokio::process::Command::new(binary_path)
        .arg("--scan-path")
        .arg(format!("{}/tests/resources", crate_dir))
        .arg("--db-store")
        .arg(db_path)
        .spawn()
        .expect("failed to spawn server");

    // Poll the server until it's ready
    let client = reqwest::Client::new();
    for _ in 0..30 {
        if client.get("http://localhost:3333").send().await.is_ok() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let client = reqwest::Client::new();
    let res = client
        .get("http://localhost:3333/crlf.git/commit")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    child.kill().await.expect("failed to kill server");
}

