use anyhow::{anyhow, Result};
use clap::{
    /*crate_authors, crate_description, crate_name, Arg, Command as ClapApp, */ Parser,
    Subcommand,
};
 // Corrected import path for ArgMatches
use gnostr_asyncgit::sync::RepoPath;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::{
    //env,
    fs::{self, File},
    path::PathBuf,
};

// Import individual sub_commands modules directly
use crate::sub_commands::fetch;
use crate::sub_commands::init;
use crate::sub_commands::send;
use crate::sub_commands::push;
use crate::sub_commands::login;
use crate::sub_commands::legit;
use crate::sub_commands::ngit;
use crate::sub_commands::set_metadata;
use crate::sub_commands::note;
use crate::sub_commands::publish_contactlist_csv;
use crate::sub_commands::delete_event;
use crate::sub_commands::delete_profile;
use crate::sub_commands::react;
use crate::sub_commands::list_events;
use crate::sub_commands::generate_keypair;
use crate::sub_commands::convert_key;
use crate::sub_commands::vanity;
use crate::sub_commands::create_public_channel;
use crate::sub_commands::set_channel_metadata;
use crate::sub_commands::send_channel_message;
use crate::sub_commands::hide_public_channel_message;
use crate::sub_commands::mute_publickey;
use crate::sub_commands::broadcast_events;
use crate::sub_commands::create_badge;
use crate::sub_commands::award_badge;
use crate::sub_commands::profile_badges;
use crate::sub_commands::custom_event;
use crate::sub_commands::user_status;
// Import the new relay subcommand module
use crate::sub_commands::relay;
// Import the new QuerySubCommand struct
use crate::sub_commands::query::QuerySubCommand;
// Import the sniper subcommand module
use crate::sub_commands::sniper;
use crate::sub_commands::gitsh;

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
    pub command: Option<LegitCommands>,
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
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_cli_spinners: bool,
}

#[derive(Subcommand, Debug)]
pub enum LegitCommands {
    /// update cache with latest updates from nostr
    Fetch(fetch::FetchArgs),
    /// signal you are this repo's maintainer accepting proposals via
    /// nostr
    Init(init::InitArgs),
    /// issue commits as a proposal
    Send(send::SendArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    Push(push::PushArgs),
    /// fetch and apply new proposal commits / revisions linked to
    /// branch
    Pull,
    /// run with --nsec flag to change npub
    Login(login::LoginArgs),
    /// Mine a git commit with a given prefix
    Mine,
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
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub disable_cli_spinners: bool,
}

#[derive(Subcommand, Debug)]
pub enum NgitCommands {
    /// update cache with latest updates from nostr
    Fetch(fetch::FetchArgs),
    /// signal you are this repo's maintainer accepting proposals via
    /// nostr
    Init(init::InitArgs),
    /// issue commits as a proposal
    Send(send::SendArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    Push(push::PushArgs),
    /// fetch and apply new proposal commits / revisions linked to
    /// branch
    Pull,
    /// run with --nsec flag to change npub
    Login(login::LoginArgs),
    /// Query events from relays
    Query(QuerySubCommand),
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
		default_values_t = vec!["wss://relay.damus.io".to_string(), "wss://nos.lol".to_string()]
    pub relays: Vec<String>,
    /// Proof of work difficulty target
    #[arg(short, long, action = clap::ArgAction::Append, default_value_t = 0)]
    pub difficulty_target: u8,

    /// Enable info logging
    #[clap(long, default_value = "false", conflicts_with = "logging")]
    pub info: bool,

    /// Enable debug logging
    #[clap(long, default_value = "false", conflicts_with = "logging")]
    pub debug: bool,

    /// Enable trace logging
    #[clap(long, default_value = "false", conflicts_with = "logging")]
    pub trace: bool,

    /// Enable warn logging
    #[clap(long, default_value = "false", conflicts_with = "logging")]
    pub warn: bool,

    /// Generate bugreport
    #[clap(long, default_value = "false")]
    pub bugreport: bool,

    /// Enable logging
    #[clap(long, default_value = "false", conflicts_with = "info", conflicts_with = "debug", conflicts_with = "trace", conflicts_with = "warn")]
    pub logging: bool,
}

#[derive(Subcommand)]
pub enum GnostrCommands {
    /// Gnostr sub commands
    Tui(crate::core::GnostrSubCommands),
    /// Perform actions related to sniping relays
    Sniper(sniper::SniperArgs),
    /// Chat sub commands
    Chat(crate::p2p::chat::ChatSubCommands),
    /// Legit sub commands
    Legit(legit::LegitSubCommand),
    /// Ngit sub commands
    Ngit(ngit::NgitSubCommand),
    /// Set metadata.
    /// CAUTION!
    /// This will replace your current kind 0 event.
    SetMetadata(set_metadata::SetMetadataSubCommand),
    /// Send text note
    Note(note::NoteSubCommand),
    /// Publish contacts from a CSV file
    PublishContactListCsv(publish_contactlist_csv::PublishContactListCsvSubCommand),
    /// Delete an event
    DeleteEvent(delete_event::DeleteEventSubCommand),
    /// Delete a profile
    DeleteProfile(delete_profile::DeleteProfileSubCommand),
    /// React to an event
    React(react::ReactionSubCommand),
    /// Get all events
    ListEvents(list_events::ListEventsSubCommand),
    /// Generate a new keypair
    GenerateKeypair(generate_keypair::GenerateKeypairSubCommand),
    /// Convert key from bech32 to hex or hex to bech32
    ConvertKey(convert_key::ConvertKeySubCommand),
    /// Vanity public key mining
    Vanity(vanity::VanitySubCommand),
    /// Create a new public channel
    CreatePublicChannel(create_public_channel::CreatePublicChannelSubCommand),
    /// Update channel metadata
    SetChannelMetadata(set_channel_metadata::SetChannelMetadataSubCommand),
    /// Send a message to a public channel
    SendChannelMessage(send_channel_message::SendChannelMessageSubCommand),
    /// Hide a message in a public chat room
    HidePublicChannelMessage(
        hide_public_channel_message::HidePublicChannelMessageSubCommand,
    ),
    /// Mute a public key
    MutePublicKey(mute_publickey::MutePublickeySubCommand),
    /// Broadcast events from file
    BroadcastEvents(broadcast_events::BroadcastEventsSubCommand),
    /// Create a new badge
    CreateBadge(create_badge::CreateBadgeSubCommand),
    /// Publish award badge event
    AwardBadge(award_badge::AwardBadgeSubCommand),
    /// Set profile badges
    ProfileBadges(profile_badges::ProfileBadgesSubCommand),
    /// Create	custom	event	more
    /// 1	custom	event	more
    /// 2	custom	event	more
    CustomEvent(custom_event::CustomEventCommand),
    /// Create a user status event
    SetUserStatus(user_status::UserStatusSubCommand),
        /// Relay sub commands
    Relay(relay::RelaySubCommand),
    // Add the query subcommand here, using the new QuerySubCommand struct
    Query(QuerySubCommand),
    /// Gitsh sub commands
    Gitsh(gitsh::GitshSubCommand),
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

//#[test]
//fn verify_app() {
//    app().debug_assert();
//}
