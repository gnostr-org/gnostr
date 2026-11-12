//! Networked cross-version round trips: v1 swarms talking to v2 swarms over
//! real libp2p QUIC connections, pinning exactly what the wire does in both
//! directions (all tests `#[ignore]`d like the other networked p2p tests).
//!
//! Matrix:
//!
//! - **v2 -> v1, real swarms (any codec)**: protocol negotiation fails with
//!   `OutboundFailure::UnsupportedProtocols` — after the protocol-id split no
//!   protocol string overlaps, so the stacks refuse to talk cleanly instead
//!   of half-talking.
//! - **v1 -> v2, real swarms**: same, symmetric.
//! - **Shared-id simulation** (v2 behaviours mounted on v1's
//!   `/fractal/repair/1.0.0/json` via `build_fractal_swarm_v2_with_protocols`,
//!   i.e. the pre-split state):
//!   - v2 requester -> v1 responder: the v2 request JSON decodes fine on v1
//!     (serde silently drops `len` — proven byte-level in
//!     fractal_wire_compat.rs), v1 serves the id and sends `ResponseSent`,
//!     but its plain-`ProtocolSlice` response cannot be parsed as
//!     `Result<ProtocolSliceV2, String>` — the v2 requester fails with
//!     `OutboundFailure::Io`. The request "succeeds" at v1's layer; the
//!     response is unparseable for v2.
//!   - v1 requester -> v2 responder: v1's request bytes lack `len`, so the
//!     v2 responder's codec read fails. libp2p-request-response 0.27 drops
//!     such streams before the behaviour ever sees them — handler.rs
//!     `on_fully_negotiated_inbound`: "By failing to schedule it, it means
//!     the upper layer will never know about the inbound request. Because of
//!     that we do not report any inbound failure." So the v2 responder serves
//!     nothing and emits no failure event; the v1 requester gets
//!     `OutboundFailure::Io(UnexpectedEof)` on the closed stream.
//! - **Control**: v2 -> v2 over both mounted codecs completes full round
//!   trips (`Ok` slices and explicit `Err` for missing ids) — the baseline
//!   the cross-version cases deviate from.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use futures::StreamExt;
use gnostr_p2p::perfect_ip_core::{
    build_fractal_swarm as build_v1_perfect_ip_swarm, FractalBehaviourEvent as V1PerfectIpEvent,
};
use gnostr_p2p::perfect_ip_core_v2::{RepairBinEventV2, RepairJsonEventV2};
use gnostr_p2p::{
    build_fractal_swarm_v2_with_protocols, packetize_v2, request_slices_v2, FractalBehaviourV2,
    FractalBehaviourV2Event, Header, HeaderV2, IntegrityManager, IntegrityManagerV2, ProtocolSlice,
    ProtocolSliceV2, WireCodecV2,
};
use libp2p::{request_response, swarm::SwarmEvent, Multiaddr};

/// v1's repair protocol id, for the shared-id simulations.
const V1_JSON_PROTOCOL: &str = "/fractal/repair/1.0.0/json";

fn sample_payload(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 251) as u8).collect()
}

fn v2_probe(id: &str) -> ProtocolSliceV2 {
    ProtocolSliceV2 {
        id: id.to_string(),
        header: HeaderV2 {
            seq_num: 0,
            total_packets: 0,
            len: 0,
        },
        data: Vec::new(),
        is_parity: false,
    }
}

fn v1_probe(id: &str) -> ProtocolSlice {
    ProtocolSlice {
        id: id.to_string(),
        header: Header {
            seq_num: 0,
            total_packets: 0,
        },
        data: Vec::new(),
        is_parity: false,
    }
}

/// Small v2 payload + manager + id list, shared by the round-trip tests.
fn v2_fixture() -> (Vec<u8>, Vec<String>, IntegrityManagerV2) {
    let payload = sample_payload(3000);
    let result = packetize_v2("ROOT".to_string(), payload.clone());
    let ids: Vec<String> = result
        .manifest
        .iter()
        .map(|entry| entry.id.clone())
        .collect();
    let mut manager = IntegrityManagerV2::new(result.manifest);
    for packet in &result.batch.packets {
        manager.record_slice(packet.clone());
    }
    (payload, ids, manager)
}

