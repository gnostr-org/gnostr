use crate::docker as docker_mod;

/// Run the Docker container TUI (oxker) to view containers.
pub async fn run(use_cli: bool) -> anyhow::Result<()> {
    if use_cli {
        // forwarded via env not needed; caller passes --use-cli to oxker args
    }
    let mut args = Vec::new();
    if use_cli {
        args.push("--use-cli".to_string());
    }
    crate::setup_tracing();
    crate::run_with_args(&args, Some(docker_mod::CONTAINER_NAME.to_string())).await;
    Ok(())
}