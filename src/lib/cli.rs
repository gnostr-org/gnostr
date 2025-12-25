#![warn(
    missing_docs,
)]

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
use crate::sub_commands::bech32_to_any;
use crate::sub_commands::privkey_to_bech32;
use crate::sub_commands::fetch_by_id;
// Import the new relay subcommand module
use crate::sub_commands::relay;
// Import the new QuerySubCommand struct
use crate::sub_commands::query::QuerySubCommand;
// Import the sniper subcommand module
use crate::sub_commands::sniper;
use crate::sub_commands::git;

/// CliArgs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliArgs {
	/// theme
    pub theme: PathBuf,
	/// repo_path
    pub repo_path: RepoPath,
	/// notify_watch
    pub notify_watcher: bool,
}

/// LegitCli
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct LegitCli {
	/// command
    #[command(subcommand)]
    pub command: Option<LegitCommands>,
}

/// LegitCommands
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

/// NgitCli
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct NgitCli {
	/// command
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

/// NgitCommands
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

/// GnostrCli
#[derive(Parser)]
#[command(name = "gnostr")]
#[command(author = "gnostr <admin@gnostr.org>, 0xtr. <oxtrr@protonmail.com")]
#[command(version = "0.0.1")]
#[command(author, version, about, long_about = None)]
pub struct GnostrCli {
    /// command
    #[command(subcommand)]
    pub command: Option<GnostrCommands>,
    /// nsec
    #[arg(
        short = 'n',
        long = "nsec",
        action = clap::ArgAction::Append,
        default_value = "0000000000000000000000000000000000000000000000000000000000000001",
        help = "nostr secret key",
        long_help = r#"
    Filter by event IDs (comma-separated).

    The argument supports complex command expansion patterns:

    gnostr --nsec $(gnostr --hash "key_material_to_sha256") \
    note \
    -c "text note"
    "#
    )]
    pub nsec: Option<String>,
    /// hash
    #[arg(long, help = "gnostr --hash <string>")]
    pub hash: Option<String>,
    /// TODO handle gnostr tui --repo_path
    #[arg(
        long,
        value_name = "WORKDIR",
        default_value = ".",
        help = "gnostr --workdir '<string>'"
    )]
	/// workdir
    pub workdir: Option<String>,
    /// TODO handle gnostr tui --repo_path
    #[arg(
        long,
        value_name = "GITDIR",
        default_value = ".",
        help = "gnostr --gitdir '<string>'"
    )]
	/// gitdir
    pub gitdir: Option<RepoPath>,
    /// directory
    #[arg(long, value_name = "DIRECTORY", help = "gnostr --directory '<string>'")]
    pub directory: Option<String>,
    /// theme
    #[arg(long, value_name = "THEME", help = "gnostr --theme '<string>'")]
    pub theme: Option<String>,
    /// watcher
    #[arg(long, value_name = "WATCHER", help = "gnostr --watcher '<string>'")]
    pub watcher: Option<String>,
    /// weeble
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub weeble: bool,
    /// blockheigh
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub blockheight: bool,
    /// wobble
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub wobble: bool,
    /// blockhash
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub blockhash: bool,
    ///  relays
    #[arg(short, long, action = clap::ArgAction::Append,
		default_values_t = ["wss://relay.damus.io".to_string(),"wss://nos.lol".to_string()])]
    pub relays: Vec<String>,
    /// Proof of work difficulty target
    #[arg(short, long, action = clap::ArgAction::Append, default_value_t = 0)]
    pub difficulty_target: u8,

    /// Take screenshots at a given interval in seconds. The interval defaults to 1 second.
    #[arg(long, value_name = "INTERVAL_SECONDS", num_args = 0..=1, default_missing_value = "1")]
    pub screenshots: Option<u8>,

    /// Enable info logging
    #[arg(long, default_value = "false", conflicts_with = "logging")]
    pub info: bool,

    /// Enable debug logging
    #[arg(long, default_value = "false", conflicts_with = "logging")]
    pub debug: bool,

    /// Enable trace logging
    #[arg(long, default_value = "false", conflicts_with = "logging")]
    pub trace: bool,

    /// Enable warn logging
    #[arg(long, default_value = "false", conflicts_with = "logging")]
    pub warn: bool,

    /// Generate bugreport
    #[arg(long, default_value = "false")]
    pub bugreport: bool,

    /// Enable logging
    #[arg(long, default_value = "false", conflicts_with = "info", conflicts_with = "debug", conflicts_with = "trace", conflicts_with = "warn")]
    pub logging: bool,
}

impl Default for GnostrCli {
    fn default() -> Self {
        Self {
            command: None,
            nsec: Some("0000000000000000000000000000000000000000000000000000000000000001".to_string()),
            hash: None,
            workdir: Some(".".to_string()),
            gitdir: Some(".".into()),
            directory: None,
            theme: None,
            watcher: None,
            weeble: false,
            blockheight: false,
            wobble: false,
            blockhash: false,
            relays: vec!["wss://relay.damus.io".to_string(), "wss://nos.lol".to_string()],
            difficulty_target: 0,
            screenshots: None,
            info: false,
            debug: false,
            trace: false,
            warn: false,
            bugreport: false,
            logging: false,
        }
    }
}

/// GnostrCommands
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
    /// Create custom event    more --help
    CustomEvent(custom_event::CustomEventCommand),
    /// Create a user status event
    SetUserStatus(user_status::UserStatusSubCommand),
    /// Convert bech32 string to other formats
    Bech32ToAny(bech32_to_any::Bech32ToAnySubCommand),
    /// Convert a private key to its bech32 representation
    PrivkeyToBech32(privkey_to_bech32::PrivkeyToBech32SubCommand),
    /// Fetch an event by ID
    FetchById(fetch_by_id::FetchByIdSubCommand),
        /// Relay sub commands
    Relay(relay::RelaySubCommand),
    /// Add the query subcommand here, using the new QuerySubCommand struct
    Query(QuerySubCommand),
    /// Git sub commands
    Git(git::GitSubCommand),
    /// Nip34 sub commands
    Nip34(crate::sub_commands::nip34::Nip34Command),
}

/// setup_logging
pub fn setup_logging() -> Result<()> {
    let mut path = get_app_cache_path()?;
    path.push("gnostr.log");

    println!("Logging enabled. log written to: {path:?}");

    WriteLogger::init(LevelFilter::Trace, Config::default(), File::create(path)?)?;

    Ok(())
}

/// get_app_cache_path
pub fn get_app_cache_path() -> Result<PathBuf> {
    let mut path = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join(".config"))
    } else {
        dirs::config_dir()
    }
    .ok_or_else(|| anyhow!("failed to find os cache dir."))?;

    path.push("gnostr");
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// get_app_config_path
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
