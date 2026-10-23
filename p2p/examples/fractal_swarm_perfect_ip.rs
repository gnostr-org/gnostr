use gnostr_p2p::perfect_ip;

#[tokio::main]
async fn main() {
    if let Err(e) = perfect_ip::run().await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