/// Small v1 payload + manager + id list, for the v1 responder/requester.
fn v1_fixture() -> (Vec<String>, IntegrityManager) {
    let payload = sample_payload(3000);
    let batch = gnostr_p2p::packetize("ROOT".to_string(), payload);
    let ids: Vec<String> = batch
        .packets
        .iter()
        .map(|packet| packet.id.clone())
        .collect();
    let mut manager = IntegrityManager::new(ids.clone());
    for packet in &batch.packets {
        manager.record_slice(packet.clone());
    }
    (ids, manager)
}

async fn spawn_v1_responder(
    manager: IntegrityManager,
    seed: &str,
    requests: Arc<AtomicUsize>,
    responses_sent: Arc<AtomicUsize>,
) -> (tokio::task::JoinHandle<()>, Multiaddr) {
    let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
    let handle = tokio::spawn({
        let seed = seed.to_string();
        async move {
            let keypair = gnostr_p2p::keypair_from_seed(Some(seed));
            let mut swarm = build_v1_perfect_ip_swarm(keypair).await.expect("v1 swarm");
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
                    SwarmEvent::Behaviour(V1PerfectIpEvent::RepairRpc(event)) => match event {
                        request_response::Event::Message { message, .. } => {
                            if let request_response::Message::Request {
                                request, channel, ..
                            } = message
                            {
                                requests.fetch_add(1, Ordering::SeqCst);
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
                                let _ = swarm
                                    .behaviour_mut()
                                    .repair_rpc
                                    .send_response(channel, response);
                            }
                        }
                        request_response::Event::ResponseSent { .. } => {
                            responses_sent.fetch_add(1, Ordering::SeqCst);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    });
    let addr = tokio::time::timeout(Duration::from_secs(5), addr_rx)
        .await
        .expect("v1 responder address timeout")
        .expect("v1 responder address channel");
    (handle, addr)
}

/// v2 responder with split behaviour arms (needed to call the right
/// `send_response`), counting inbound requests served and decode failures.
async fn spawn_v2_responder_split(
    manager: IntegrityManagerV2,
    seed: &str,
    requests: Arc<AtomicUsize>,
    inbound_failures: Arc<AtomicUsize>,
    json_protocol: &str,
) -> (tokio::task::JoinHandle<()>, Multiaddr) {
    let (addr_tx, addr_rx) = tokio::sync::oneshot::channel();
    let json_protocol = json_protocol.to_string();
    let handle = tokio::spawn({
        let seed = seed.to_string();
        async move {
            let keypair = gnostr_p2p::keypair_from_seed(Some(seed));
            let mut swarm = build_fractal_swarm_v2_with_protocols(
                keypair,
                "/fractal/repair/1.1.0/postcard",
                &json_protocol,
            )
            .await
            .expect("v2 swarm");
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
                    )) => handle_v2_responder_event(
                        &mut swarm,
                        event,
                        &manager,
                        &requests,
                        &inbound_failures,
                        true,
                    ),
                    SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairJson(
                        RepairJsonEventV2(event),
                    )) => handle_v2_responder_event(
                        &mut swarm,
                        event,
                        &manager,
                        &requests,
                        &inbound_failures,
                        false,
                    ),
                    _ => {}
                }
            }
        }
    });
    let addr = tokio::time::timeout(Duration::from_secs(5), addr_rx)
        .await
        .expect("v2 responder address timeout")
        .expect("v2 responder address channel");
    (handle, addr)
}

