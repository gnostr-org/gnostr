use clap::Parser;
use libp2p::{Multiaddr, PeerId};
use gnostr_p2p::{args::Args, lookup::LookupClient};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let args = Args::parse();
    let client = LookupClient::new(args.network);

    let peer = match (args.multiaddr, args.peer) {
        (Some(addr), _) => client.lookup_directly(addr).await?,
        (None, Some(peer)) => {
            let peer_id: PeerId = peer.parse()?;
            client.lookup_on_dht(peer_id).await?
        }
        (None, None) => anyhow::bail!("provide either --multiaddr or --peer"),
    };

    println!("{peer}");
    Ok(())
}
