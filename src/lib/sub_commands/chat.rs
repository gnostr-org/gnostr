#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

//use crate::sub_commands::chat::Utc;

//migrate carefully
use anyhow::Result;
use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

//use crate::p2p::chat::p2p::evt_loop; //migrate carefully
use crate::p2p::chat::ChatSubCommands;
use crate::types::PrivateKey;

/// chat
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn chat(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    run(sub_command_args).await?;
    Ok(())
}

/// run
///
/// # Panics
///
/// Panics if the tracing directive cannot be parsed.
///
/// # Errors
///
/// This function will return an error if the command fails.
pub async fn run(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    // Determine the username to set for the USER environment variable
    let username_to_set: Option<String> = if let Some(name) = sub_command_args.name.clone() {
        // If --name is provided, use it.
        Some(name)
    } else if let Some(nsec_hex) = sub_command_args.nsec.clone() {
        // If --name is not provided, but --nsec is, try to derive the public key
        // fingerprint.
        match PrivateKey::try_from_hex_string(&nsec_hex) {
            Ok(private_key) => {
                let public_key = private_key.public_key();
                Some(public_key.as_hex_string().chars().take(8).collect()) // Use first 8 chars of public key hex as fingerprint
            }
            Err(e) => {
                // Log a warning if nsec is provided but invalid, but don't crash.
                // The USER env var won't be set from nsec in this case.
                tracing::warn!(
                    "Could not derive public key from --nsec due to error: {}. USER env var will not be set from nsec.",
                    e
                );
                None
            }
        }
    } else {
        // Neither --name nor --nsec was provided.
        None
    };

    // Only set the USER environment variable if a username was successfully
    // determined.
    if let Some(user_name) = username_to_set {
        if !user_name.is_empty() {
            // Ensure we don't set it to an empty string if derivation resulted in one
            // (though unlikely with hex)
            use std::env;
            env::set_var("USER", &user_name);
            tracing::debug!("USER environment variable set to: {}", user_name);
        }
    }

    let level = if sub_command_args.debug {
        Level::DEBUG
    } else if sub_command_args.trace {
        Level::TRACE
    } else if sub_command_args.info {
        Level::INFO
    } else {
        Level::WARN
    };
    //TODO chat specific filters
    let filter = EnvFilter::default()
        .add_directive(level.into())
        .add_directive("nostr_sdk=off".parse().unwrap())
        .add_directive("nostr_sdk::relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::client=off".parse().unwrap())
        .add_directive("nostr_sdk::client::handler=off".parse().unwrap())
        .add_directive("nostr_relay_pool=off".parse().unwrap())
        .add_directive("nostr_sdk::relay::connection=off".parse().unwrap())
        .add_directive("gnostr::chat::p2p=off".parse().unwrap())
        .add_directive("gnostr::message=off".parse().unwrap())
        .add_directive("gnostr::nostr_proto=off".parse().unwrap())
        .add_directive("libp2p_mdns::behaviour::iface=off".parse().unwrap())
        .add_directive("libp2p_gossipsub::behaviour=off".parse().unwrap());

    //    let subscriber = Registry::default()
    //        .with(fmt::layer().with_writer(std::io::stdout))
    //        .with(filter);

    let subscriber = Registry::default()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                //.with_timer(fmt::time::Utc::rfc_3339()) // Corrected line
                .with_thread_ids(true),
        )
        .with(filter);

    let _ = subscriber.try_init();
    tracing::trace!("\n{:?}\n", &sub_command_args);
    tracing::debug!("\n{:?}\n", &sub_command_args);
    tracing::info!("\n{:?}\n", &sub_command_args);

    crate::p2p::chat::chat(sub_command_args).await?;

    Ok(())
}
