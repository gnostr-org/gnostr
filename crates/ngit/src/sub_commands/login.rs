use anyhow::Result;
use clap;

#[cfg(not(test))]
use crate::client::Client;
#[cfg(test)]
use crate::client::MockConnect;
use crate::{client::Connect, login, NgitCli};

#[derive(Debug, clap::Args)]
pub struct LoginSubCommandArgs {
    /// don't fetch user metadata and relay list from relays
    #[arg(long, action)]
    pub offline: bool,
}

pub async fn launch(args: &NgitCli, command_args: &LoginSubCommandArgs) -> Result<()> {
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
