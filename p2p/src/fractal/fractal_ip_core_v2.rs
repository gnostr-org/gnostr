//! v2 of the `perfect_ip` repair wire protocol — gap-fill of `p2p/TODO.md`
//! plans 1, 2, and 6, implemented beside v1 (`fractal_ip_core.rs`, untouched).
//!
//! - **Plan 1 — explicit absence.** A repair request is answered with
//!   `Result<ProtocolSliceV2, String>`: `Ok(slice)` when held, otherwise
//!   `Err("not held: <id>")`. The v1 empty-placeholder response, which was
//!   indistinguishable from a zero-length leaf, is gone.
//! - **Plan 2 — binary wire.** The production codec is postcard on
//!   `/fractal/repair/1.1.0/postcard`; JSON stays mounted beside it for
//!   debugging on `/fractal/repair/2.0.0/json` — versioned per stack so a v2
//!   node never negotiates the debug codec against a v1 peer answering plain
//!   `ProtocolSlice`. [`WireCodecV2`] selects which behaviour a requester
//!   sends on; both are always mounted so a node can serve either.
//! - **Plan 6 — `git fsck --strict`.** [`git_fsck_strict`] runs before the
//!   clone-based verification in the v2 example and tests.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::marker::PhantomData;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

use futures::StreamExt;
use gnostr_types::perfect_ip_v2::{IntegrityManagerV2, ProtocolSliceV2};
use libp2p::{
    request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    Multiaddr, PeerId, StreamProtocol,
};
use serde::{de::DeserializeOwned, Serialize};

/// Binary (postcard) repair protocol id — the v2 production codec.
pub const REPAIR_PROTOCOL_V2_POSTCARD: &str = "/fractal/repair/1.1.0/postcard";
/// JSON repair protocol id — kept for debugging/inspection.
pub const REPAIR_PROTOCOL_V2_JSON: &str = "/fractal/repair/2.0.0/json";

/// Repair response: explicit presence/absence on the wire (TODO.md plan 1).
pub type RepairResponseV2 = Result<ProtocolSliceV2, String>;

/// Which wire codec a requester speaks. Both behaviours are mounted on every
/// v2 swarm; this only chooses the one used for outbound requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WireCodecV2 {
    /// postcard on `/fractal/repair/1.1.0/postcard` (default, production).
    #[default]
    Binary,
    /// JSON on `/fractal/repair/2.0.0/json` (debugging/inspection). The id is
    /// versioned per stack so a v2 node never negotiates the debug codec
    /// against a v1 peer answering plain `ProtocolSlice`.
    Json,
}

/// A request-response codec backed by [`postcard`].
///
/// libp2p 0.54 ships JSON and CBOR codecs only; this mirrors the upstream
/// JSON codec (same size limits, same read-to-end/write-all shape) with
/// postcard serialization for a ~2x+ smaller wire footprint (TODO.md plan 2).
#[derive(Debug)]
pub struct PostcardCodec<Req, Resp> {
    phantom: PhantomData<(Req, Resp)>,
}

impl<Req, Resp> Default for PostcardCodec<Req, Resp> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
}

impl<Req, Resp> Clone for PostcardCodec<Req, Resp> {
    fn clone(&self) -> Self {
        Self::default()
    }
}

/// Max request size in bytes.
const REQUEST_SIZE_MAXIMUM: u64 = 1024 * 1024;
/// Max response size in bytes.
const RESPONSE_SIZE_MAXIMUM: u64 = 10 * 1024 * 1024;

