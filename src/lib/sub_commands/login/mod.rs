use anyhow::Result;

use crate::cli::Cli;

#[derive(Clone, clap::Args)]
pub struct LoginArgs {
    /// login to the local git repository only
    #[arg(long)]
    pub local: bool,

    /// don't fetch user metadata and relay list from relays
    #[arg(long)]
    pub offline: bool,

    /// signer relay for nostrconnect (can be used multiple times)
    #[arg(long = "signer-relay")]
    pub signer_relays: Vec<String>,

    /// bunker:// URL from signer app for non-interactive remote signer login
    #[arg(long = "bunker-url")]
    pub bunker_url: Option<String>,
}

fn ngit_cli(args: &Cli) -> ngit::cli::Cli {
    ngit::cli::Cli {
        command: None,
        bunker_uri: args.bunker_uri.clone(),
        bunker_app_key: args.bunker_app_key.clone(),
        nsec: args.nsec.clone(),
        password: args.password.clone(),
        disable_cli_spinners: args.disable_cli_spinners,
        customize: false,
        defaults: args.defaults,
        interactive: args.interactive,
        force: args.force,
        verbose: args.verbose,
    }
}

pub async fn launch(args: &Cli, command_args: &LoginArgs) -> Result<()> {
    let cli = ngit_cli(args);
    let sub_args = ngit::sub_commands::login::SubCommandArgs {
        offline: command_args.offline,
        disable_cli_spinners: args.disable_cli_spinners,
        password: args.password.clone(),
        nsec: args.nsec.clone(),
        bunker_app_key: args.bunker_app_key.clone(),
        bunker_uri: command_args
            .bunker_url
            .clone()
            .or_else(|| args.bunker_uri.clone()),
    };

    ngit::sub_commands::login::launch(&cli, &sub_args).await
}
