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
    let res = client
        .get("http://localhost:3333/test/tree")
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    child.kill().await.expect("failed to kill server");
}

#[tokio::test]
async fn test_patch_endpoint_with_id() {
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

    // Test patch endpoint with specific commit ID
    let res = client
        .get("http://localhost:3333/crlf.git/patch?id=00075f5cf3b95f42baba2b355b8a3197f949e297")
        .send()
        .await
        .expect("Failed to send request");

    // Verify patch endpoint works
    assert!(res.status().is_success());
    assert_eq!(res.headers().get("content-type").unwrap(), "text/plain");

    // Verify it's a valid git patch format
    let patch_content = res.text().await.expect("Failed to read patch content");
    assert!(patch_content.contains("From 00075f5cf3b95f42baba2b355b8a3197f949e297"));
    assert!(patch_content.contains("Subject: [PATCH] 1896/932470/576827"));

    // Test error case with invalid commit ID
    let res_invalid = client
        .get("http://localhost:3333/crlf.git/patch?id=invalidcommitid")
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(res_invalid.status(), reqwest::StatusCode::NOT_FOUND);

    child.kill().await.expect("failed to kill server");
}
