use crate::sub_commands;
use crate::sub_commands::*;
use anyhow::{anyhow, Result};
use anyhow::bail;
use clap::{
    /*crate_authors, crate_description, crate_name, Arg, Command as ClapApp, */ Parser,
    Subcommand,
};
use gnostr_asyncgit::sync::RepoPath;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{
    //env,
    fs::{self, File},
    path::PathBuf,
};

use crate::login::SignerInfo;

#[derive(Subcommand, Debug)]
pub enum AccountCommands {
    /// login with nsec or nostr connect
    Login(sub_commands::login::SubCommandArgs),
    /// remove nostr account details stored in git config
    Logout,
    /// export nostr keys to login to other nostr clients
    ExportKeys,
}

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct AccountSubCommandArgs {
    #[command(subcommand)]
    pub account_command: AccountCommands,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
    pub theme: PathBuf,
    pub repo_path: RepoPath,
    pub notify_watcher: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LegitCli {
    #[command(subcommand)]
    pub command: LegitCommands,
    /// remote signer address
    #[arg(long, global = true)]
    pub bunker_uri: Option<String>,
    /// remote signer app secret key
    #[arg(long, global = true)]
    pub bunker_app_key: Option<String>,
    /// nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    /// password to decrypt nsec
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    /// disable spinner animations
    #[arg(long, action, default_value = "false")]
    pub disable_cli_spinners: bool,
}

#[derive(Subcommand, Debug)]
pub enum LegitCommands {
    /// signal you are this repo's maintainer accepting proposals via
    /// nostr
    Init(sub_commands::init::SubCommandArgs),
    /// issue commits as a proposal
    Send(sub_commands::send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// fetch and apply new proposal commits / revisions linked to
    /// branch
    Pull,
    /// run with --nsec flag to change npub
    Login(sub_commands::login::SubCommandArgs),
}
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NgitCli {
    #[command(subcommand)]
    pub command: NgitCommands,
    /// remote signer address
    #[arg(long, global = true)]
    pub bunker_uri: Option<String>,
    /// remote signer app secret key
    #[arg(long, global = true)]
    pub bunker_app_key: Option<String>,
    /// nsec or hex private key
    #[arg(short, long, global = true)]
    pub nsec: Option<String>,
    /// password to decrypt nsec
    #[arg(short, long, global = true)]
    pub password: Option<String>,
    /// disable spinner animations
    #[arg(long, action, default_value = "false")]
    pub disable_cli_spinners: bool,
}

#[derive(Subcommand, Debug)]
pub enum NgitCommands {
    /// signal you are this repo's maintainer accepting proposals via
    /// nostr
    Init(sub_commands::init::SubCommandArgs),
    /// issue commits as a proposal
    Send(sub_commands::send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// fetch and apply new proposal commits / revisions linked to
    /// branch
    Pull,
    /// run with --nsec flag to change npub
    Login(sub_commands::login::SubCommandArgs),
}

/// GnostrCli application to interact with nostr
#[derive(Parser)]
#[command(name = "gnostr")]
#[command(author = "gnostr <admin@gnostr.org>, 0xtr. <oxtrr@protonmail.com")]
#[command(version = "0.0.1")]
#[command(author, version, about, long_about = None)]
pub struct GnostrCli {
    //
    #[command(subcommand)]
    pub command: Option<GnostrCommands>,
    ///
    #[arg(short, long,
        action = clap::ArgAction::Append,
        default_value = "0000000000000000000000000000000000000000000000000000000000000001")]
    pub nsec: Option<String>,
    ///
    #[arg(long, value_name = "HASH", help = "gnostr --hash '<string>'")]
    pub hash: Option<String>,
    /// TODO handle gnostr tui --repo_path
    #[arg(
        long,
        value_name = "WORKDIR",
        default_value = ".",
        help = "gnostr --workdir '<string>'"
    )]
    pub workdir: Option<String>,
    /// TODO handle gnostr tui --repo_path
    #[arg(
        long,
        value_name = "GITDIR",
        default_value = ".",
        help = "gnostr --gitdir '<string>'"
    )]
    pub gitdir: Option<RepoPath>,
    ///
    #[arg(long, value_name = "DIRECTORY", help = "gnostr --directory '<string>'")]
    pub directory: Option<String>,
    ///
    #[arg(long, value_name = "THEME", help = "gnostr --theme '<string>'")]
    pub theme: Option<String>,
    ///
    #[arg(long, value_name = "WATCHER", help = "gnostr --watcher '<string>'")]
    pub watcher: Option<String>,
    ///
    #[arg(short, long, action = clap::ArgAction::Append,
		default_values_t = ["wss://relay.damus.io".to_string(),"wss://nos.lol".to_string()])]
    pub relays: Vec<String>,
    /// Proof of work difficulty target
    #[arg(short, long, action = clap::ArgAction::Append, default_value_t = 0)]
    pub difficulty_target: u8,

    /// Enable info logging
    #[clap(long, default_value = "false")]
    pub info: bool,

    /// Enable debug logging
    #[clap(long, default_value = "false")]
    pub debug: bool,

    /// Enable trace logging
    #[clap(long, default_value = "false")]
    pub trace: bool,

    /// Enable warn logging
    #[clap(long, default_value = "false")]
    pub warn: bool,

    /// Generate bugreport
    #[clap(long, default_value = "false")]
    pub bugreport: bool,

    /// Enable logging
    #[clap(long, default_value = "false")]
    pub logging: bool,
}

