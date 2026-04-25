use anyhow::Result;

use crate::cli::Cli;

pub use ngit::sub_commands::send::SubCommandArgs as SendArgs;

fn ngit_cli(args: &Cli, send_args: &SendArgs) -> ngit::cli::Cli {
    ngit::cli::Cli {
        command: None,
        bunker_uri: send_args
            .bunker_uri
            .clone()
            .or_else(|| args.bunker_uri.clone()),
        bunker_app_key: send_args
            .bunker_app_key
            .clone()
            .or_else(|| args.bunker_app_key.clone()),
        nsec: send_args.nsec.clone().or_else(|| args.nsec.clone()),
        password: send_args.password.clone().or_else(|| args.password.clone()),
        disable_cli_spinners: send_args.disable_cli_spinners || args.disable_cli_spinners,
        customize: false,
        defaults: args.defaults,
        interactive: args.interactive,
        force: args.force,
        verbose: args.verbose,
    }
}

pub async fn launch(args: &Cli, send_args: &SendArgs, no_fetch: bool) -> Result<()> {
    let cli = ngit_cli(args, send_args);
    ngit::sub_commands::send::launch(&cli, send_args, no_fetch).await
}
