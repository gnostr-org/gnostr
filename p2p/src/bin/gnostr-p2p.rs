use futures::StreamExt;

use clap::Parser;
use gnostr_p2p::{cli, command_handler, event_handler, swarm_builder::build_swarm};
use libp2p::kad;
use tokio::io::AsyncBufReadExt;

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Run the main gnostr p2p daemon.",
    long_about = "Run the libp2p swarm used by gnostr chat and related services.\n\nThis daemon owns discovery, DHT bootstrap, gossipsub traffic, relay handling, and stdin command processing.",
    help_template = cli::HELP_TEMPLATE,
    next_line_help = true,
    disable_help_subcommand = true,
    after_help = "Use the sibling relay and rendezvous binaries when you want a dedicated infrastructure role."
)]
struct Opt {
    #[command(flatten)]
    node: cli::NodeOpts,

    #[command(flatten)]
    network: cli::NetworkOpts,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::init_tracing();

    let opt = Opt::parse();
    let keypair = cli::keypair_from_seed(opt.node.secret_key_seed);
    let mut swarm = build_swarm(keypair).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));

    if let Some(network) = opt.network.network {
        for (addr, peer_id) in network.bootnodes() {
            swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
        }
    }

    cli::listen_default_addresses(
        &mut swarm,
        opt.node.listen_address,
        opt.node.port,
        opt.node.use_ipv6,
    )?;

    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let Some(line) = line? else { break; };
                if line.trim().is_empty() {
                    continue;
                }
                command_handler::handle_input_line(&mut swarm, line).await;
            }
            event = swarm.select_next_some() => {
                event_handler::handle_swarm_event(&mut swarm, event).await;
            }
        }
    }
    Ok(())
}
