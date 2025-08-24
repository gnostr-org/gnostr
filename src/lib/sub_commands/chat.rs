#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

use crate::cli::ChatSubCommands;
use anyhow::Result;
use serde::ser::StdError;

use nostr_sdk_0_37_0::Keys;
use tracing::{debug, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub async fn chat(
    key: &String,
    sub_command_args: &mut ChatSubCommands,
) -> Result<(), Box<dyn StdError>> {
    run(key, sub_command_args).await?;
    Ok(())
}

pub async fn run(
    key: &String,
    sub_command_args: &mut ChatSubCommands,
) -> Result<(), Box<dyn StdError>> {
    //let &(mut sub_command_args) = sub_command_args;
    if let Some(name) = sub_command_args.name.clone() {
        use std::env;
        env::set_var("USER", &name);
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
    //print!("{:?}", &sub_command_args);

    if (sub_command_args.debug || sub_command_args.trace) && sub_command_args.nsec.clone().is_some()
    {
        let keys = Keys::parse(sub_command_args.nsec.clone().unwrap().clone()).unwrap();
        debug!(
            "{{\"private_key\":\"{}\"}}",
            keys.secret_key().display_secret()
        );
        debug!("{{\"public_key\":\"{}\"}}", keys.public_key());
    }

    let chat = crate::chat::chat(key, sub_command_args).await;

    Ok(())
}
