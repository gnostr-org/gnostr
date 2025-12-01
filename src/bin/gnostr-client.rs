use gnostr::types::{NostrClient, UncheckedUrl};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (tx, mut rx) = mpsc::channel(100);

    let mut client = NostrClient::new(tx);

    let relay_url = UncheckedUrl("wss://relay.damus.io".to_string());
    client.connect_relay(relay_url).await?;

    let channel_id = "test".to_string();
    client.subscribe_to_channel(channel_id).await;

    println!("Listening for events...");

    while let Some(event) = rx.recv().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}