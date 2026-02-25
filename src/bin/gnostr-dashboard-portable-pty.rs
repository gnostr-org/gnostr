#[tokio::main]
async fn main() -> anyhow::Result<()> {
    gnostr::dashboard::run_dashboard().await
}