fn handle_v2_responder_event(
    swarm: &mut libp2p::Swarm<FractalBehaviourV2>,
    event: request_response::Event<ProtocolSliceV2, gnostr_p2p::RepairResponseV2>,
    manager: &IntegrityManagerV2,
    requests: &Arc<AtomicUsize>,
    inbound_failures: &Arc<AtomicUsize>,
    binary: bool,
) {
    match event {
        request_response::Event::Message { message, .. } => {
            if let request_response::Message::Request {
                request, channel, ..
            } = message
            {
                requests.fetch_add(1, Ordering::SeqCst);
                let response = match manager.received_slices.get(&request.id) {
                    Some(slice) => Ok(slice.clone()),
                    None => Err(format!("not held: {}", request.id)),
                };
                if binary {
                    let _ = swarm
                        .behaviour_mut()
                        .repair_bin
                        .inner
                        .send_response(channel, response);
                } else {
                    let _ = swarm
                        .behaviour_mut()
                        .repair_json
                        .inner
                        .send_response(channel, response);
                }
            }
        }
        request_response::Event::InboundFailure { .. } => {
            inbound_failures.fetch_add(1, Ordering::SeqCst);
        }
        _ => {}
    }
}

#[tokio::test]
#[ignore = "networked v2<->v2 control round trip: run with --ignored --nocapture"]
async fn v2_v2_round_trip_control_both_codecs() {
    // Baseline: both mounted codecs complete full round trips, including
    // explicit Err responses for missing ids.
    let (payload, ids, manager) = v2_fixture();
    let responder_manager = manager.clone();
    let (handle, addr) = spawn_v2_responder_split(
        responder_manager,
        "v2_control_responder",
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        "/fractal/repair/2.0.0/json",
    )
    .await;

    for wire in [WireCodecV2::Binary, WireCodecV2::Json] {
        let mut request_ids = ids.clone();
        request_ids.push("ROOT.definitely_missing".to_string());
        let fetch =
            request_slices_v2(addr.clone(), "v2_control_requester", request_ids, wire).await;
        println!(
            "control {wire:?}: {} received, {} errors",
            fetch.received.len(),
            fetch.errors.len()
        );
        assert_eq!(fetch.received.len(), ids.len());
        assert_eq!(fetch.errors.len(), 1);
        assert!(fetch.errors[0].contains("not held: ROOT.definitely_missing"));

        let mut check = IntegrityManagerV2::new(manager.manifest.values().cloned().collect());
        for slice in fetch.received.values() {
            check.record_slice(slice.clone());
        }
        assert!(check.rejected.is_empty());
        assert_eq!(check.reconstruct_payload().expect("payload"), payload);
    }
    handle.abort();
}

#[tokio::test]
#[ignore = "networked v2->v1 negotiation refusal: run with --ignored --nocapture"]
async fn v2_requester_real_v1_responder_refuses_negotiation() {
    let (v1_ids, v1_manager) = v1_fixture();
    let (v1_handle, v1_addr) = spawn_v1_responder(
        v1_manager,
        "v1_real_responder",
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    )
    .await;

    let keypair = gnostr_p2p::keypair_from_seed(Some("v2_requester".to_string()));
    let mut swarm = gnostr_p2p::build_fractal_swarm_v2(keypair)
        .await
        .expect("v2 swarm");
    swarm
        .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm.dial(v1_addr).unwrap();

    let mut messages = 0usize;
    let outcome = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("connected to {peer_id}; sending postcard request");
                    swarm
                        .behaviour_mut()
                        .repair_bin
                        .inner
                        .send_request(&peer_id, v2_probe(&v1_ids[0]));
                }
                SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairBin(RepairBinEventV2(
                    event,
                ))) => match event {
                    request_response::Event::Message { .. } => {
                        messages += 1;
                    }
                    request_response::Event::OutboundFailure { error, .. } => break error,
                    other => println!("event: {other:?}"),
                },
                SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairJson(RepairJsonEventV2(
                    event,
                ))) => {
                    println!("json event: {event:?}");
                }
                other => println!("swarm event: {other:?}"),
            }
        }
    })
    .await
    .expect("timed out waiting for negotiation outcome");

    println!("outcome: {outcome:?}");
    assert!(
        matches!(
            outcome,
            request_response::OutboundFailure::UnsupportedProtocols
        ),
        "expected UnsupportedProtocols, got {outcome:?}"
    );
    assert_eq!(messages, 0, "no application message must be exchanged");
    v1_handle.abort();
}

