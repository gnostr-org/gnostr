use clap::{Parser, Subcommand};
use gnostr::types::{
    EventKind, KeySigner, NostrClient, PreEventV3, PrivateKey, Signer, UncheckedUrl, Unixtime, EventV3
};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "wss://relay.damus.io")]
    relay_url: String,
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Publish a text note
    Publish {
        #[arg(short, long)]
        content: String,
    },
    /// Subscribe to a channel
    Channel {
        #[arg(short, long, default_value = "test")]
        id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (tx, mut rx) = mpsc::channel(100);

    let mut client = NostrClient::new(tx);

    let relay_url = UncheckedUrl(args.relay_url);
    client.connect_relay(relay_url).await?;

    match args.command {
        SubCommand::Publish { content } => {
            println!("Publishing: {}", content);
            let signer =
                KeySigner::from_private_key(PrivateKey::generate(), "", 1).unwrap();
            let pubkey = signer.public_key();
            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::TextNote,
                tags: vec![],
                content,
            };
            let id = preevent.hash()?;
            let sig = signer.sign_id(id)?;
            let event = EventV3 {
                id,
                pubkey: preevent.pubkey,
                created_at: preevent.created_at,
                kind: preevent.kind,
                tags: preevent.tags,
                content: preevent.content,
                sig,
            };
            client.send_event(event).await?;
        }
        SubCommand::Channel { id } => {
            client.subscribe_to_channel(id).await;
        }
    }

    println!("Listening for events...");

    while let Some(event) = rx.recv().await {
        println!("Received event: {:?}", event);
    }

    Ok(())
}