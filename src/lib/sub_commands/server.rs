use clap::Parser;

/// Runs the Blossom server wrapper.
#[derive(Parser, Debug, Clone)]
#[command(about = "Run the Blossom server", long_about = None)]
pub struct ServerSubCommand {
    /// Pass-through arguments for blossom-server.
    #[arg(trailing_var_arg = true)]
    pub blossom_args: Vec<String>,
}

/// Launch the Blossom server wrapper.
pub async fn server(args: &ServerSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    crate::server::run_with_args(args.blossom_args.clone())?;
    Ok(())
}