#[async_trait::async_trait]
impl<Req, Resp> request_response::Codec for PostcardCodec<Req, Resp>
where
    Req: Send + Serialize + DeserializeOwned,
    Resp: Send + Serialize + DeserializeOwned,
{
    type Protocol = StreamProtocol;
    type Request = Req;
    type Response = Resp;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Req>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        let mut vec = Vec::new();
        io.take(REQUEST_SIZE_MAXIMUM).read_to_end(&mut vec).await?;
        postcard::from_bytes(&vec)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> std::io::Result<Resp>
    where
        T: futures::AsyncRead + Unpin + Send,
    {
        use futures::AsyncReadExt;

        let mut vec = Vec::new();
        io.take(RESPONSE_SIZE_MAXIMUM).read_to_end(&mut vec).await?;
        postcard::from_bytes(&vec)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;

        let data = postcard::to_allocvec(&req)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        io.write_all(data.as_ref()).await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        resp: Self::Response,
    ) -> std::io::Result<()>
    where
        T: futures::AsyncWrite + Unpin + Send,
    {
        use futures::AsyncWriteExt;

        let data = postcard::to_allocvec(&resp)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        io.write_all(data.as_ref()).await?;
        Ok(())
    }
}

/// Binary repair behaviour, newtyped so the composite behaviour can tell its
/// events apart from the JSON one's (both emit the same
/// `request_response::Event` type).
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "RepairBinEventV2")]
pub struct RepairBinV2 {
    pub inner: request_response::Behaviour<PostcardCodec<ProtocolSliceV2, RepairResponseV2>>,
}

impl RepairBinV2 {
    pub fn new(
        protocols: impl IntoIterator<Item = (StreamProtocol, request_response::ProtocolSupport)>,
        config: request_response::Config,
    ) -> Self {
        Self {
            inner: request_response::Behaviour::new(protocols, config),
        }
    }
}

/// Event wrapper for [`RepairBinV2`].
#[derive(Debug)]
pub struct RepairBinEventV2(pub request_response::Event<ProtocolSliceV2, RepairResponseV2>);

impl From<request_response::Event<ProtocolSliceV2, RepairResponseV2>> for RepairBinEventV2 {
    fn from(event: request_response::Event<ProtocolSliceV2, RepairResponseV2>) -> Self {
        Self(event)
    }
}

/// JSON repair behaviour, kept for debugging (TODO.md plan 2).
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "RepairJsonEventV2")]
pub struct RepairJsonV2 {
    pub inner: request_response::json::Behaviour<ProtocolSliceV2, RepairResponseV2>,
}

impl RepairJsonV2 {
    pub fn new(
        protocols: impl IntoIterator<Item = (StreamProtocol, request_response::ProtocolSupport)>,
        config: request_response::Config,
    ) -> Self {
        Self {
            inner: request_response::json::Behaviour::new(protocols, config),
        }
    }
}

/// Event wrapper for [`RepairJsonV2`].
#[derive(Debug)]
pub struct RepairJsonEventV2(pub request_response::Event<ProtocolSliceV2, RepairResponseV2>);

impl From<request_response::Event<ProtocolSliceV2, RepairResponseV2>> for RepairJsonEventV2 {
    fn from(event: request_response::Event<ProtocolSliceV2, RepairResponseV2>) -> Self {
        Self(event)
    }
}

/// Composite v2 repair behaviour: postcard (production) + JSON (debugging),
/// both mounted on every v2 swarm.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "FractalBehaviourV2Event")]
pub struct FractalBehaviourV2 {
    pub repair_bin: RepairBinV2,
    pub repair_json: RepairJsonV2,
}

/// Events emitted by the v2 repair behaviour.
#[derive(Debug)]
pub enum FractalBehaviourV2Event {
    RepairBin(RepairBinEventV2),
    RepairJson(RepairJsonEventV2),
}

impl From<RepairBinEventV2> for FractalBehaviourV2Event {
    fn from(event: RepairBinEventV2) -> Self {
        Self::RepairBin(event)
    }
}

impl From<RepairJsonEventV2> for FractalBehaviourV2Event {
    fn from(event: RepairJsonEventV2) -> Self {
        Self::RepairJson(event)
    }
}

/// Build the v2 repair swarm with both wire codecs mounted.
pub async fn build_fractal_swarm_v2(
    local_key: libp2p::identity::Keypair,
) -> Result<Swarm<FractalBehaviourV2>, Box<dyn Error + Send + Sync>> {
    build_fractal_swarm_v2_with_protocols(
        local_key,
        REPAIR_PROTOCOL_V2_POSTCARD,
        REPAIR_PROTOCOL_V2_JSON,
    )
    .await
}

