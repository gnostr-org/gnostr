use clap::Args;
use nostr_sdk::prelude::*;

// cli for code collaboration over nostr with nip34 support
//
// Usage: ngit [OPTIONS] <COMMAND>
//
// Commands:
//   init   signal you are this repo's maintainer accepting proposals via nostr
//   send   issue commits as a proposal
//   list   list proposals; checkout, apply or download selected
//   push   send proposal revision
//   pull   fetch and apply new proposal commits / revisions linked to branch
//   login  run with --nsec flag to change npub
//   help   Print this message or the help of the given subcommand(s)
//
// Options:
//   -n, --nsec <NSEC>           nsec or hex private key
//   -p, --password <PASSWORD>   password to decrypt nsec
//       --disable-cli-spinners  disable spinner animations
//   -h, --help                  Print help
//   -V, --version               Print version
//

#[derive(Args, Debug)]
pub struct NgitSubCommand {
    /// ngit --init
    #[arg(long, short, default_value_t = false)]
    init: bool,
    /// ngit --send
    #[arg(long, short, default_value_t = false)]
    send: bool,
    /// ngit --list
    #[arg(long, default_value_t = false)]
    list: bool,
    /// ngit --push
    #[arg(long, default_value_t = false)]
    push: bool,
    /// ngit --pull
    #[arg(long, default_value_t = false)]
    pull: bool,
    /// ngit --login
    #[arg(long, short, default_value_t = false)]
    login: bool,
    /// ngit --help
    #[arg(long, short, default_value_t = false)]
    ngit_help: bool,
}

pub async fn ngit(sub_command_args: &NgitSubCommand) -> Result<()> {
    if sub_command_args.init {
        println!("sub_command_args.init={}", sub_command_args.init);
    } else if sub_command_args.send {
        println!("sub_command_args.send={}", sub_command_args.send);
    } else if sub_command_args.list {
        println!("sub_command_args.list={}", sub_command_args.list);
    } else if sub_command_args.push {
        println!("sub_command_args.push={}", sub_command_args.push);
    } else if sub_command_args.pull {
        println!("sub_command_args.pull={}", sub_command_args.pull);
    } else if sub_command_args.login {
        println!("sub_command_args.login={}", sub_command_args.login);
    } else if sub_command_args.ngit_help {
        println!("sub_command_args.ngit_help={}", sub_command_args.ngit_help);
    } else {
        println!("sub_command_args={:?}", sub_command_args);
    }

    Ok(())
}
