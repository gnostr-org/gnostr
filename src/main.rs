//use crate::sub_commands::custom_event::CustomEventCommand;
//use crate::Commands::CustomEvent;
use clap::{Parser, Subcommand};
mod global_rt;
use crate::global_rt::global_rt;
use nostr_sdk::Result;
use sha2::{Digest, Sha256};
use std::env;
mod sub_commands;
mod utils;

use std::{io::stdout, time::Duration};

use futures::{future::FutureExt, select, StreamExt};
use futures_timer::Delay;

use crossterm::{
    cursor::position,
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

/// Simple CLI application to interact with nostr
#[derive(Parser)]
#[command(name = "gnostr")]
#[command(author = "gnostr <admin@gnostr.org>, 0xtr. <oxtrr@protonmail.com")]
#[command(version = "0.0.1")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    ///
    #[arg(short, long, action = clap::ArgAction::Append, default_value = "0000000000000000000000000000000000000000000000000000000000000001")]
    nsec: Option<String>,
    ///
    #[arg(long, value_name = "STRING", help = "gnostr --hash '<string>'")]
    hash: Option<String>,
    ///
    #[arg(short, long, action = clap::ArgAction::Append, default_values_t = ["wss://relay.damus.io".to_string(),"wss://e.nos.lol".to_string()])]
    relays: Vec<String>,
    /// Proof of work difficulty target
    #[arg(short, long, action = clap::ArgAction::Append, default_value_t = 0)]
    difficulty_target: u8,
}

#[derive(Subcommand)]
enum Commands {
    /// Ngit sub commands
    Ngit(sub_commands::ngit::NgitSubCommand),
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

const HELP: &str = r#"EventStream based on futures_util::Stream with tokio
 - Keyboard, mouse and terminal resize events enabled
 - Prints "." every second if there's no event
 - Hit "c" to print current cursor position
 - Use Esc to quit
"#;

async fn print_events() {
    let mut reader = EventStream::new();

    loop {
        let mut delay = Delay::new(Duration::from_millis(1_000)).fuse();
        let mut event = reader.next().fuse();

        select! {
            _ = delay => { println!(".\r"); },
            maybe_event = event => {
                match maybe_event {
                    Some(Ok(event)) => {
                        println!("Event::{:?}\r", event);

                        if event == Event::Key(KeyCode::Char('c').into()) {
                            println!("Cursor position: {:?}\r", position());
                        }

                        if event == Event::Key(KeyCode::Esc.into()) {
                            break;
                        }
                    }
                    Some(Err(e)) => println!("Error: {:?}\r", e),
                    None => break,
                }
            }
        };
    }
}

async fn interactive() -> Result<()> {
    println!("{}", HELP);
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnableMouseCapture)?;
    print_events().await;
    execute!(stdout, DisableMouseCapture)?;
    Ok(disable_raw_mode()?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let env_args: Vec<String> = env::args().collect();

    let global_rt_result = global_rt().spawn(async move {
        if env_args.len().clone() == 1 {
            // No arguments, read from stdin (using tokio).
            if let Err(e) = interactive().await {
                eprintln!("Error processing stdin: {}", e);
                std::process::exit(1);
            }
        } else {
        }
        String::from("global_rt async task!")
    });
    println!("global_rt_result={:?}", global_rt_result.await);
    let global_rt_result = global_rt().spawn(async move { String::from("global_rt async task!") });
    println!("global_rt_result={:?}", global_rt_result.await);

    let env_args: Vec<String> = env::args().collect();
    let args: Cli = Cli::parse();

    //if args.sec.is_some() && args.nsec.is_none() {
    //		args.nsec = Some(args.sec.clone().expect("REASON"));
    //}

    if !args.hash.is_none() {
        //not none
        if let Some(input_string) = args.hash {
            let mut hasher = Sha256::new();
            hasher.update(input_string.as_bytes());
            let result = hasher.finalize();
            if env_args.len().clone() == 3 {
                println!("{:x}", result);
            }
        } else {
        }
    } else {
        if args.hash.is_none() {}
    }

    // Post event
    match &args.command {
        Some(Commands::Ngit(sub_command_args)) => sub_commands::ngit::ngit(sub_command_args).await,
        Some(Commands::SetMetadata(sub_command_args)) => {
            {
                sub_commands::set_metadata::set_metadata(
                    args.nsec,
                    args.relays,
                    args.difficulty_target,
                    sub_command_args,
                )
            }
            .await
        }
        Some(Commands::TextNote(sub_command_args)) => {
            sub_commands::text_note::broadcast_textnote(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::PublishContactListCsv(sub_command_args)) => {
            sub_commands::publish_contactlist_csv::publish_contact_list_from_csv_file(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::DeleteEvent(sub_command_args)) => {
            sub_commands::delete_event::delete(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::DeleteProfile(sub_command_args)) => {
            sub_commands::delete_profile::delete(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::React(sub_command_args)) => {
            sub_commands::react::react_to_event(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::ListEvents(sub_command_args)) => {
            sub_commands::list_events::list_events(args.relays, sub_command_args).await
        }
        Some(Commands::GenerateKeypair(sub_command_args)) => {
            sub_commands::generate_keypair::get_new_keypair(sub_command_args).await
        }
        Some(Commands::ConvertKey(sub_command_args)) => {
            sub_commands::convert_key::convert_key(sub_command_args).await
        }
        Some(Commands::Vanity(sub_command_args)) => {
            sub_commands::vanity::vanity(sub_command_args).await
        }
        Some(Commands::CreatePublicChannel(sub_command_args)) => {
            sub_commands::create_public_channel::create_public_channel(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::SetChannelMetadata(sub_command_args)) => {
            sub_commands::set_channel_metadata::set_channel_metadata(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::SendChannelMessage(sub_command_args)) => {
            sub_commands::send_channel_message::send_channel_message(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::HidePublicChannelMessage(sub_command_args)) => {
            sub_commands::hide_public_channel_message::hide_public_channel_message(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::MutePublicKey(sub_command_args)) => {
            sub_commands::mute_publickey::mute_publickey(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::BroadcastEvents(sub_command_args)) => {
            sub_commands::broadcast_events::broadcast_events(args.relays, sub_command_args).await
        }
        Some(Commands::CreateBadge(sub_command_args)) => {
            sub_commands::create_badge::create_badge(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::AwardBadge(sub_command_args)) => {
            sub_commands::award_badge::award_badge(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::ProfileBadges(sub_command_args)) => {
            sub_commands::profile_badges::set_profile_badges(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::CustomEvent(sub_command_args)) => {
            sub_commands::custom_event::create_custom_event(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(Commands::SetUserStatus(sub_command_args)) => {
            sub_commands::user_status::set_user_status(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        None => {
            {
                //println!("gnostr -h");
            };
            //println!("gnostr -h");
            Ok(())
        }
    }
}
