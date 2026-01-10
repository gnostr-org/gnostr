use clap::Parser;
use gnostr::sub_commands::privkey_to_bech32::{PrivkeyToBech32SubCommand, privkey_to_bech32};

fn main() -> anyhow::Result<()> {
    let args = PrivkeyToBech32SubCommand::parse();
    privkey_to_bech32(&args)
}
