use clap::Args;
use nostr_sdk_0_32_0::prelude::*;

#[derive(Args, Debug)]
pub struct GenerateKeypairSubCommand {
    /// Print keys in hex. Defaults to printing bech32 encoded keys.
    #[arg(short, long, default_value = "false")]
    print_hex: bool,
}

pub async fn get_new_keypair(sub_command_args: &GenerateKeypairSubCommand) -> Result<()> {
    let keys = Keys::generate();
    if !sub_command_args.print_hex {
        print!(
            "{{\"private_key\":\"{}\"}}",
            keys.secret_key()?.to_bech32()?
        );
        print!("{{\"public_key\":\"{}\"}}", keys.public_key().to_bech32()?);
        //println!("Private key: {}", keys.secret_key()?.to_bech32()?);
        //println!("Public key: {}", keys.public_key().to_bech32()?);
        //println!("Private key: {}", keys.secret_key()?.display_secret());
        //println!("Public key: {}", keys.public_key())
    } else {
        print!(
            "{{\"private_key\":\"{}\"}}",
            keys.secret_key()?.display_secret()
        );
        print!("{{\"public_key\":\"{}\"}}", keys.public_key());
    }
    Ok(())
}
