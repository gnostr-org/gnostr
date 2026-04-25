use anyhow::Result;

pub use ngit::sub_commands::init::SubCommandArgs as InitArgs;

pub async fn launch(args: &InitArgs) -> Result<()> {
    let cli = ngit::cli::Cli {
        command: None,
        bunker_uri: None,
        bunker_app_key: None,
        nsec: None,
        password: None,
        disable_cli_spinners: false,
        customize: false,
        defaults: false,
        interactive: false,
        force: false,
        verbose: false,
    };
    ngit::sub_commands::init::launch(&cli, args).await
}
