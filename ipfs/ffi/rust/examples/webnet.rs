use kubo_ffi::{self, Node};
use std::thread;
use std::time::Duration;
fn main() {
    println!("Kubo version: {}", kubo_ffi::version());
    let dir = std::env::temp_dir().join("kubo-rust-webnet");
    let _ = std::fs::remove_dir_all(&dir);
    let repo = dir.to_string_lossy().to_string();
    if let Err(e) = kubo_ffi::init_repo_networked(&repo) {
        eprintln!("init failed: {}", e);
        return;
    }
    match Node::start(&repo, true) {
        Ok(node) => {
            println!("peer id: {:?}", node.peer_id());
            match node.start_api("/ip4/127.0.0.1/tcp/0") {
                Some(url) => {
                    let parts: Vec<&str> = url.split('/').collect();
                    let mut host = "127.0.0.1";
                    let mut port = "5001";
                    let mut i = 0;
                    while i < parts.len() {
                        if parts[i] == "ip4" && i + 1 < parts.len() { host = parts[i + 1]; }
                        if parts[i] == "tcp" && i + 1 < parts.len() { port = parts[i + 1]; }
                        i += 1;
                    }
                    println!("WebUI: http://{}:{}/webui", host, port);
                    println!("Press Ctrl-C to stop.");
                    loop { thread::sleep(Duration::from_secs(1)); }
                }
                None => eprintln!("start_api failed: {:?}", kubo_ffi::last_error()),
            }
        }
        Err(e) => eprintln!("node start failed: {}", e),
    }
}
