use anyhow::Result;
use clap::{Parser /*, Subcommand*/};
use gnostr::cli::{get_app_cache_path, setup_logging, GnostrCli, GnostrCommands};
use gnostr::{blockheight, sub_commands};
use sha2::{Digest, Sha256};
use std::env;
use tracing::{debug, trace};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::FmtSubscriber;

use serde::ser::StdError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env::set_var("BLOCKHEIGHT", blockheight::blockheight_sync());
    let mut args: GnostrCli = GnostrCli::parse();
    let app_cache = get_app_cache_path();
    let _logging = if args.logging {
        let logging = setup_logging();
        trace!("{:?}", logging);
    };
    let level = if args.debug {
        LevelFilter::DEBUG
    } else if args.trace {
        LevelFilter::TRACE
    } else if args.info {
        LevelFilter::INFO
    } else if args.warn {
        LevelFilter::WARN
    } else {
        LevelFilter::OFF
    };
    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    trace!("{:?}", app_cache);

    let env_args: Vec<String> = env::args().collect();

    for arg in &env_args {
        trace!("arg={:?}", arg);
    }
    if !args.hash.is_none() {
        //not none
        if let Some(input_string) = args.hash {
            let mut hasher = Sha256::new();
            hasher.update(input_string.as_bytes());
            let result = hasher.finalize();
            //Usage: gnostr --hash <string>
            //Usage: gnostr --debug --hash <string>
            if env_args.len().clone() >= 3 && env_args.len().clone() <= 4
            /*--debug, --trace, --info, etc...*/
            {
                print!("{:x}", result);
                std::process::exit(0);
            }
            args.nsec = format!("{:x}", result).into();
        } else {
        }
    } else {
    }

    // Post event
    match &args.command {
        Some(GnostrCommands::Tui(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::tui::tui(sub_command_args).await
        }
        Some(GnostrCommands::Chat(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::chat::chat(&args.nsec.unwrap().to_string(), sub_command_args).await
        }
        Some(GnostrCommands::Ngit(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::ngit::ngit(sub_command_args).await
        }
        Some(GnostrCommands::SetMetadata(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
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
        Some(GnostrCommands::Note(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::note::broadcast_textnote(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::PublishContactListCsv(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::publish_contactlist_csv::publish_contact_list_from_csv_file(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::DeleteEvent(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::delete_event::delete(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::DeleteProfile(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::delete_profile::delete(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::React(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::react::react_to_event(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::ListEvents(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::list_events::list_events(args.relays, sub_command_args).await
        }
        Some(GnostrCommands::GenerateKeypair(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::generate_keypair::get_new_keypair(sub_command_args).await
        }
        Some(GnostrCommands::ConvertKey(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::convert_key::convert_key(sub_command_args).await
        }
        Some(GnostrCommands::Vanity(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::vanity::vanity(sub_command_args).await
        }
        Some(GnostrCommands::CreatePublicChannel(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::create_public_channel::create_public_channel(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SetChannelMetadata(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::set_channel_metadata::set_channel_metadata(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SendChannelMessage(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::send_channel_message::send_channel_message(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::HidePublicChannelMessage(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::hide_public_channel_message::hide_public_channel_message(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::MutePublicKey(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::mute_publickey::mute_publickey(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::BroadcastEvents(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::broadcast_events::broadcast_events(
                Some(args.nsec.expect("")),
                args.relays,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::CreateBadge(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::create_badge::create_badge(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::AwardBadge(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::award_badge::award_badge(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::ProfileBadges(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::profile_badges::set_profile_badges(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::CustomEvent(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::custom_event::create_custom_event(
                args.nsec,
                args.relays,
                args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SetUserStatus(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
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
                let gnostr_subcommands = gnostr::gnostr::GnostrSubCommands::default();
                let _ = sub_commands::tui::tui(&gnostr_subcommands).await;
            };
            Ok(())
        }
    }
}