#[tokio::test]
#[ignore = "networked v1->v2 negotiation refusal: run with --ignored --nocapture"]
async fn v1_requester_real_v2_responder_refuses_negotiation() {
    let (_payload, ids, manager) = v2_fixture();
    let (v2_handle, v2_addr) = spawn_v2_responder_split(
        manager,
        "v2_real_responder",
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        "/fractal/repair/2.0.0/json",
    )
    .await;

    let keypair = gnostr_p2p::keypair_from_seed(Some("v1_requester".to_string()));
    let mut swarm = build_v1_perfect_ip_swarm(keypair).await.expect("v1 swarm");
    swarm
        .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm.dial(v2_addr).unwrap();

    let mut messages = 0usize;
    let outcome = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("connected to {peer_id}; sending v1 json request");
                    swarm
                        .behaviour_mut()
                        .repair_rpc
                        .send_request(&peer_id, v1_probe(&ids[0]));
                }
                SwarmEvent::Behaviour(V1PerfectIpEvent::RepairRpc(event)) => match event {
                    request_response::Event::Message { .. } => {
                        messages += 1;
                    }
                    request_response::Event::OutboundFailure { error, .. } => break error,
                    other => println!("event: {other:?}"),
                },
                other => println!("swarm event: {other:?}"),
            }
        }
    })
    .await
    .expect("timed out waiting for negotiation outcome");

    println!("outcome: {outcome:?}");
    assert!(
        matches!(
            outcome,
            request_response::OutboundFailure::UnsupportedProtocols
        ),
        "expected UnsupportedProtocols, got {outcome:?}"
    );
    assert_eq!(messages, 0, "no application message must be exchanged");
    v2_handle.abort();
}

#[tokio::test]
#[ignore = "networked shared-id v2->v1 response decode failure: run with --ignored --nocapture"]
async fn shared_id_v2_requester_v1_responder_response_decode_fails() {
    // The pre-split state: v2 types mounted on v1's json protocol id. The v2
    // request decodes on v1 (serde drops `len`), v1 serves it — but the
    // plain-ProtocolSlice response cannot be parsed as
    // Result<ProtocolSliceV2, String> on the v2 side.
    let (v1_ids, v1_manager) = v1_fixture();
    let v1_requests = Arc::new(AtomicUsize::new(0));
    let v1_sent = Arc::new(AtomicUsize::new(0));
    let (v1_handle, v1_addr) = spawn_v1_responder(
        v1_manager,
        "v1_shared_responder",
        v1_requests.clone(),
        v1_sent.clone(),
    )
    .await;

    let keypair = gnostr_p2p::keypair_from_seed(Some("v2_shared_requester".to_string()));
    let mut swarm = build_fractal_swarm_v2_with_protocols(
        keypair,
        "/fractal/repair/1.1.0/postcard",
        V1_JSON_PROTOCOL,
    )
    .await
    .expect("v2 shared-id swarm");
    swarm
        .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm.dial(v1_addr).unwrap();

    let mut received = 0usize;
    let outcome: request_response::OutboundFailure =
        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                match swarm.select_next_some().await {
                    SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                        println!("connected; sending v2 json probe on v1's protocol id");
                        swarm
                            .behaviour_mut()
                            .repair_json
                            .inner
                            .send_request(&peer_id, v2_probe(&v1_ids[0]));
                    }
                    SwarmEvent::Behaviour(FractalBehaviourV2Event::RepairJson(
                        RepairJsonEventV2(event),
                    )) => match event {
                        request_response::Event::Message { message, .. } => {
                            if let request_response::Message::Response { .. } = message {
                                received += 1;
                            }
                        }
                        request_response::Event::OutboundFailure { error, .. } => break error,
                        other => println!("event: {other:?}"),
                    },
                    other => println!("swarm event: {other:?}"),
                }
            }
        })
        .await
        .expect("timed out waiting for response decode failure");

    println!("v2 requester outcome: {outcome:?}");
    println!(
        "v1 responder: {} requests decoded and served, {} responses sent",
        v1_requests.load(Ordering::SeqCst),
        v1_sent.load(Ordering::SeqCst)
    );
    // The request fully succeeded at v1's layer ...
    assert!(
        v1_requests.load(Ordering::SeqCst) >= 1,
        "v1 must decode the v2 request"
    );
    assert!(
        v1_sent.load(Ordering::SeqCst) >= 1,
        "v1 must send a response"
    );
    // ... but the response is unparseable for v2: serde rejects the plain
    // slice map as an unknown variant of the Result enum.
    match &outcome {
        request_response::OutboundFailure::Io(error) => {
            let message = format!("{error:?}");
            assert!(
                message.contains("unknown variant `id`, expected `Ok` or `Err`"),
                "expected the Result-wrapper decode error, got: {message}"
            );
        }
        other => panic!("expected Io (response decode error), got {other:?}"),
    }
    assert_eq!(received, 0, "v2 must not accept the v1 response");
    v1_handle.abort();
}

