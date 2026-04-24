use clap::Parser;

/// Runs the Blossom server wrapper.
#[derive(Parser, Debug, Clone)]
#[command(about = "Run the Blossom server", long_about = None, disable_help_flag = true)]
pub struct ServerSubCommand {
    /// Show blossom-server help.
    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue)]
    pub help: bool,

    /// Pass-through arguments for blossom-server.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub blossom_args: Vec<String>,
}

/// Launch the Blossom server wrapper.
pub async fn server(args: &ServerSubCommand) -> Result<(), Box<dyn std::error::Error>> {
    if args.help {
        crate::server::print_help()?;
        return Ok(());
    }

    crate::server::run_with_args(args.blossom_args.clone())?;
    Ok(())
}
