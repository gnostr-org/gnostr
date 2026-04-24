use clap::Parser;

/// Runs the Blossom server wrapper.
#[derive(Parser, Debug, Clone)]
#[command(about = "Run the Blossom server", long_about = None, disable_help_flag = true)]
pub struct ServerSubCommand {
    /// Show blossom-server help.
    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue)]
    pub help: bool,

    /// Pass-through arguments for blossom-server.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub blossom_args: Vec<String>,
}

/// Launch the Blossom server wrapper.
pub async fn server(args: &ServerSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    if args.help {
        crate::server::print_help()?;
        return Ok(());
    }

    crate::server::run_with_args(args.blossom_args.clone())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        net::SocketAddr,
        path::{Path, PathBuf},
        process::Command,
        sync::{Arc, Mutex},
    };

    use axum::{
        extract::{Path as AxumPath, State},
        http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
        response::IntoResponse,
        routing::{get, put},
        Json, Router,
    };
    use hex::encode;
    use serde::Serialize;
    use sha2::{Digest, Sha256};
    use tempfile::TempDir;
    use tokio::net::TcpListener;

    use gnostr_git_helpers::{
        blossom_backend::BlossomRemote,
        protocol::{PushSpec, RemoteHelper},
    };

    #[derive(Clone, Default)]
    struct TestServerState {
        pubkey: String,
        base_url: String,
        next_created: u64,
        blobs: HashMap<String, StoredBlob>,
    }

    #[derive(Clone)]
    struct StoredBlob {
        content_type: String,
        created: u64,
        body: Vec<u8>,
    }

    #[derive(Serialize)]
    struct UploadResponse {
        sha256: String,
        size: usize,
    }

    #[derive(Serialize)]
    struct BlobDescriptor {
        sha256: String,
        url: String,
        #[serde(rename = "type")]
        content_type: Option<String>,
        created: Option<u64>,
    }

    struct CurrentDirGuard {
        original: PathBuf,
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    fn set_current_dir(path: &Path) -> CurrentDirGuard {
        let original = std::env::current_dir().expect("current dir");
        std::env::set_current_dir(path).expect("set current dir");
        CurrentDirGuard { original }
    }

    fn git(dir: &Path, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("run git");
        assert!(status.success(), "git {:?} failed with {status}", args);
    }

    fn git_output(dir: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .output()
            .expect("run git");
        assert!(output.status.success(), "git {:?} failed", args);
        String::from_utf8(output.stdout).expect("utf8 output").trim().to_string()
    }

    fn init_repo(dir: &Path) {
        git(dir, &["init"]);
        git(dir, &["config", "user.name", "Test User"]);
        git(dir, &["config", "user.email", "test@example.com"]);
        git(dir, &["checkout", "-b", "main"]);
    }

    fn commit_file(dir: &Path, name: &str, contents: &str) -> String {
        std::fs::write(dir.join(name), contents).expect("write file");
        git(dir, &["add", name]);
        git(dir, &["commit", "-m", "test commit"]);
        git_output(dir, &["rev-parse", "HEAD"])
    }

    async fn start_test_server(pubkey: String) -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind test server");
        let addr = listener.local_addr().expect("local addr");
        let state = Arc::new(Mutex::new(TestServerState {
            pubkey,
            base_url: format!("http://{addr}"),
            ..Default::default()
        }));

        let app = Router::new()
            .route("/upload", put(upload_blob))
            .route("/:key", get(get_blob_or_list))
            .with_state(state);

        let handle = tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .expect("serve test server");
        });

        (addr, handle)
    }

    async fn upload_blob(
        State(state): State<Arc<Mutex<TestServerState>>>,
        headers: HeaderMap,
        body: axum::body::Bytes,
    ) -> Result<Json<UploadResponse>, StatusCode> {
        let mut hasher = Sha256::new();
        hasher.update(&body);
        let sha256 = encode(hasher.finalize());

        let content_type = headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("application/octet-stream")
            .to_string();
        let mut state = state.lock().expect("state lock");
        let created = state.next_created.saturating_add(1);
        state.next_created = created;
        state.blobs.insert(
            sha256.clone(),
            StoredBlob {
                content_type,
                created,
                body: body.to_vec(),
            },
        );

        Ok(Json(UploadResponse {
            sha256,
            size: body.len(),
        }))
    }

    async fn get_blob_or_list(
        State(state): State<Arc<Mutex<TestServerState>>>,
        AxumPath(key): AxumPath<String>,
    ) -> Result<axum::response::Response, StatusCode> {
        let state = state.lock().expect("state lock");

        if key == state.pubkey {
            let mut blobs: Vec<BlobDescriptor> = state
                .blobs
                .iter()
                .map(|(sha256, blob)| BlobDescriptor {
                    sha256: sha256.clone(),
                    url: format!("{}/{}", state.base_url, sha256),
                    content_type: Some(blob.content_type.clone()),
                    created: Some(blob.created),
                })
                .collect();
            blobs.sort_by(|a, b| b.created.cmp(&a.created));
            return Ok((StatusCode::OK, Json(blobs)).into_response());
        }

        if let Some(blob) = state.blobs.get(&key) {
            return Ok((StatusCode::OK, blob.body.clone()).into_response());
        }

        Err(StatusCode::NOT_FOUND)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn blossom_remote_push_list_and_fetch_round_trip() {
        let _cwd_lock = TEST_CWD_LOCK.lock().expect("cwd lock");
        let pubkey = "0".repeat(64);
        let (addr, server_handle) = start_test_server(pubkey.clone()).await;
        let server_url = format!("http://{addr}");

        tokio::task::spawn_blocking({
            let server_url = server_url.clone();
            let pubkey = pubkey.clone();
            move || {
                let tempdir = TempDir::new().expect("tempdir");
                init_repo(tempdir.path());
                let commit_sha = commit_file(tempdir.path(), "hello.txt", "hello blossom");

                let _cwd = set_current_dir(tempdir.path());
                let mut remote = BlossomRemote::new(&server_url, &pubkey, "demo");
                let push_results = remote
                    .push(vec![PushSpec {
                        force: false,
                        src: "HEAD".into(),
                        dst: "refs/heads/main".into(),
                    }])
                    .expect("push through blossom server");
                assert!(push_results.iter().all(|result| result.result.is_ok()));

                let refs = remote.list(false).expect("list through blossom server");
                assert!(refs.iter().any(|r| r.name == "refs/heads/main" && r.oid == commit_sha));
                assert!(refs
                    .iter()
                    .any(|r| r.name == "HEAD" && r.symref_target.as_deref() == Some("refs/heads/main")));

                let fetch_dir = TempDir::new().expect("fetch dir");
                init_repo(fetch_dir.path());
                let _fetch_cwd = set_current_dir(fetch_dir.path());
                remote.fetch(vec![]).expect("fetch through blossom server");
                let object_type = git_output(fetch_dir.path(), &["cat-file", "-t", &commit_sha]);
                assert_eq!(object_type, "commit");
            }
        })
        .await
        .expect("blocking integration test");

        server_handle.abort();
    }

    static TEST_CWD_LOCK: Mutex<()> = Mutex::new(());
}
