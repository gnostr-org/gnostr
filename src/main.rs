use anyhow::Result;
//use crate::sub_commands::custom_event::CustomEventCommand;
//use crate::Commands::CustomEvent;
//use gnostr::global_rt::global_rt;
use clap::{Parser /*, Subcommand*/};
//use gnostr::global_rt;
//use gnostr::input::InputEvent;
use gnostr::cli::{GnostrCli, GnostrCommands};
use gnostr::sub_commands;
//use gnostr::chat::*;
//use gnostr::chat::chat;
//use gnostr::tui::*;
//use gnostr::utils;
//use nostr_sdk::Result as NostrResult;
use sha2::{Digest, Sha256};
use std::env;
//use std::{error::Error, time::Duration};
//use tracing::{/*debug, /*error, info, span,*/ trace, /* warn,*/*/ Level};
use tracing_subscriber::FmtSubscriber;

use tracing::{debug /*, info*/};
use tracing_core::metadata::LevelFilter;

use serde::ser::StdError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    let mut args: GnostrCli = GnostrCli::parse();
    let level = if args.debug {
        LevelFilter::DEBUG
    } else if args.trace {
        LevelFilter::TRACE
    } else {
        LevelFilter::OFF
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let env_args: Vec<String> = env::args().collect();

    for arg in &env_args {
        println!("arg={:?}", arg);
    }
    if !args.hash.is_none() {
        //not none
        if let Some(input_string) = args.hash {
            let mut hasher = Sha256::new();
            hasher.update(input_string.as_bytes());
            let result = hasher.finalize();
            if env_args.len().clone() == 3 {
                print!("{:x}", result);
            }
            args.nsec = format!("148:{:x}", result).into();
        } else {
        }
    } else {
    }

    // Post event
    match &args.command {
        Some(GnostrCommands::Tui(sub_command_args)) => {
            sub_commands::tui::tui(sub_command_args).await
        }
        Some(GnostrCommands::Chat(sub_command_args)) => {
            sub_commands::chat::chat(sub_command_args).await
        }
        Some(GnostrCommands::Ngit(sub_command_args)) => {
            sub_commands::ngit::ngit(sub_command_args).await
        }
        Some(GnostrCommands::SetMetadata(sub_command_args)) => {
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
        Some(GnostrCommands::TextNote(sub_command_args)) => {
            sub_commands::text_note::broadcast_textnote(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::PublishContactListCsv(sub_command_args)) => {
            sub_commands::publish_contactlist_csv::publish_contact_list_from_csv_file(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::DeleteEvent(sub_command_args)) => {
            sub_commands::delete_event::delete(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::DeleteProfile(sub_command_args)) => {
            sub_commands::delete_profile::delete(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::React(sub_command_args)) => {
            sub_commands::react::react_to_event(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::ListEvents(sub_command_args)) => {
            sub_commands::list_events::list_events(args.relays, sub_command_args).await
        }
        Some(GnostrCommands::GenerateKeypair(sub_command_args)) => {
            sub_commands::generate_keypair::get_new_keypair(sub_command_args).await
        }
        Some(GnostrCommands::ConvertKey(sub_command_args)) => {
            sub_commands::convert_key::convert_key(sub_command_args).await
        }
        Some(GnostrCommands::Vanity(sub_command_args)) => {
            sub_commands::vanity::vanity(sub_command_args).await
        }
        Some(GnostrCommands::CreatePublicChannel(sub_command_args)) => {
            sub_commands::create_public_channel::create_public_channel(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SetChannelMetadata(sub_command_args)) => {
            sub_commands::set_channel_metadata::set_channel_metadata(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SendChannelMessage(sub_command_args)) => {
            sub_commands::send_channel_message::send_channel_message(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::HidePublicChannelMessage(sub_command_args)) => {
            sub_commands::hide_public_channel_message::hide_public_channel_message(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::MutePublicKey(sub_command_args)) => {
            sub_commands::mute_publickey::mute_publickey(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::BroadcastEvents(sub_command_args)) => {
            sub_commands::broadcast_events::broadcast_events(args.relays, sub_command_args).await
        }
        Some(GnostrCommands::CreateBadge(sub_command_args)) => {
            sub_commands::create_badge::create_badge(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::AwardBadge(sub_command_args)) => {
            sub_commands::award_badge::award_badge(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::ProfileBadges(sub_command_args)) => {
            sub_commands::profile_badges::set_profile_badges(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::CustomEvent(sub_command_args)) => {
            sub_commands::custom_event::create_custom_event(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SetUserStatus(sub_command_args)) => {
            sub_commands::user_status::set_user_status(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        _ => {
            {
                //let _ = gnostr::tui::tui().await;
            };
            Ok(())
        }
    }
}