#[tokio::test]
#[ignore = "networked shared-id v1->v2 request decode failure: run with --ignored --nocapture"]
async fn shared_id_v1_requester_v2_responder_request_decode_fails() {
    // The other pre-split direction: v1 requests lack `len`, so the v2
    // responder's codec read fails. libp2p-request-response 0.27 drops such
    // streams before the behaviour sees them (no InboundFailure — see the
    // module docs), so the v2 responder serves nothing; the v1 requester
    // gets OutboundFailure::Io(UnexpectedEof) on the closed stream.
    let (_payload, ids, manager) = v2_fixture();
    let v2_requests = Arc::new(AtomicUsize::new(0));
    let v2_inbound_failures = Arc::new(AtomicUsize::new(0));
    let (v2_handle, v2_addr) = spawn_v2_responder_split(
        manager,
        "v2_shared_responder",
        v2_requests.clone(),
        v2_inbound_failures.clone(),
        V1_JSON_PROTOCOL,
    )
    .await;

    let keypair = gnostr_p2p::keypair_from_seed(Some("v1_shared_requester".to_string()));
    let mut swarm = build_v1_perfect_ip_swarm(keypair).await.expect("v1 swarm");
    swarm
        .listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().unwrap())
        .unwrap();
    swarm.dial(v2_addr).unwrap();

    let mut received = 0usize;
    let outcome = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            match swarm.select_next_some().await {
                SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                    println!("connected; sending v1 json probe");
                    swarm
                        .behaviour_mut()
                        .repair_rpc
                        .send_request(&peer_id, v1_probe(&ids[0]));
                }
                SwarmEvent::Behaviour(V1PerfectIpEvent::RepairRpc(event)) => match event {
                    request_response::Event::Message { .. } => {
                        received += 1;
                    }
                    request_response::Event::OutboundFailure { error, .. } => break error,
                    other => println!("event: {other:?}"),
                },
                other => println!("swarm event: {other:?}"),
            }
        }
    })
    .await
    .expect("timed out waiting for outbound failure");

    println!("v1 requester outcome: {outcome:?}");
    println!(
        "v2 responder: {} requests served, {} inbound failures",
        v2_requests.load(Ordering::SeqCst),
        v2_inbound_failures.load(Ordering::SeqCst)
    );
    // v2 could not decode the v1 request: nothing is served, and the decode
    // failure never reaches the behaviour layer in request-response 0.27
    // (silently dropped stream), so the inbound-failure counter stays 0.
    assert_eq!(
        v2_requests.load(Ordering::SeqCst),
        0,
        "v2 must never serve a v1 request"
    );
    assert_eq!(
        v2_inbound_failures.load(Ordering::SeqCst),
        0,
        "0.27 drops undecodable inbound requests before the behaviour sees them"
    );
    // The v1 requester sees the stream close mid-read: Io(UnexpectedEof),
    // and no response.
    match &outcome {
        request_response::OutboundFailure::Io(error) => {
            assert_eq!(
                error.kind(),
                std::io::ErrorKind::UnexpectedEof,
                "expected EOF on the silently-dropped stream, got {error:?}"
            );
        }
        other => panic!("expected Io(UnexpectedEof), got {other:?}"),
    }
    assert_eq!(received, 0);
    v2_handle.abort();
}
