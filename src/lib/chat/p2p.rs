use futures::stream::StreamExt;
use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux};
use std::error::Error;
use std::time::Duration;
use tokio::{io, select};
use tracing::{debug, warn};

use ureq::Agent;

use crate::chat::msg::{Msg, MsgKind};
//use tokio::task;

//const TOPIC: &str = "gnostr";
/// MyBehaviour
// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
}

/// async_prompt
pub async fn async_prompt(mempool_url: String) -> String {
    let s = tokio::spawn(async move {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(10))
            .timeout_write(Duration::from_secs(10))
            .build();
        let body: String = agent
            .get(&mempool_url)
            .call()
            .expect("")
            .into_string()
            .expect("mempool_url:body:into_string:fail!");

        body
    });

    s.await.unwrap()
}

///// fetch_data_async
//async fn fetch_data_async<T>(url: String) -> Result<ureq::Response<T>, ureq::Error> {
//    task::spawn_blocking(move || {
//        let response = ureq::get(&url).call();
//        response
//    })
//    .await
//    .unwrap() // Handle potential join errors
//}

/// evt_loop
pub async fn evt_loop(
    mut send: tokio::sync::mpsc::Receiver<Msg>,
    recv: tokio::sync::mpsc::Sender<Msg>,
    topic: gossipsub::IdentTopic,
) -> Result<(), Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            // NOTE: To content-address message,
            // we can take the hash of message
            // and use it as an ID.
            // This is used to deduplicate messages.
            //
            let message_id_fn = |message: &gossipsub::Message| {
                use std::hash::DefaultHasher;
                use std::hash::Hash;
                use std::hash::Hasher;
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            };

            // Set a custom gossipsub configuration
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .heartbeat_interval(Duration::from_secs(10))
                // This is set to aid debugging by not cluttering the log space
                .validation_mode(gossipsub::ValidationMode::Strict)
                // This sets the kind of message validation.
                // The default is Strict (enforce message signing)
                // .message_id_fn(message_id_fn)
                // content-address messages.
                // No two messages of the same content will be propagated.
                .build()
                .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?;
            // Temporary hack because `build` does not return a proper `std::error::Error`.

            // build a gossipsub network behaviour
            let gossipsub = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )?;

            let mdns = libp2p::mdns::tokio::Behaviour::new(
                libp2p::mdns::Config::default(),
                key.public().to_peer_id(),
            )?;
            Ok(MyBehaviour { gossipsub, mdns })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    // subscribes to our topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;

    // Listen on all interfaces and whatever port the OS assigns
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    debug!("Enter messages via STDIN and they will be sent to connected peers using Gossipsub");

    // Kick it off
    // Kick it off
    loop {
        select! {
            Some(m) = send.recv() => {
                if let Err(e) = swarm
                    .behaviour_mut().gossipsub
                    .publish(topic.clone(), serde_json::to_vec(&m)?) {
                    debug!("Publish error: {e:?}");
                    let m = Msg::default().set_content(format!("publish error: {e:?}")).set_kind(MsgKind::System);
                    recv.send(m).await?;
                }
            }
            event = swarm.select_next_some() => match event {
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discovered a new peer: {peer_id}");
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                    // let m = Msg::default().set_content(format!("discovered new peer: {peer_id}")).set_kind(MsgKind::System);
                        // recv.send(m).await?;
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        debug!("mDNS discover peer has expired: {peer_id}");
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        // let m = Msg::default().set_content(format!("peer expired: {peer_id}")).set_kind(MsgKind::System);
                        // recv.send(m).await?;
                    }
                },
                SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    debug!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    );
                    match serde_json::from_slice::<Msg>(&message.data) {
                        Ok(msg) => {
                            recv.send(msg).await?;
                        },
                        Err(e) => {
                            warn!("Error deserializing message: {e:?}");
                            let m = Msg::default().set_content(format!("Error deserializing message: {e:?}")).set_kind(MsgKind::System);
                            recv.send(m).await?;
                        }
                    }
                },
                SwarmEvent::NewListenAddr { address, .. } => {
                    debug!("Local node is listening on {address}");
                }
                _ => {}
            }
        }
    }
}
