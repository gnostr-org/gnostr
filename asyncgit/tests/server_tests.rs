use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
#[ignore]
async fn test_commit_page_does_not_panic() {
    let db_dir = tempdir().unwrap();
    let db_path = db_dir.path().to_str().unwrap();

    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let binary_path = format!("{}/target/debug/gnostr-gnit", crate_dir);

    // Find available port starting from 3333
    let port = crate::find_available_port().await;
    println!("port={}", &port);
    let mut child = tokio::process::Command::new(binary_path)
        .arg("--scan-path")
        .arg(format!("{}/tests/resources", crate_dir))
        .arg("--db-store")
        .arg(db_path)
        .arg("--bind-port")
        .arg(port.to_string())
        .current_dir(crate_dir)
        .env("RUST_BACKTRACE", "1") // Enable backtrace for more info
        .env("PATH", std::env::var("PATH").unwrap_or_default()) // Pass host PATH to child
        .spawn()
        .expect("failed to spawn server");

    // Poll the server until it's ready
    let client = reqwest::Client::new();
    for poll_count in 0..30 {
        println!("poll_count={}", poll_count);
        if client
            .get(&format!("http://localhost:{}", port)) // no slash
            .send()
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    //test forwarding
    let res = client
        .get(&format!("http://localhost:{}/", port)) //slash
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    let res = client
        .get(&format!("http://localhost:{}/test/about", port)) //TODO reconfigure in asyncgit context
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    let res = client
        .get(&format!("http://localhost:{}/test/summary", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    let res = client
        .get(&format!("http://localhost:{}/test/refs", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    let res = client
        .get(&format!("http://localhost:{}/test/log", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    let res = client
        .get(&format!("http://localhost:{}/test/tree", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    println!("res.status().is_success()={}", res.status().is_success());
    let res = client
        .get(&format!("http://localhost:{}/test/diff", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    // Test patch endpoint with specific commit ID
    let res = client
        .get(&format!(
            "http://localhost:{}/test/patch?id=00075f5cf3b95f42baba2b355b8a3197f949e297",
            port
        ))
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

    // end test forwarding

    let res = client
        .get(&format!("http://localhost:{}/crlf.git/commit", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());
    println!("res.status().is_success()={}", res.status().is_success());
    let res = client
        .get(&format!("http://localhost:{}/test/tree", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(res.status().is_success());

    // Test patch endpoint with specific commit ID
    let res = client
        .get(&format!(
            "http://localhost:{}/crlf.git/patch?id=00075f5cf3b95f42baba2b355b8a3197f949e297",
            port
        ))
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

    //TODO mote tests

    // Test error case with invalid commit ID
    let res_invalid = client
        .get(&format!(
            "http://localhost:{}/crlf.git/patch?id=invalidcommitid",
            port
        ))
        .send()
        .await
        .expect("Failed to send request");

    assert_ne!(res_invalid.status(), reqwest::StatusCode::NOT_FOUND);

    child.kill().await.expect("failed to kill server");
}

// Helper function to find available port
async fn find_available_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};
    use tokio::net::TcpListener;

    static PORT_COUNTER: AtomicU16 = AtomicU16::new(3333);

    for port_offset in 0..100 {
        let test_port = PORT_COUNTER.fetch_add(1, Ordering::Relaxed) + port_offset;
        if let Ok(_) = TcpListener::bind(&format!("127.0.0.1:{}", test_port)).await {
            return test_port;
        }
    }

    3333 // fallback if no ports available
}
