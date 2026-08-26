use gnostr::p2p::lookup::{LookupClient, LookupError, Peer};
use clap::Parser;
use libp2p::{Multiaddr, PeerId};
use std::str::FromStr;
use gnostr::p2p::network_config::Network;
use futures::future::{FutureExt, TryFutureExt};

#[derive(Debug, Parser)]
#[command(name = "gnostr-lookup", about = "Lookup libp2p nodes.")]
enum Opt {
    /// Lookup peer by its address.
    Direct {
        /// Address of the peer.
        #[arg(long)]
        address: Multiaddr,
    },
    /// Lookup peer by its ID via the Kademlia DHT.
    Dht {
        /// ID of the peer.
        #[arg(long)]
        peer_id: PeerId,
        /// Network of the peer.
        #[arg(
        long,
        value_enum,
        default_value = "ipfs",
        )]
        network: Network,
    },
}

#[async_std::main]
async fn main() {
    env_logger::init();
    let opt = Opt::parse();

    let lookup = match opt {
        Opt::Dht { peer_id, network } => {
            let client = LookupClient::new(Some(network));
            client.lookup_on_dht(peer_id).boxed()
        }
        Opt::Direct { address } => {
            let client = LookupClient::new(None);
            client.lookup_directly(address).boxed()
        }
    };

    let timed_lookup = async_std::future::timeout(std::time::Duration::from_secs(20), lookup)
        .map_err(|_| LookupError::Timeout);

    match timed_lookup.await {
        Ok(Ok(peer)) => {
            print!("{peer}");
        }
        Ok(Err(e)) | Err(e) => {
            log::error!("Lookup failed: {e:?}.");
            std::process::exit(1);
        }
    }
}
