use crate::sub_commands;
use crate::sub_commands::*;

use clap::{Parser, Subcommand};

#[derive(Parser)]
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
    pub disable_cli_spinners: Option<bool>,
}

#[derive(Subcommand)]
pub enum NgitCommands {
    /// update cache with latest updates from nostr
    Fetch(sub_commands::fetch::SubCommandArgs),
    /// signal you are this repo's maintainer accepting proposals via
    /// nostr
    Init(sub_commands::init::SubCommandArgs),
    /// issue commits as a proposal
    Send(sub_commands::send::SubCommandArgs),
    /// list proposals; checkout, apply or download selected
    List,
    /// send proposal revision
    Push(sub_commands::push::SubCommandArgs),
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
struct GnostrCli {
    #[command(subcommand)]
    command: Option<GnostrCommands>,
    ///
    #[arg(short, long, action = clap::ArgAction::Append, default_value = "0000000000000000000000000000000000000000000000000000000000000001")]
    nsec: Option<String>,
    ///
    #[arg(long, value_name = "STRING", help = "gnostr --hash '<string>'")]
    hash: Option<String>,
    ///
    #[arg(short, long, action = clap::ArgAction::Append,
		default_values_t = ["wss://relay.damus.io".to_string(),"wss://nos.lol".to_string()])]
    relays: Vec<String>,
    /// Proof of work difficulty target
    #[arg(short, long, action = clap::ArgAction::Append, default_value_t = 0)]
    difficulty_target: u8,

    /// Enable debug logging
    #[clap(long, default_value = "false")]
    debug: bool,

    /// Enable trace logging
    #[clap(long, default_value = "false")]
    trace: bool,
}

#[derive(Subcommand)]
enum GnostrCommands {
    /// Chat sub commands
    Chat(crate::chat::ChatSubCommands),
    /// Ngit sub commands
    Ngit(ngit::NgitSubCommand),
    /// Set metadata. Be aware that this will simply replace your current kind 0 event.
    SetMetadata(sub_commands::set_metadata::SetMetadataSubCommand),
    /// Send text note
    TextNote(sub_commands::text_note::TextNoteSubCommand),
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
    /// Create custom event
    CustomEvent(sub_commands::custom_event::CustomEventCommand),
    /// Create a user status event
    SetUserStatus(sub_commands::user_status::UserStatusSubCommand),
}

//#[derive(Parser, Debug)]
//#[command(author, version, about, long_about = None)]
//#[command(propagate_version = true)]
//pub struct ChatCli {
//    //    #[command(subcommand)]
//    //    pub command: ChatCommands,
//    /// remote signer address
//    //#[arg(long, global = true)]
//    //pub bunker_uri: Option<String>,
//    ///// remote signer app secret key
//    //#[arg(long, global = true)]
//    //pub bunker_app_key: Option<String>,
//    /// nsec or hex private key
//    #[arg(short, long, global = true)]
//    pub nsec: Option<String>,
//    /// password to decrypt nsec
//    #[arg(short, long, global = true)]
//    pub password: Option<String>,
//    ///// disable spinner animations
//    //#[arg(long, action)]
//    //pub disable_cli_spinners: bool,
//}

//#[derive(Subcommand, Debug)]
//pub enum ChatCommands {
//    /// update cache with latest updates from nostr
//    Fetch(sub_commands::fetch::SubCommandArgs),
//    /// signal you are this repo's maintainer accepting proposals via
//    /// nostr
//    Init(sub_commands::init::SubCommandArgs),
//    /// issue commits as a proposal
//    Send(sub_commands::send::SubCommandArgs),
//    /// list proposals; checkout, apply or download selected
//    List,
//    /// send proposal revision
//    Push(sub_commands::push::SubCommandArgs),
//    /// fetch and apply new proposal commits / revisions linked to
//    /// branch
//    Pull,
//    /// run with --nsec flag to change npub
//    Login(sub_commands::login::SubCommandArgs),
//}
