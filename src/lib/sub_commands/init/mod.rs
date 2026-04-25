use anyhow::Result;

pub use ngit::sub_commands::init::SubCommandArgs as InitArgs;

fn ngit_cli(args: &InitArgs) -> ngit::cli::Cli {
    ngit::cli::Cli {
        command: None,
        bunker_uri: args.bunker_uri.clone(),
        bunker_app_key: args.bunker_app_key.clone(),
        nsec: args.nsec.clone(),
        password: args.password.clone(),
        disable_cli_spinners: args.disable_cli_spinners,
        customize: false,
        defaults: false,
        interactive: false,
        force: false,
        verbose: false,
    }
}

pub async fn launch(args: &InitArgs) -> Result<()> {
    let cli = ngit_cli(args);
    ngit::sub_commands::init::launch(&cli, args).await
}
