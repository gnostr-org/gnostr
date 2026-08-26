use std::{
    collections::HashMap,
    error::Error,
    path::PathBuf,
};

use futures::StreamExt;
use libp2p::{
    gossipsub,
    kad::{self, store::MemoryStore},
    request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    PeerId, StreamProtocol,
};
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, SeekFrom},
    sync::oneshot,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolSlice {
    pub block_id: String,
    pub id: String,
    pub offset: u64,
    pub data: Vec<u8>,
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "FractalBehaviourEvent")]
pub struct FractalBehaviour {
    pub repair_rpc: request_response::cbor::Behaviour<ProtocolSlice, ProtocolSlice>,
    pub kad: kad::Behaviour<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

#[derive(Debug)]
pub enum FractalBehaviourEvent {
    RepairRpc(request_response::Event<ProtocolSlice, ProtocolSlice>),
    Kad(kad::Event),
    Gossipsub(gossipsub::Event),
}

impl From<request_response::Event<ProtocolSlice, ProtocolSlice>> for FractalBehaviourEvent {
    fn from(event: request_response::Event<ProtocolSlice, ProtocolSlice>) -> Self {
        Self::RepairRpc(event)
    }
}

impl From<kad::Event> for FractalBehaviourEvent {
    fn from(event: kad::Event) -> Self {
        Self::Kad(event)
    }
}

impl From<gossipsub::Event> for FractalBehaviourEvent {
    fn from(event: gossipsub::Event) -> Self {
        Self::Gossipsub(event)
    }
}

pub struct IntegrityManager {
    file_path: PathBuf,
}

impl IntegrityManager {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
        }
    }

    pub async fn read_slice(&self, offset: u64, len: usize) -> std::io::Result<Vec<u8>> {
        let mut file = File::open(&self.file_path).await?;
        file.seek(SeekFrom::Start(offset)).await?;
        let mut buf = vec![0; len];
        file.read_exact(&mut buf).await?;
        Ok(buf)
    }
}

pub async fn build_fractal_swarm(
    local_key: libp2p::identity::Keypair,
) -> Result<Swarm<FractalBehaviour>, Box<dyn Error + Send + Sync>> {
    let peer_id = PeerId::from(local_key.public());

    #[cfg(target_os = "tvos")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_behaviour(|key| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::cbor::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
                kad: kad::Behaviour::new(peer_id, MemoryStore::new(peer_id)),
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::ConfigBuilder::default().build()?,
                )?,
            })
        })?
        .build();

    #[cfg(not(target_os = "tvos"))]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::cbor::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
                kad: kad::Behaviour::new(peer_id, MemoryStore::new(peer_id)),
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()),
                    gossipsub::ConfigBuilder::default().build()?,
                )?,
            })
        })?
        .build();

    Ok(swarm)
}

pub async fn run_fractal_engine(
    local_key: libp2p::identity::Keypair,
    listen_address: libp2p::Multiaddr,
    audio_topic: &str,
    managers: HashMap<String, IntegrityManager>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    run_fractal_engine_with_shutdown(local_key, listen_address, audio_topic, managers, None).await
}

pub async fn run_fractal_engine_with_shutdown(
    local_key: libp2p::identity::Keypair,
    listen_address: libp2p::Multiaddr,
    audio_topic: &str,
    managers: HashMap<String, IntegrityManager>,
    mut shutdown: Option<oneshot::Receiver<()>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut swarm = build_fractal_swarm(local_key).await?;
    let topic = gossipsub::IdentTopic::new(audio_topic);
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    swarm.listen_on(listen_address)?;

    loop {
        let event = match shutdown.as_mut() {
            Some(shutdown_rx) => tokio::select! {
                _ = shutdown_rx => {
                    break;
                }
                event = swarm.select_next_some() => event,
            },
            None => swarm.select_next_some().await,
        };

        match event {
            SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                if let request_response::Event::Message { message, .. } = event {
                    if let request_response::Message::Request {
                        request, channel, ..
                    } = message
                    {
                        let ProtocolSlice {
                            block_id,
                            id,
                            offset,
                            data: requested_bytes,
                        } = request;
                        let requested_len = requested_bytes.len();
                        let response = if let Some(mgr) = managers.get(&block_id) {
                            match mgr.read_slice(offset, requested_len).await {
                                Ok(data) => ProtocolSlice {
                                    block_id,
                                    id,
                                    offset,
                                    data,
                                },
                                Err(error) => {
                                    eprintln!(
                                        "[fractral] failed to read slice for {} at {}: {}",
                                        block_id, offset, error
                                    );
                                    ProtocolSlice {
                                        block_id,
                                        id,
                                        offset,
                                        data: Vec::new(),
                                    }
                                }
                            }
                        } else {
                            eprintln!(
                                "[fractral] missing integrity manager for block {}",
                                block_id
                            );
                            ProtocolSlice {
                                block_id,
                                id,
                                offset,
                                data: Vec::new(),
                            }
                        };

                        let _ = swarm.behaviour_mut().repair_rpc.send_response(channel, response);
                    }
                }
            }
            SwarmEvent::Behaviour(FractalBehaviourEvent::Gossipsub(
                gossipsub::Event::Message { message, .. },
            )) => {
                eprintln!(
                    "[fractral] received {} bytes of real-time stream",
                    message.data.len()
                );
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair_from_seed;
    use libp2p::Multiaddr;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn read_slice_returns_requested_bytes() {
        let temp = NamedTempFile::new().expect("temp file");
        std::fs::write(temp.path(), b"abcdefgh").expect("write file");
        let manager = IntegrityManager::new(temp.path());

        assert_eq!(manager.read_slice(2, 3).await.expect("slice"), b"cde");
    }

    #[tokio::test]
    async fn read_slice_errors_when_range_exceeds_file() {
        let temp = NamedTempFile::new().expect("temp file");
        std::fs::write(temp.path(), b"abc").expect("write file");
        let manager = IntegrityManager::new(temp.path());

        assert!(manager.read_slice(1, 4).await.is_err());
    }

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
