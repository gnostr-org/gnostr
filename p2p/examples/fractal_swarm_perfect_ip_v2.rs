use gnostr_p2p::perfect_ip_v2;

#[tokio::main]
async fn main() {
    if let Err(e) = perfect_ip_v2::run_v2().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
