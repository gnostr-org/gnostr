use kubo_ffi::{self, Node};

fn main() {
    println!("Kubo version: {}", kubo_ffi::version());
    let dir = std::env::temp_dir().join("kubo-rust-demo");
    let _ = std::fs::remove_dir_all(&dir);
    let repo = dir.to_string_lossy().to_string();
    if let Err(e) = kubo_ffi::init_repo(&repo) {
        eprintln!("init failed: {}", e);
        return;
    }
    match Node::start(&repo, true) {
        Ok(node) => {
            println!("peer id: {:?}", node.peer_id());
            // Dial a bootstrap peer to demonstrate real network connectivity
            let bootstrap =
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN";
            println!("dialing bootstrap: {}", bootstrap);
            let ok = node.connect(bootstrap);
            println!("connect result: {}", ok);
            std::thread::sleep(std::time::Duration::from_secs(10));
            let peers = node.swarm_peers();
            println!("connected peers: {}", peers.len());
            for p in peers.iter().take(10) {
                println!("  {}", p);
            }
            for p in peers.iter().take(5) {
                println!("  {}", p);
            }
            let logs = kubo_ffi::get_logs();
            if !logs.is_empty() {
                println!("--- logs ---\n{}", logs);
            }
        }
        Err(e) => eprintln!("node start failed: {}", e),
    }
}
