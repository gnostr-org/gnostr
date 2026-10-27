#[tokio::main]
async fn main() {
    gnostr_mcp_agent::setup_tracing();
    gnostr_mcp_agent::run().await;
}
