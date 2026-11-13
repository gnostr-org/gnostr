// Smoke test: publish a file, fetch it on a second node, serve a registry over HTTP.
use kubo_ffi::Node;
use std::fs;

fn main() {
    let base = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/kubo-smoke".into());
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();

    // node A: publish
    kubo_ffi::init_repo(&format!("{}/repoA", base)).unwrap();
    let a = Node::start(&format!("{}/repoA", base), true).unwrap();
    let data = b"hello from margo-p2p via kubo-ffi";
    let cid = a.unixfs_add_bytes(data).unwrap();
    println!("A published CID: {}", cid);

    // serve registry + gateway
    let regdir = format!("{}/reg", base);
    std::fs::create_dir_all(format!("{}/de/mo", regdir)).unwrap();
    std::fs::write(format!("{}/config.json", regdir), b"{\"dl\":\"x\"}").unwrap();
    assert!(a.add_static("/", &regdir));
    let gw = a.start_gateway("/ip4/127.0.0.1/tcp/0").unwrap();
    let port = gw.split("/tcp/").nth(1).unwrap();
    let url = format!("http://127.0.0.1:{}", port);
    println!("A gateway: {}", url);
    std::thread::sleep(std::time::Duration::from_millis(300));

    // verify gateway serves bytes
    let out = std::process::Command::new("curl")
        .args(["-s", &format!("{}/ipfs/{}", url, cid)])
        .output()
        .unwrap();
    assert_eq!(out.stdout, data, "gateway bytes mismatch");
    println!("gateway /ipfs/<cid> -> exact bytes OK");

    // node B: fetch by CID
    kubo_ffi::init_repo(&format!("{}/repoB", base)).unwrap();
    let b = Node::start(&format!("{}/repoB", base), true).unwrap();
    let first_addr = a
        .listening_addrs()
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let dial = format!("{}/p2p/{}", first_addr, a.peer_id().unwrap());
    assert!(b.connect(&dial), "connect failed");
    let got = b.unixfs_cat(&cid).unwrap();
    assert_eq!(got, data, "fetch bytes mismatch");
    println!("B fetched {} bytes, match OK", got.len());

    // networked init
    kubo_ffi::init_repo_networked(&format!("{}/repoN", base)).unwrap();
    let n = Node::start(&format!("{}/repoN", base), true).unwrap();
    println!(
        "networked node listening: {}",
        n.listening_addrs().unwrap().lines().next().unwrap()
    );
    drop(n);

    println!("ALL SMOKE CHECKS PASSED");
}
