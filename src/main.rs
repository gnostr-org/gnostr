use anyhow::Result;
use clap::{Parser /*, Subcommand*/};
use gnostr::cli::*;
use gnostr::cli::{get_app_cache_path, setup_logging, GnostrCli, GnostrCommands};
use gnostr::sub_commands;
use gnostr_asyncgit::sync::RepoPath;
use sha2::{Digest, Sha256};
use std::env;
use tracing::{debug, trace};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::FmtSubscriber;

use serde::ser::StdError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn StdError>> {
    env::set_var("WEEBLE", "0");
    env::set_var("BLOCKHEIGHT", "0");
    env::set_var("WOBBLE", "0");
    let mut gnostr_cli_args: GnostrCli = GnostrCli::parse();

    let app_cache = get_app_cache_path();
    if gnostr_cli_args.logging {
        let logging = setup_logging();
        trace!("{:?}", logging);
    };
    let level = if gnostr_cli_args.debug {
        LevelFilter::DEBUG
    } else if gnostr_cli_args.trace {
        LevelFilter::TRACE
    } else if gnostr_cli_args.info {
        LevelFilter::INFO
    } else if gnostr_cli_args.warn {
        LevelFilter::WARN
    } else {
        LevelFilter::OFF
    };
    let env_args: Vec<String> = env::args().collect();
    for arg in &env_args {
        debug!("40:arg={:?}", arg);
    }

    if env_args.contains(&String::from("--gitdir")) {
        debug!("44:The --gitdir argument was found!");
    } else {
        debug!("46:The --gitdir argument was not found.");
    }

    let mut gitdir_value: Option<String> = None;

    for i in 0..env_args.len() {
        if env_args[i] == "--gitdir" {
            if i + 1 < env_args.len() {
                // We found --gitdir and there's a next argument
                gitdir_value = Some(env_args[i + 1].clone());
            }
            break; // We found what we're looking for, no need to continue the loop
        }
    }

    match gitdir_value.clone() {
        Some(value) => {
            println!("63:The --gitdir value is: {}", value);
            let repo_path: RepoPath = RepoPath::from(gitdir_value.clone().unwrap().as_str());
            debug!("main:73:repo_path={:?}", repo_path);
            // Convert the RepoPath to an OsStr reference
            let path_os_str = repo_path.as_path().as_os_str();

            // Now set the environment variable
            env::set_var("GNOSTR_GITDIR", path_os_str);
        }
        None => println!("63:The --gitdir argument was not found or has no value."),
    }

    let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    trace!("{:?}", app_cache);

    //if gnostr_cli_args.gitdir.is_some() {
    //    // Assuming 'args' and 'gitdir' are correctly defined elsewhere
    //    let repo_path: RepoPath = gnostr_cli_args.gitdir.clone().expect("");
    //    debug!("main:73:repo_path={:?}", repo_path);
    //    // Convert the RepoPath to an OsStr reference
    //    let path_os_str = repo_path.as_path().as_os_str();

    //    // Now set the environment variable
    //    env::set_var("GNOSTR_GITDIR", path_os_str);

    //    debug!("main:80:{:?}", gnostr_cli_args.gitdir.clone().expect(""));
    //    //env::set_var("GNOSTR_GITDIR", args.gitdir.clone().expect(""));
    //    debug!("82:{}", env::var("GNOSTR_GITDIR").unwrap().to_string());
    //    //replace gnostr tui --gitdir
    //    //std::process::exit(0);
    //}
    if gnostr_cli_args.workdir.is_some() {}
    if gnostr_cli_args.directory.is_some() {}
    if gnostr_cli_args.hash.is_some() {
        //not none
        if let Some(input_string) = gnostr_cli_args.hash {
            let mut hasher = Sha256::new();
            hasher.update(input_string.as_bytes());
            let result = hasher.finalize();
            //Usage: gnostr --hash <string>
            //Usage: gnostr --debug --hash <string>
            if env_args.len() >= 3 && env_args.len() <= 4
            /*--debug, --trace, --info, etc...*/
            {
                print!("{:x}", result);
                std::process::exit(0);
            }
            gnostr_cli_args.nsec = format!("{:x}", result).into();
        }
    }

    // Post event
    match &gnostr_cli_args.command {
        ////
        //Some(GnostrCommands::Tui(sub_command_args)) => {
        //    debug!("sub_command_args:{:?}", sub_command_args);
        //    sub_commands::tui::tui(sub_command_args).await
        //}
        ////
        Some(GnostrCommands::Chat(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::chat::chat(
                &gnostr_cli_args.nsec.unwrap().to_string(),
                &mut sub_command_args.clone(),
            )
            .await
        }
        Some(GnostrCommands::Legit(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::legit::legit(sub_command_args).await
        }
        Some(GnostrCommands::Ngit(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::ngit::ngit(sub_command_args).await
        }
        Some(GnostrCommands::SetMetadata(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            {
                sub_commands::set_metadata::set_metadata(
                    gnostr_cli_args.nsec,
                    gnostr_cli_args.relays,
                    gnostr_cli_args.difficulty_target,
                    sub_command_args,
                )
            }
            .await
        }
        Some(GnostrCommands::Note(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::note::broadcast_textnote(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::PublishContactListCsv(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::publish_contactlist_csv::publish_contact_list_from_csv_file(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::DeleteEvent(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::delete_event::delete(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::DeleteProfile(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::delete_profile::delete(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::React(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::react::react_to_event(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::ListEvents(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::list_events::list_events(gnostr_cli_args.relays, sub_command_args).await
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
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SetChannelMetadata(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::set_channel_metadata::set_channel_metadata(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SendChannelMessage(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::send_channel_message::send_channel_message(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::HidePublicChannelMessage(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::hide_public_channel_message::hide_public_channel_message(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::MutePublicKey(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::mute_publickey::mute_publickey(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::BroadcastEvents(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::broadcast_events::broadcast_events(
                Some(gnostr_cli_args.nsec.expect("")),
                gnostr_cli_args.relays,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::CreateBadge(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::create_badge::create_badge(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::AwardBadge(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::award_badge::award_badge(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::ProfileBadges(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::profile_badges::set_profile_badges(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::CustomEvent(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::custom_event::create_custom_event(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::SetUserStatus(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::user_status::set_user_status(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await
        }
        Some(GnostrCommands::Tui(sub_command_args)) => {
            debug!("main:318:sub_command_args:{:?}", sub_command_args.clone());
            let mut sub_command_args_mut = sub_command_args.clone();

            if let Ok(gitdir_env_value) = env::var("GNOSTR_GITDIR") {
                // This block is only entered if the env var is found and is valid.
                println!(
                    "324:The GNOSTR_GITDIR environment variable is set to: {}",
                    gitdir_env_value
                );

                //println!("{}", gnostr_cli_args.gitdir);
                //if let Some(git_dir_value) = sub_command_args_mut.clone().gitdir {
                if let Some(git_dir_value) = gitdir_value {
                    // You have the value!
                    println!("331:OVERRIDE!! The git directory is: {:?}", git_dir_value);

                    // Corrected line using .as_str():
                    let gitdir_string = format!("{}", gitdir_env_value);
                    println!(
                        "335:OVERRIDE!! The git directory is: {:?}",
                        gitdir_string.clone()
                    );
                    sub_command_args_mut.gitdir = Some(RepoPath::from(gitdir_string.as_str()));
                    //
                    sub_commands::tui::tui(sub_command_args_mut.clone()).await
                } else {
                    Ok({
                        println!("No git directory specified.");
                    })
                }
            } else {
                // This block is entered if the env var is not found.
                println!("350:The GNOSTR_GITDIR environment variable is not set.");
                sub_commands::tui::tui(sub_command_args.clone()).await
            }
        }
        None => {
            {
                let gnostr_subcommands = gnostr::gnostr::GnostrSubCommands::default();
                let _ = sub_commands::tui::tui(gnostr_subcommands).await;
            };
            Ok(())
        }
    }
}
