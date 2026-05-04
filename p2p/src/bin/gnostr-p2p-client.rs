use clap::Parser;
use gnostr_p2p::cli;
use gnostr_p2p::lookup::LookupClient;
use libp2p::PeerId;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::init_tracing();

    let args = cli::LookupOpts::parse();
    let client = LookupClient::new(args.network);

    let peer = match (args.multiaddr, args.peer) {
        (Some(addr), _) => client.lookup_directly(addr).await?,
        (None, Some(peer)) => {
            let peer_id: PeerId = peer.parse()?;
            client.lookup_on_dht(peer_id).await?
        }
        (None, None) => unreachable!("clap enforces lookup-target"),
    };

    println!("{peer}");
    Ok(())
}
