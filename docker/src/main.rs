#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gnostr_docker::cli::run().await
}
