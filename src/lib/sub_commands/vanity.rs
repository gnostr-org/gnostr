use anyhow::Result;
use clap::Args;

use crate::types::Keys;

#[derive(Args, Debug)]
pub struct VanitySubCommand {
    /// Prefixes
    #[arg(short, long, required = true, action = clap::ArgAction::Append)]
    prefixes: Vec<String>,
    /// Vanity pubkey in hex format
    #[arg(long, default_value_t = false)]
    hex: bool,
}

pub async fn vanity(sub_command_args: &VanitySubCommand) -> anyhow::Result<()> {
    let num_cores = num_cpus::get();
    let keys = Keys::vanity(
        sub_command_args.prefixes.clone(),
        !sub_command_args.hex,
        num_cores,
    )?;

    if sub_command_args.hex {
        println!("Public key (hex): {}", keys.public_key());
    } else {
        println!("Public key: {}", keys.public_key().as_bech32_string());
    }

    println!("Private key: {}", keys.secret_key()?.as_bech32_string());

    Ok(())
}
