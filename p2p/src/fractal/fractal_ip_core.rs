//! Recursive packet framing and parity helpers for the `perfect_ip` protocol.
//!
//! `perfect_ip` is a repair-oriented tree protocol. It does not try to be a
//! general transport; instead it turns a byte buffer into a deterministic tree
//! of MTU-safe data slices and parity slices that can be reconstructed when a
//! sibling packet is missing.
//!
//! ## Packet model
//!
//! - [`process_slice`] recursively splits payloads until each emitted leaf fits
//!   within the packet budget.
//! - Internal nodes emit a left child, right child, and parity frame.
//! - Packet ids form a stable recursive path such as `ROOT.1.0.P`.
//! - [`packetize`] finalizes the batch by filling `total_packets` into every
//!   packet header.
//!
//! ## Wire model
//!
//! On the wire, packet slices are exchanged as JSON-encoded [`ProtocolSlice`]
//! values over libp2p request/response. The repair protocol id is
//! `/fractal/repair/1.0.0/json`, and the behaviour is exposed through
//! [`FractalBehaviour`] and [`run_fractal_engine`].
//!
//! ## Repair model
//!
//! The parity scheme is XOR-based. Given one sibling payload and the matching
//! parity frame, [`recover_missing_data`] and [`recover_missing_slice`] can
//! rebuild the missing sibling payload. [`IntegrityManager`] tracks the
//! manifest, records received slices, persists the partial state to disk, and
//! verifies that parity frames still match their children.
//!
//! The layout intentionally mirrors the packet-tree sketch in the
//! `perfect_ip.rs` gist.

use std::error::Error;

use futures::StreamExt;
use libp2p::{
    request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    StreamProtocol,
};

pub use gnostr_types::{
    calculate_parity, generate_manifest, list_directory, packetize, process_slice,
    reconstruct_directory, reconstruct_payload, recover_missing_data, recover_missing_slice,
    run_directory_walk, sha256_hex, summarize_packets, DirectoryWalk, Header, IntegrityManager,
    MTU_PAYLOAD, PacketBatch, ProtocolSlice,
};

/// libp2p repair behaviour for exchanging packet slices.
///
/// This is a small request/response behaviour that accepts one
/// [`ProtocolSlice`] and returns one [`ProtocolSlice`]. It uses JSON on the
/// wire so the packets can be inspected or proxied as text if needed.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "FractalBehaviourEvent")]
pub struct FractalBehaviour {
    /// JSON request/response channel for packet slice repair.
    pub repair_rpc: request_response::json::Behaviour<ProtocolSlice, ProtocolSlice>,
}

/// Behaviour events emitted by the repair RPC layer.
///
/// The enum stays small because the behaviour exposes one request/response
/// flow.
#[derive(Debug)]
pub enum FractalBehaviourEvent {
    /// A JSON request/response protocol event.
    RepairRpc(request_response::Event<ProtocolSlice, ProtocolSlice>),
}

impl From<request_response::Event<ProtocolSlice, ProtocolSlice>> for FractalBehaviourEvent {
    fn from(event: request_response::Event<ProtocolSlice, ProtocolSlice>) -> Self {
        Self::RepairRpc(event)
    }
}

/// Build the repair swarm used to exchange packet slices with peers.
///
/// The swarm uses the current identity, Tokio, and QUIC transport so the demo
/// can listen on a simple localhost multiaddr without extra transport setup.
pub async fn build_fractal_swarm(
    local_key: libp2p::identity::Keypair,
) -> Result<Swarm<FractalBehaviour>, Box<dyn Error + Send + Sync>> {
    #[cfg(target_os = "tvos")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_behaviour(|_| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::json::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0/json"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
            })
        })?
        .build();

    #[cfg(not(target_os = "tvos"))]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::json::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0/json"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
            })
        })?
        .build();

    Ok(swarm)
}

/// Start the repair swarm and forward incoming repair requests to the packet map.
///
/// This is the runtime repair loop for the protocol. It listens for
/// `ProtocolSlice` requests over JSON, looks up the requested id in
/// [`IntegrityManager::received_slices`], and replies with the stored slice or
/// an empty placeholder when the packet is absent.
pub async fn run_fractal_engine(
    local_key: libp2p::identity::Keypair,
    listen_address: libp2p::Multiaddr,
    manager: IntegrityManager,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut swarm = build_fractal_swarm(local_key).await?;
    swarm.listen_on(listen_address)?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                if let request_response::Event::Message { message, .. } = event {
                    if let request_response::Message::Request {
                        request, channel, ..
                    } = message
                    {
                        let response = manager
                            .received_slices
                            .get(&request.id)
                            .cloned()
                            .unwrap_or_else(|| ProtocolSlice {
                                id: request.id.clone(),
                                header: request.header.clone(),
                                data: Vec::new(),
                                is_parity: false,
                            });
                        let _ = swarm.behaviour_mut().repair_rpc.send_response(channel, response);
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair_from_seed;
    use libp2p::Multiaddr;

    #[tokio::test]
    async fn build_fractal_swarm_accepts_quic_listen_address() {
        let keypair = keypair_from_seed(Some(
            gnostr_asyncgit::default_gnostr_private_key_hex(),
        ));
        let mut swarm = build_fractal_swarm(keypair).await.expect("swarm");
        let addr: Multiaddr = "/ip4/127.0.0.1/udp/0/quic-v1"
            .parse()
            .expect("quic multiaddr");

        swarm.listen_on(addr).expect("quic listen");
    }
}
