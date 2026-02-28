use anyhow::Result;
use clap::Parser;
use zeroize::Zeroize;

use gnostr_asyncgit::types::PrivateKey;

#[derive(Parser, Debug, Clone)]
pub struct PrivkeyToBech32SubCommand {
    /// Private key in hex format. If not provided, it will be prompted
    /// securely.
    #[arg(value_name = "PRIVATE_KEY_HEX", required = false)]
    pub privkey_hex: Option<String>,
}

pub fn privkey_to_bech32(sub_command_args: &PrivkeyToBech32SubCommand) -> Result<()> {
    let mut private_key_hex = String::new();

    if let Some(hex_arg) = &sub_command_args.privkey_hex {
        private_key_hex.clone_from(hex_arg);
    } else {
        private_key_hex = rpassword::prompt_password("Private key hex: ").unwrap();
    }

    let mut private_key = gnostr_asyncgit::types::PrivateKey::try_from_hex_string(&private_key_hex)?;
    private_key_hex.zeroize(); // Zeroize after use

    let mut bech32 = private_key.as_bech32_string();
    print!("{}", bech32);
    bech32.zeroize(); // Zeroize after use

    Ok(())
}
