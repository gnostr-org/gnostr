#[tokio::main]
async fn main() {
    gnostr_docker::setup_tracing();
    gnostr_docker::run().await;
}
