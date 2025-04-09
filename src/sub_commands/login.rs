use anyhow::Result;
use clap;

#[cfg(not(test))]
use ngit::client::Client;
#[cfg(test)]
use ngit::client::MockConnect;
use ngit::{client::Connect, login, NgitCli};

#[derive(clap::Args)]
pub struct SubCommandArgs {
    /// don't fetch user metadata and relay list from relays
    #[arg(long, action)]
    offline: bool,
}

pub async fn launch(args: &NgitCli, command_args: &SubCommandArgs) -> Result<()> {
    if command_args.offline {
        login::launch(&args.nsec, &args.password, None).await?;
        Ok(())
    } else {
        #[cfg(not(test))]
        let client = Client::default();
        #[cfg(test)]
        let client = <MockConnect as std::default::Default>::default();

        login::launch(&args.nsec, &args.password, Some(&client)).await?;
        client.disconnect().await?;
        Ok(())
    }
}
