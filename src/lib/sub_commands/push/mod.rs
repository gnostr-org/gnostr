use anyhow::Result;

#[derive(Debug, clap::Args, Clone)]
pub struct PushArgs {
    #[arg(long)]
    /// send proposal revision from checked out proposal branch
    pub force: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_cli_spinners: bool,
    pub password: Option<String>,
    pub nsec: Option<String>,
    pub bunker_app_key: Option<String>,
    pub bunker_uri: Option<String>,
}

pub async fn launch(args: &PushArgs) -> Result<()> {
    let cli = ngit::cli::Cli {
        command: None,
        bunker_uri: args.bunker_uri.clone(),
        bunker_app_key: args.bunker_app_key.clone(),
        nsec: args.nsec.clone(),
        password: args.password.clone(),
        disable_cli_spinners: args.disable_cli_spinners,
        customize: false,
        defaults: false,
        interactive: false,
        force: args.force,
        verbose: false,
    };
    let send_args = ngit::sub_commands::send::SubCommandArgs {
        since_or_range: String::new(),
        in_reply_to: vec![],
        no_cover_letter: true,
        title: None,
        description: None,
        force_pr: args.force,
        force_patch: false,
        push_options: vec![],
    };

    ngit::sub_commands::send::launch(&cli, &send_args, false).await
}
