use clap::{Parser /*, Subcommand*/};
use gnostr::cli::{get_app_cache_path, setup_logging, GnostrCli, GnostrCommands};
use gnostr::sub_commands;
use gnostr_asyncgit::sync::RepoPath;
use gnostr::blockheight;
use gnostr::blockhash;
use gnostr::weeble;
use gnostr::wobble;
use sha2::{Digest, Sha256};
use std::env;
use tracing::{debug, /*info, */trace};
use tracing_core::metadata::LevelFilter;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};
//use tracing_subscriber::util::SubscriberInitExt;

use anyhow::anyhow; // Import the anyhow macro

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env::set_var("WEEBLE", "0");
    env::set_var("BLOCKHEIGHT", "0");
    env::set_var("WOBBLE", "0");
    let mut gnostr_cli_args: GnostrCli = GnostrCli::parse();

    let app_cache = get_app_cache_path();
    //if gnostr_cli_args.logging {
    //    let logging = setup_logging();
    //    debug!("{:?}", logging);
    //};
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

    let filter = EnvFilter::default()
        .add_directive(level.into())
        .add_directive("hickory_proto=off".parse().unwrap())
        .add_directive("hickory_proto::rr=off".parse().unwrap())
        .add_directive("hickory_proto::rr::record_data=off".parse().unwrap())
        .add_directive("libp2p_mdns=off".parse().unwrap())
        //.add_directive("gnostr::p2p=off".parse().unwrap())
        .add_directive("libp2p=off".parse().unwrap())
        .add_directive("mio=off".parse().unwrap())
        //.add_directive("gnostr::p2p::chat=off".parse().unwrap())
        //.add_directive("gnostr::p2p::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap());

    if gnostr_cli_args.logging {
        let logging = setup_logging();
        debug!("{:?}", logging);
	} else {
        let subscriber = Registry::default()
            .with(fmt::layer().with_writer(std::io::stdout))
            .with(filter);
        let _ = subscriber.try_init();
    };

    let env_args: Vec<String> = env::args().collect();
    for arg in &env_args {
        debug!("40:arg={:?}", arg);
    }

    if env_args.contains(&String::from("--gitdir")) {
        debug!("main::72:The --gitdir argument was found!");
    } else {
        debug!("main::72:The --gitdir argument was not found.");
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
            debug!("main:91:The --gitdir value is: {}", value);
            let repo_path: RepoPath = RepoPath::from(gitdir_value.clone().unwrap().as_str());
            debug!("main:93:repo_path={:?}", repo_path);
            // Convert the RepoPath to an OsStr reference
            let path_os_str = repo_path.as_path().as_os_str();

            // Now set the environment variable
            env::set_var("GNOSTR_GITDIR", path_os_str);
        }
        None => {
			//OBJECTIVE to let sub services like "gnostr chat" to run outside of repos
			//TODO check if use home dir has $HOME/gnostr if not then create
            //THEN env::set_var("GNOSTR_GITDIR", $HOME/.gnostr);
			debug!("72:The --gitdir argument was not found or has no value.")
		},
    }

    //let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
    //tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    trace!("{:?}", app_cache);

    // These if statements don't return anything, which is fine as long as the match statement returns Result.
    // if gnostr_cli_args.workdir.is_some() {};
    // if gnostr_cli_args.directory.is_some() {};
    if gnostr_cli_args.hash.is_some() {
        //not none
        if let Some(ref input_string) = gnostr_cli_args.hash {
            let mut hasher = Sha256::new();
            hasher.update(input_string.as_bytes());
            let result = hasher.finalize();
            //Usage: gnostr --hash <string>
            //Usage: gnostr --debug --hash <string>
            if env_args.len() >= 3 && env_args.len() <= 4
            /*--debug, --trace, --info, etc...*/
            {
                print!("{:x}", result);
                std::process::exit(0); // Exits the program, so no need to return Ok(())
            }
            gnostr_cli_args.nsec = format!("{:x}", result).into();
        }
    }
    if gnostr_cli_args.weeble {
        let result = weeble::weeble();
        print!("{}", result.unwrap());
        std::process::exit(0);
    }
    if gnostr_cli_args.wobble {
        let result = wobble::wobble();
        print!("{}", result.unwrap());
        std::process::exit(0);
    }
    if gnostr_cli_args.blockheight {
        let result = blockheight::blockheight();
        print!("{}", result.unwrap());
        std::process::exit(0);
    }
    if gnostr_cli_args.blockhash {
        let result = blockhash::blockhash().unwrap();
        env::set_var("BLOCKHASH", &result);
        print!("{}", result);
        std::process::exit(0);
    }

    // Post event
    match &gnostr_cli_args.command {
        Some(GnostrCommands::Chat(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::chat::chat(
                &sub_command_args.clone(),
            )
            .await.map_err(|e| anyhow!("Error in chat subcommand: {}", e))
        }
        Some(GnostrCommands::Legit(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::legit::legit(sub_command_args).await.map_err(|e| anyhow!("Error in legit subcommand: {}", e))
        }
        Some(GnostrCommands::Ngit(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::ngit::ngit(sub_command_args).await.map_err(|e| anyhow!("Error in ngit subcommand: {}", e))
        }
        Some(GnostrCommands::Query(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::query::launch(sub_command_args).await.map_err(|e| anyhow!("Error in query subcommand: {}", e))
        }
        Some(GnostrCommands::SetMetadata(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::set_metadata::set_metadata(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in set_metadata subcommand: {}", e))
        }
        Some(GnostrCommands::Note(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::note::broadcast_textnote(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in note subcommand: {}", e))
        }
        Some(GnostrCommands::PublishContactListCsv(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::publish_contactlist_csv::publish_contact_list_from_csv_file(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in publish_contact_list_csv subcommand: {}", e))
        }
        Some(GnostrCommands::DeleteEvent(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::delete_event::delete(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in delete_event subcommand: {}", e))
        }
        Some(GnostrCommands::DeleteProfile(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::delete_profile::delete(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in delete_profile subcommand: {}", e))
        }
        Some(GnostrCommands::React(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::react::react_to_event(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in react subcommand: {}", e))
        }
        Some(GnostrCommands::ListEvents(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::list_events::list_events(gnostr_cli_args.relays, sub_command_args).await.map_err(|e| anyhow!("Error in list_events subcommand: {}", e))
        }
        Some(GnostrCommands::GenerateKeypair(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::generate_keypair::get_new_keypair(sub_command_args).await.map_err(|e| anyhow!("Error in generate_keypair subcommand: {}", e))
        }
        Some(GnostrCommands::ConvertKey(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::convert_key::convert_key(sub_command_args).await.map_err(|e| anyhow!("Error in convert_key subcommand: {}", e))
        }
        Some(GnostrCommands::Vanity(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::vanity::vanity(sub_command_args).await.map_err(|e| anyhow!("Error in vanity subcommand: {}", e))
        }
        Some(GnostrCommands::CreatePublicChannel(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::create_public_channel::create_public_channel(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in create_public_channel subcommand: {}", e))
        }
        Some(GnostrCommands::SetChannelMetadata(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::set_channel_metadata::set_channel_metadata(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in set_channel_metadata subcommand: {}", e))
        }
        Some(GnostrCommands::SendChannelMessage(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::send_channel_message::send_channel_message(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in send_channel_message subcommand: {}", e))
        }
        Some(GnostrCommands::HidePublicChannelMessage(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::hide_public_channel_message::hide_public_channel_message(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in hide_public_channel_message subcommand: {}", e))
        }
        Some(GnostrCommands::MutePublicKey(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::mute_publickey::mute_publickey(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in mute_publickey subcommand: {}", e))
        }
        Some(GnostrCommands::BroadcastEvents(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::broadcast_events::broadcast_events(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in broadcast_events subcommand: {}", e))
        }
        Some(GnostrCommands::CreateBadge(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::create_badge::create_badge(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in create_badge subcommand: {}", e))
        }
        Some(GnostrCommands::AwardBadge(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::award_badge::award_badge(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in award_badge subcommand: {}", e))
        }
        Some(GnostrCommands::ProfileBadges(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::profile_badges::set_profile_badges(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in profile_badges subcommand: {}", e))
        }
        Some(GnostrCommands::CustomEvent(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::custom_event::create_custom_event(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in custom_event subcommand: {}", e))
        }
        Some(GnostrCommands::SetUserStatus(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::user_status::set_user_status(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in set_user_status subcommand: {}", e))
        }
        Some(GnostrCommands::Tui(sub_command_args)) => {
            debug!("main:318:sub_command_args:{:?}", sub_command_args.clone());
            let mut sub_command_args_mut = sub_command_args.clone();
            let result: anyhow::Result<()>;// = Ok(()); // Initialize result to Ok

            // Check if GNOSTR_GITDIR environment variable is set
            if let Ok(gitdir_env_value) = env::var("GNOSTR_GITDIR") {
                eprintln!("333:The GNOSTR_GITDIR environment variable is set to: {}", gitdir_env_value);
                // Check if --gitdir argument was provided (from command line args)
                if let Some(git_dir_value) = gitdir_value { // Assuming gitdir_value is from command line args parsing
                    eprintln!("339:OVERRIDE!! The git directory is: {:?}", git_dir_value);
                    let gitdir_string = gitdir_env_value.to_string();
                    debug!("342:OVERRIDE!! The git directory is: {:?}", gitdir_string.clone());
                    sub_command_args_mut.gitdir = Some(RepoPath::from(gitdir_string.as_str()));
                    // Call tui and map error, then assign to result
                    result = sub_commands::tui::tui(sub_command_args_mut.clone(), &gnostr_cli_args).await.map_err(|e| anyhow!("Error in tui subcommand: {}", e));
                } else {
                    // If gitdir_value is None, we don't override. The result remains Ok(()).
                    result = Ok(()); // Explicitly set for clarity
                }
            } else {
                // GNOSTR_GITDIR environment variable is not set.
                debug!("354:The GNOSTR_GITDIR environment variable is not set.");
                // Call tui with original args and map error, then assign to result
                result = sub_commands::tui::tui(sub_command_args.clone(), &gnostr_cli_args).await.map_err(|e| anyhow!("Error in tui subcommand: {}", e));
            }
            result // Return the accumulated result
        }
        Some(GnostrCommands::FetchById(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::fetch_by_id::run_fetch_by_id(sub_command_args)
                .await
                .map_err(|e| anyhow!("Error in fetch_by_id subcommand: {}", e))
        },
        Some(GnostrCommands::Relay(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            let local_set = tokio::task::LocalSet::new();
            local_set.run_until(async move {
                sub_commands::relay::relay(sub_command_args.clone()).await
            }).await.map_err(|e| anyhow!("Error in relay subcommand: {}", e))
        },
        Some(GnostrCommands::Sniper(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::sniper::run_sniper(sub_command_args.clone())
                .await
                .map_err(|e| anyhow!("Error in sniper subcommand: {}", e))
        },
        Some(GnostrCommands::Git(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::git::git(sub_command_args).await.map_err(|e| anyhow!("Error in git subcommand: {}", e))
        },
        Some(GnostrCommands::Nip34(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::nip34::launch(
                gnostr_cli_args.nsec,
                gnostr_cli_args.relays,
                gnostr_cli_args.difficulty_target,
                sub_command_args,
            )
            .await.map_err(|e| anyhow!("Error in nip34 subcommand: {}", e))
        },
        Some(GnostrCommands::Bech32ToAny(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::bech32_to_any::bech32_to_any(sub_command_args).map_err(|e| anyhow!("Error in bech32_to_any subcommand: {}", e))
        },
        Some(GnostrCommands::PrivkeyToBech32(sub_command_args)) => {
            debug!("sub_command_args:{:?}", sub_command_args);
            sub_commands::privkey_to_bech32::privkey_to_bech32(sub_command_args).map_err(|e| anyhow!("Error in privkey_to_bech32 subcommand: {}", e))
        },
        None => {
            // TODO handle more scenarios
            // Call tui with default commands and propagate its result
            sub_commands::tui::tui(gnostr::core::GnostrSubCommands::default(), &gnostr_cli_args).await.map_err(|e| anyhow!("Error in default tui subcommand: {}", e))
        }
    }
}
