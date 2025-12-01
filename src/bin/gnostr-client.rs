use clap::Parser;
use gnostr::types::{NostrClient, UncheckedUrl};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "wss://relay.damus.io")]
    relay_url: String,
    #[arg(short, long, default_value = "test")]
    channel_id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (tx, mut rx) = mpsc::channel(100);

    let mut client = NostrClient::new(tx);

    let relay_url = UncheckedUrl(args.relay_url);
    client.connect_relay(relay_url).await?;

    client.subscribe_to_channel(args.channel_id).await;

    println!("Listening for events...");

    while let Some(event) = rx.recv().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}