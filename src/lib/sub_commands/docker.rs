use anyhow::Result;

/// Docker helper subcommand: launches the gnostr-mcp-agent Docker TUI.
pub async fn docker() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let use_cli = args.iter().any(|a| a == "--use-cli");
    let mut oxker_args = Vec::new();
    if use_cli {
        oxker_args.push("--use-cli".to_string());
    }
    gnostr_mcp_agent::run_with_args(&oxker_args, Some(gnostr_mcp_agent::docker::CONTAINER_NAME.to_string())).await;
    Ok(())
}
