use anyhow::Result;

/// Docker helper subcommand: exposes the gnostr-docker CLI.
pub async fn docker(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        // Default to the Docker container TUI when no subcommand is given.
        let oxker_args = Vec::new();
        gnostr_docker::run_with_args(&oxker_args, Some(gnostr_docker::docker::CONTAINER_NAME.to_string())).await;
        return Ok(());
    }

    gnostr_docker::cli::run_from(args).await?;
    Ok(())
}
