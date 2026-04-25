use anyhow::Result;

pub use ngit::sub_commands::send::SubCommandArgs as SendArgs;

fn ngit_cli() -> ngit::cli::Cli {
    ngit::cli::Cli {
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
    }
}

pub async fn launch(send_args: &SendArgs, no_fetch: bool) -> Result<()> {
    let cli = ngit_cli();
    ngit::sub_commands::send::launch(&cli, send_args, no_fetch).await
}