#[derive(Subcommand)]
pub enum GnostrCommands {
    /// Gnostr sub commands
    Tui(crate::gnostr::GnostrSubCommands),
    /// Chat sub commands
    Chat(crate::chat::ChatSubCommands),
    /// Ngit sub commands
    Ngit(ngit::NgitSubCommand),//
    /// Set metadata.
    /// CAUTION!
    /// This will replace your current kind 0 event.
    SetMetadata(sub_commands::set_metadata::SetMetadataSubCommand),
    /// Send text note
    Note(sub_commands::note::NoteSubCommand),
    /// Publish contacts from a CSV file
    PublishContactListCsv(sub_commands::publish_contactlist_csv::PublishContactListCsvSubCommand),
    /// Delete an event
    DeleteEvent(sub_commands::delete_event::DeleteEventSubCommand),
    /// Delete a profile
    DeleteProfile(sub_commands::delete_profile::DeleteProfileSubCommand),
    /// React to an event
    React(sub_commands::react::ReactionSubCommand),
    /// Get all events
    ListEvents(sub_commands::list_events::ListEventsSubCommand),
    /// Generate a new keypair
    GenerateKeypair(sub_commands::generate_keypair::GenerateKeypairSubCommand),
    /// Convert key from bech32 to hex or hex to bech32
    ConvertKey(sub_commands::convert_key::ConvertKeySubCommand),
    /// Vanity public key mining
    Vanity(sub_commands::vanity::VanitySubCommand),
    /// Create a new public channel
    CreatePublicChannel(sub_commands::create_public_channel::CreatePublicChannelSubCommand),
    /// Update channel metadata
    SetChannelMetadata(sub_commands::set_channel_metadata::SetChannelMetadataSubCommand),
    /// Send a message to a public channel
    SendChannelMessage(sub_commands::send_channel_message::SendChannelMessageSubCommand),
    /// Hide a message in a public chat room
    HidePublicChannelMessage(
        sub_commands::hide_public_channel_message::HidePublicChannelMessageSubCommand,
    ),
    /// Mute a public key
    MutePublicKey(sub_commands::mute_publickey::MutePublickeySubCommand),
    /// Broadcast events from file
    BroadcastEvents(sub_commands::broadcast_events::BroadcastEventsSubCommand),
    /// Create a new badge
    CreateBadge(sub_commands::create_badge::CreateBadgeSubCommand),
    /// Publish award badge event
    AwardBadge(sub_commands::award_badge::AwardBadgeSubCommand),
    /// Set profile badges
    ProfileBadges(sub_commands::profile_badges::ProfileBadgesSubCommand),
    /// Create	custom	event	more
    /// 1	custom	event	more
    /// 2	custom	event	more
    CustomEvent(sub_commands::custom_event::CustomEventCommand),
    /// Create a user status event
    SetUserStatus(sub_commands::user_status::UserStatusSubCommand),
}

pub fn setup_logging() -> Result<()> {
    let mut path = get_app_cache_path()?;
    path.push("gnostr.log");

    println!("Logging enabled. log written to: {path:?}");

    WriteLogger::init(LevelFilter::Trace, Config::default(), File::create(path)?)?;

    Ok(())
}

pub fn get_app_cache_path() -> Result<PathBuf> {
    let mut path = dirs::cache_dir().ok_or_else(|| anyhow!("failed to find os cache dir."))?;

    path.push("gnostr");
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn get_app_config_path() -> Result<PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join(".config"))
    } else {
        dirs::config_dir()
    }
    .ok_or_else(|| anyhow!("failed to find os config dir."))?;

    path.push("gnostr");
    fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn extract_signer_cli_arguments(args: &NgitCli) -> Result<Option<SignerInfo>> {
    if let Some(nsec) = &args.nsec {
        Ok(Some(SignerInfo::Nsec {
            nsec: nsec.to_string(),
            password: None,
            npub: None,
        }))
    } else if let Some(bunker_uri) = args.bunker_uri.clone() {
        if let Some(bunker_app_key) = args.bunker_app_key.clone() {
            Ok(Some(SignerInfo::Bunker {
                bunker_uri,
                bunker_app_key,
                npub: None,
            }))
        } else {
            bail!("cli argument bunker-app-key must be supplied when bunker-uri is")
        }
    } else if args.bunker_app_key.is_some() {
        bail!("cli argument bunker-uri must be supplied when bunker-app-key is")
    } else {
        Ok(None)
    }
}

//#[test]
//fn verify_app() {
//    app().debug_assert();
//}
