#![cfg_attr(not(test), warn(clippy::pedantic))]
#![cfg_attr(not(test), warn(clippy::expect_used))]

//use crate::sub_commands::chat::Utc;

//use crate::p2p::chat::p2p::evt_loop; //migrate carefully
use crate::p2p::chat::*;
 //migrate carefully
use anyhow::Result;



use nostr_sdk_0_37_0::Keys;

use tracing::{debug, Level};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

pub async fn chat(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    run(sub_command_args).await?;
    Ok(())
}

pub async fn run(sub_command_args: &ChatSubCommands) -> Result<(), anyhow::Error> {
    let sub_command_args = sub_command_args;
    if let Some(name) = sub_command_args.name.clone() {
        use std::env;
        env::set_var("USER", &name);
    };

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