/// Build the v2 repair swarm with explicit protocol ids.
///
/// Production code should use [`build_fractal_swarm_v2`]; this exists so
/// compat tests can mount the v2 behaviours on v1's protocol id and prove at
/// the network level why the ids must not be shared.
pub async fn build_fractal_swarm_v2_with_protocols(
    local_key: libp2p::identity::Keypair,
    postcard_protocol: &str,
    json_protocol: &str,
) -> Result<Swarm<FractalBehaviourV2>, Box<dyn Error + Send + Sync>> {
    let postcard_protocol = postcard_protocol.to_string();
    let json_protocol = json_protocol.to_string();
    let build =
        move |_: &libp2p::identity::Keypair| -> Result<FractalBehaviourV2, Box<dyn Error + Send + Sync>> {
            Ok(FractalBehaviourV2 {
                repair_bin: RepairBinV2::new(
                    [(
                        StreamProtocol::try_from_owned(postcard_protocol)?,
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
                repair_json: RepairJsonV2::new(
                    [(
                        StreamProtocol::try_from_owned(json_protocol)?,
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
            })
        };

    #[cfg(target_os = "tvos")]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_behaviour(|key| build(key))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    #[cfg(not(target_os = "tvos"))]
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|key| build(key))?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    Ok(swarm)
}

/// Build a repair response: the held slice, or an explicit absence error —
/// never a fabricated placeholder (TODO.md plan 1).
pub fn repair_response_for(
    manager: &IntegrityManagerV2,
    request: &ProtocolSliceV2,
) -> RepairResponseV2 {
    match manager.received_slices.get(&request.id) {
        Some(slice) => Ok(slice.clone()),
        None => Err(format!("not held: {}", request.id)),
    }
}

/// Send a repair response on the behaviour matching `wire`.
pub fn send_repair_response(
    swarm: &mut Swarm<FractalBehaviourV2>,
    wire: WireCodecV2,
    channel: request_response::ResponseChannel<RepairResponseV2>,
    response: RepairResponseV2,
) {
    match wire {
        WireCodecV2::Binary => {
            let _ = swarm
                .behaviour_mut()
                .repair_bin
                .inner
                .send_response(channel, response);
        }
        WireCodecV2::Json => {
            let _ = swarm
                .behaviour_mut()
                .repair_json
                .inner
                .send_response(channel, response);
        }
    }
}

/// Serve one inbound repair request event; inbound requests on the other
/// behaviour are ignored (requesters in the v2 flow do not serve).
pub fn serve_repair_event(
    swarm: &mut Swarm<FractalBehaviourV2>,
    event: request_response::Event<ProtocolSliceV2, RepairResponseV2>,
    wire: WireCodecV2,
    manager: &IntegrityManagerV2,
) {
    if let request_response::Event::Message { message, .. } = event {
        if let request_response::Message::Request {
            request, channel, ..
        } = message
        {
            let response = repair_response_for(manager, &request);
            send_repair_response(swarm, wire, channel, response);
        }
    }
}

/// Start the v2 repair engine and serve slice requests from `manager`.
///
/// Missing ids answer `Err("not held: <id>")` on the wire; there is no
/// placeholder path.
pub async fn run_fractal_engine_v2(
    local_key: libp2p::identity::Keypair,
    listen_address: libp2p::Multiaddr,
    manager: IntegrityManagerV2,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut swarm = build_fractal_swarm_v2(local_key).await?;
    swarm.listen_on(listen_address)?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairBin(RepairBinEventV2(event))) => {
                serve_repair_event(&mut swarm, event, WireCodecV2::Binary, &manager)
            }
            SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairJson(RepairJsonEventV2(
                event,
            ))) => serve_repair_event(&mut swarm, event, WireCodecV2::Json, &manager),
            _ => {}
        }
    }
}

/// Outcome of a v2 repair fetch.
#[derive(Debug, Default)]
pub struct RepairFetchV2 {
    /// Slices answered with `Ok`.
    pub received: HashMap<String, ProtocolSliceV2>,
    /// `Err` messages from the responder (e.g. `not held: <id>`). Ids here
    /// count as answered-with-absence and are never inserted into an
    /// [`IntegrityManagerV2`].
    pub errors: Vec<String>,
}

/// Send one repair request for `id` on the behaviour matching `wire`.
pub fn send_repair_request(
    swarm: &mut Swarm<FractalBehaviourV2>,
    wire: WireCodecV2,
    peer: PeerId,
    request: ProtocolSliceV2,
) {
    match wire {
        WireCodecV2::Binary => {
            swarm
                .behaviour_mut()
                .repair_bin
                .inner
                .send_request(&peer, request);
        }
        WireCodecV2::Json => {
            swarm
                .behaviour_mut()
                .repair_json
                .inner
                .send_request(&peer, request);
        }
    }
}

/// Build a probe request for `id` (empty data; the responder only reads `id`).
pub fn repair_probe(id: &str) -> ProtocolSliceV2 {
    ProtocolSliceV2 {
        id: id.to_string(),
        header: gnostr_types::perfect_ip_v2::HeaderV2 {
            seq_num: 0,
            total_packets: 0,
            len: 0,
        },
        data: Vec::new(),
        is_parity: false,
    }
}

/// Handle one repair response event during a fetch. Returns `true` when the
/// event carried a response (so the caller can re-check completion).
fn handle_repair_response(
    event: request_response::Event<ProtocolSliceV2, RepairResponseV2>,
    received: &mut HashMap<String, ProtocolSliceV2>,
    errors: &mut Vec<String>,
    accounted: &mut HashSet<String>,
) -> bool {
    let request_response::Event::Message { message, .. } = event else {
        return false;
    };
    let request_response::Message::Response { response, .. } = message else {
        return false;
    };
    match response {
        Ok(slice) => {
            accounted.insert(slice.id.clone());
            received.insert(slice.id.clone(), slice);
        }
        Err(message) => {
            // Explicit absence: count it, never insert a placeholder.
            if let Some(id) = message.strip_prefix("not held: ").map(str::to_string) {
                accounted.insert(id);
            }
            errors.push(message);
        }
    }
    true
}

/// Request every id in `slice_ids` from a repair peer.
///
/// `Ok` responses are collected into `received`; `Err` responses are
/// collected into `errors` and the id is never treated as received. Missing
/// ids are retried every 2 seconds until every id is accounted for or the
/// 60-second deadline elapses.
pub async fn request_slices_v2(
    responder_addr: Multiaddr,
    seed: &str,
    slice_ids: Vec<String>,
    wire: WireCodecV2,
) -> RepairFetchV2 {
    use futures::FutureExt;

    let keypair = crate::keypair_from_seed(Some(seed.to_string()));
    let mut swarm = build_fractal_swarm_v2(keypair)
        .await
        .expect("requester swarm");
    swarm
        .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm.dial(responder_addr).unwrap();

    let total = slice_ids.len();
    let mut received = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    let mut accounted: HashSet<String> = HashSet::new();
    let mut peer_id: Option<PeerId> = None;
    let mut retry_interval = tokio::time::interval(Duration::from_secs(2));
    retry_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    tokio::time::timeout(Duration::from_secs(60), async {
        loop {
            futures::select! {
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::ConnectionEstablished { peer_id: pid, .. } => {
                            peer_id = Some(pid);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            for id in &slice_ids {
                                if accounted.contains(id) {
                                    continue;
                                }
                                send_repair_request(&mut swarm, wire, pid, repair_probe(id));
                            }
                        }
                        SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairBin(
                            RepairBinEventV2(event),
                        )) => {
                            if handle_repair_response(
                                event,
                                &mut received,
                                &mut errors,
                                &mut accounted,
                            ) && accounted.len() == total
                            {
                                break;
                            }
                        }
                        SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairJson(
                            RepairJsonEventV2(event),
                        )) => {
                            if handle_repair_response(
                                event,
                                &mut received,
                                &mut errors,
                                &mut accounted,
                            ) && accounted.len() == total
                            {
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                _ = retry_interval.tick().fuse() => {
                    if let Some(pid) = peer_id {
                        let missing: Vec<String> = slice_ids
                            .iter()
                            .filter(|id| !accounted.contains(*id))
                            .take(10)
                            .cloned()
                            .collect();
                        for id in missing {
                            send_repair_request(&mut swarm, wire, pid, repair_probe(&id));
                        }
                    }
                }
            }
        }
    })
    .await
    .expect("request_slices_v2 timed out");

    RepairFetchV2 { received, errors }
}

/// Run `git fsck --strict` on a repository (TODO.md plan 6).
///
/// Surfaces fsck output on failure so object-store corruption the clone step
/// might tolerate is still caught.
pub fn git_fsck_strict(repo: &Path) -> Result<(), String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["fsck", "--strict"])
        .output()
        .map_err(|error| format!("spawn git fsck --strict: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git fsck --strict failed for {}:\nstdout:\n{}\nstderr:\n{}",
            repo.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair_from_seed;
    use gnostr_types::perfect_ip_v2::{packetize_v2, HeaderV2};
    use libp2p::request_response::Codec as _;
    use libp2p::Multiaddr;

    fn sample_payload(len: usize) -> Vec<u8> {
        (0..len).map(|i| (i % 251) as u8).collect()
    }

    #[tokio::test]
    async fn build_fractal_swarm_v2_accepts_quic_listen_address() {
        let keypair = keypair_from_seed(Some(gnostr_asyncgit::default_gnostr_private_key_hex()));
        let mut swarm = build_fractal_swarm_v2(keypair).await.expect("swarm");
        let addr: Multiaddr = "/ip4/127.0.0.1/udp/0/quic-v1"
            .parse()
            .expect("quic multiaddr");

        swarm.listen_on(addr).expect("quic listen");
    }

    #[tokio::test]
    async fn postcard_codec_round_trips_requests_and_responses() {
        let protocol = StreamProtocol::new(REPAIR_PROTOCOL_V2_POSTCARD);
        let mut codec: PostcardCodec<ProtocolSliceV2, RepairResponseV2> = PostcardCodec::default();

        let request = repair_probe("ROOT.0.1");
        let mut buffer: Vec<u8> = Vec::new();
        codec
            .write_request(&protocol, &mut buffer, request.clone())
            .await
            .expect("write request");
        let decoded = codec
            .read_request(&protocol, &mut futures::io::Cursor::new(buffer))
            .await
            .expect("read request");
        assert_eq!(decoded, request);

        let ok: RepairResponseV2 = Ok(ProtocolSliceV2 {
            id: "ROOT.0.1".to_string(),
            header: HeaderV2 {
                seq_num: 3,
                total_packets: 13,
                len: 4,
            },
            data: vec![0xDE, 0xAD, 0xBE, 0xEF],
            is_parity: false,
        });
        let mut buffer: Vec<u8> = Vec::new();
        codec
            .write_response(&protocol, &mut buffer, ok.clone())
            .await
            .expect("write ok response");
        let decoded = codec
            .read_response(&protocol, &mut futures::io::Cursor::new(buffer))
            .await
            .expect("read ok response");
        assert_eq!(decoded, ok);

        let err: RepairResponseV2 = Err("not held: ROOT.0.1".to_string());
        let mut buffer: Vec<u8> = Vec::new();
        codec
            .write_response(&protocol, &mut buffer, err.clone())
            .await
            .expect("write err response");
        let decoded = codec
            .read_response(&protocol, &mut futures::io::Cursor::new(buffer))
            .await
            .expect("read err response");
        assert_eq!(decoded, err);
    }

    #[test]
    fn postcard_wire_size_beats_json_on_real_packet_batch() {
        // TODO.md plan 2: measure the codec delta on a real packet batch and
        // keep binary at least 50% smaller than JSON on the wire.
        let payload = sample_payload(4096);
        let result = packetize_v2("ROOT".to_string(), payload.clone());

        let mut json_len = 0usize;
        let mut bin_len = 0usize;
        for slice in &result.batch.packets {
            let response: RepairResponseV2 = Ok(slice.clone());
            json_len += serde_json::to_vec(&response).expect("json encode").len();
            bin_len += postcard::to_allocvec(&response)
                .expect("postcard encode")
                .len();
        }

        let pct = if json_len > 0 {
            100.0 - (bin_len as f64 / json_len as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "v2 wire size over {} packets ({}B payload): json={}B postcard={}B ({pct:.1}% smaller)",
            result.batch.total_packets,
            payload.len(),
            json_len,
            bin_len
        );
        assert!(
            bin_len * 2 < json_len,
            "postcard should be >50% smaller than json: bin={bin_len} json={json_len}"
        );
    }

    #[tokio::test]
    #[ignore = "networked p2p sync test: run with --ignored --nocapture"]
    async fn fractal_v2_p2p_sync_reconstructs_git_repo() {
        let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../web/crlf.git")
            .canonicalize()
            .expect("locate web/crlf.git fixture");
        println!("fixture: {}", fixture.display());

        let walk = gnostr_types::perfect_ip_v2::run_directory_walk_v2(&fixture, 3, false)
            .expect("packetize fixture");
        let manifest = walk.manifest();
        let all_ids: Vec<String> = manifest.iter().map(|entry| entry.id.clone()).collect();
        println!("total slices to sync: {}", all_ids.len());

        let mut sender_manager = IntegrityManagerV2::new(manifest.clone());
        for entry in walk.files.values() {
            for packet in &entry.packets {
                sender_manager.record_slice(packet.clone());
            }
        }
        assert!(sender_manager.verify_integrity());

        let (sender_handle, sender_addr) = spawn_responder_v2(sender_manager, "sender_v2").await;
        let fetch = request_slices_v2(
            sender_addr,
            "receiver_v2",
            all_ids.clone(),
            WireCodecV2::Binary,
        )
        .await;
        sender_handle.abort();
        println!(
            "received {} slices, {} explicit errors",
            fetch.received.len(),
            fetch.errors.len()
        );
        assert!(
            fetch.errors.is_empty(),
            "unexpected errors: {:?}",
            fetch.errors
        );
        assert_eq!(fetch.received.len(), all_ids.len());

        // Every received slice must be self-consistent: no placeholders.
        for slice in fetch.received.values() {
            assert_eq!(
                slice.header.len as usize,
                slice.data.len(),
                "slice {} has a placeholder-looking shape",
                slice.id
            );
        }

        let mut receiver_manager = IntegrityManagerV2::new(manifest.clone());
        for slice in fetch.received.values() {
            receiver_manager.record_slice(slice.clone());
        }
        assert!(receiver_manager.rejected.is_empty());
        assert!(receiver_manager.verify_integrity());
        let report = receiver_manager.recover_missing();
        assert!(report.unrecoverable.is_empty());

        // Reconstruct purely from the wire-received slices.
        let temp = tempfile::TempDir::new().expect("temp dir");
        let reconstructed = temp.path().join("crlf_v2");
        let received = &fetch.received;
        let mut files = std::collections::BTreeMap::new();
        for (rel_path, entry) in &walk.files {
            let packets: Vec<ProtocolSliceV2> = entry
                .manifest
                .iter()
                .map(|e| received.get(&e.id).expect("slice present").clone())
                .collect();
            files.insert(
                rel_path.clone(),
                gnostr_types::perfect_ip_v2::FileEntryV2 {
                    packets,
                    manifest: entry.manifest.clone(),
                },
            );
        }
        let receiver_walk = gnostr_types::perfect_ip_v2::DirectoryWalkV2 {
            files,
            empty_dirs: walk.empty_dirs.clone(),
        };
        gnostr_types::perfect_ip_v2::reconstruct_directory_v2(&receiver_walk, &reconstructed)
            .expect("reconstruct directory");

        // Plan 6: fsck before clone.
        git_fsck_strict(&reconstructed).expect("git fsck --strict");

        let clone = temp.path().join("crlf_v2_repo");
        let status = Command::new("git")
            .arg("clone")
            .arg(&reconstructed)
            .arg(&clone)
            .status()
            .expect("spawn git clone");
        assert!(
            status.success(),
            "git clone from reconstructed directory failed"
        );

        let reconstructed_head = git_output(&reconstructed, &["rev-parse", "HEAD"]);
        let clone_head = git_output(&clone, &["rev-parse", "HEAD"]);
        assert_eq!(reconstructed_head, clone_head, "clone HEAD mismatch");
        let status_output = git_output(&clone, &["status", "--porcelain"]);
        assert!(status_output.is_empty(), "cloned working tree is not clean");
    }

    #[tokio::test]
    #[ignore = "networked p2p repair test: run with --ignored --nocapture"]
    async fn fractal_v2_p2p_missing_slice_returns_err_and_parity_recovers() {
        // TODO.md plan 1 verification: a missing id answers with an explicit
        // `Err`, counts as still-missing on the receiver, and parity recovery
        // still completes the payload byte-exactly.
        let payload = sample_payload(3000);
        let result = packetize_v2("ROOT".to_string(), payload.clone());
        let manifest = result.manifest.clone();

        let dropped = result
            .batch
            .packets
            .iter()
            .find(|packet| !packet.is_parity && packet.id.matches('.').count() >= 3)
            .expect("a nested data leaf")
            .id
            .clone();

        let mut sender_manager = IntegrityManagerV2::new(manifest.clone());
        for packet in &result.batch.packets {
            if packet.id != dropped {
                sender_manager.record_slice(packet.clone());
            }
        }

        let all_ids: Vec<String> = manifest.iter().map(|entry| entry.id.clone()).collect();
        let (sender_handle, sender_addr) = spawn_responder_v2(sender_manager, "sender_v2b").await;
        let fetch =
            request_slices_v2(sender_addr, "receiver_v2b", all_ids, WireCodecV2::Binary).await;
        sender_handle.abort();

        assert!(
            !fetch.received.contains_key(&dropped),
            "missing id must never be recorded as received"
        );
        assert!(
            fetch
                .errors
                .iter()
                .any(|message| message.contains(&dropped)),
            "missing id must surface as an explicit error, got: {:?}",
            fetch.errors
        );

        let mut receiver_manager = IntegrityManagerV2::new(manifest.clone());
        for slice in fetch.received.values() {
            receiver_manager.record_slice(slice.clone());
        }
        assert!(receiver_manager.rejected.is_empty());

        let report = receiver_manager.recover_missing();
        assert_eq!(report.recovered, vec![dropped.clone()]);
        assert!(report.unrecoverable.is_empty());
        assert_eq!(
            receiver_manager.reconstruct_payload().expect("payload"),
            payload
        );
    }

    async fn spawn_responder_v2(
        manager: IntegrityManagerV2,
        seed: &str,
    ) -> (tokio::task::JoinHandle<()>, Multiaddr) {
        let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
        let handle = tokio::spawn({
            let seed = seed.to_string();
            async move {
                let keypair = keypair_from_seed(Some(seed.clone()));
                let mut swarm = build_fractal_swarm_v2(keypair)
                    .await
                    .expect("responder swarm");
                swarm
                    .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
                    .unwrap();

                let mut addr_tx = Some(addr_tx);
                loop {
                    match swarm.select_next_some().await {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            if let Some(tx) = addr_tx.take() {
                                let _ = tx.send(address);
                            }
                        }
                        SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairBin(
                            RepairBinEventV2(event),
                        )) => serve_repair_event(&mut swarm, event, WireCodecV2::Binary, &manager),
                        SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairJson(
                            RepairJsonEventV2(event),
                        )) => serve_repair_event(&mut swarm, event, WireCodecV2::Json, &manager),
                        _ => {}
                    }
                }
            }
        });

        let addr = tokio::time::timeout(Duration::from_secs(5), addr_rx)
            .await
            .expect("responder address timeout")
            .expect("responder address channel");
        (handle, addr)
    }

    fn git_output(repo: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .expect("spawn git");
        assert!(
            output.status.success(),
            "git -C {} {} failed",
            repo.display(),
            args.join(" ")
        );
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }
}
