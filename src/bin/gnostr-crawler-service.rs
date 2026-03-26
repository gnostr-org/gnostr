use gnostr::crawler::relay_manager::{ActiveRelayList, RelayManager};
use gnostr::crawler::processor::Processor;
use gnostr::crawler::processor::{BOOTSTRAP_RELAY0, BOOTSTRAP_RELAY1, BOOTSTRAP_RELAY2, BOOTSTRAP_RELAY3};
use nostr_0_34_1::Keys;
use std::str::FromStr;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let app_keys = Keys::from_str("nsec1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sv7rt36").unwrap();
    let processor = Processor::new();
    let active_relay_list = ActiveRelayList::new();
    let (update_sender, mut update_receiver) = mpsc::channel(100);

    let mut relay_manager = RelayManager::new(
        app_keys,
        processor,
        active_relay_list.clone(), // Clone for RelayManager to own
        update_sender,
    );

    let bootstrap_relays = vec![
        BOOTSTRAP_RELAY0,
        BOOTSTRAP_RELAY1,
        BOOTSTRAP_RELAY2,
        BOOTSTRAP_RELAY3,
    ];

    // Run the RelayManager in a background task
    tokio::spawn(async move {
        if let Err(e) = relay_manager.run(bootstrap_relays).await {
            eprintln!("RelayManager encountered an error: {}", e);
        }
    });

    // Main loop to receive and print updates from the active relay list
    println!("Listening for active relay updates...");
    while let Some(active_relays) = update_receiver.recv().await {
        println!("Current active relays:");
        for relay_url in active_relays {
            println!("- {}", relay_url);
        }
    }

    Ok(())
}