use anyhow::Result;
use ngit::{
    client::{Client, Connect},
    git::Repo,
    login::fresh::{fresh_login_or_signup, login_with_bunker_url},
};

#[derive(Clone, Debug, clap::Args)]
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

pub async fn launch(command_args: &LoginArgs) -> Result<()> {
    let git_repo_result = Repo::discover().ok();
    let client = if command_args.offline {
        None
    } else {
        Some(Client::default())
    };

    let git_repo = git_repo_result.as_ref();
    if let Some(bunker_url) = &command_args.bunker_url {
        login_with_bunker_url(
            &git_repo,
            client.as_ref(),
            bunker_url,
            command_args.local,
            &command_args.signer_relays,
        )
        .await?;
    } else {
        fresh_login_or_signup(
            &git_repo,
            client.as_ref(),
            None,
            command_args.local,
            &command_args.signer_relays,
        )
        .await?;
    }

    if let Some(client) = client {
        client.disconnect().await?;
    }
    Ok(())
}
