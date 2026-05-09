#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::BTreeMap;
    use std::fs;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use std::time::Duration;
    use std::time::{SystemTime, UNIX_EPOCH};

    use libp2p::{
        autonat, dcutr, gossipsub, identify, noise, ping, relay, request_response,
        swarm::{NetworkBehaviour, SwarmEvent},
        tcp, yamux, Multiaddr, PeerId,
    };
    use futures::StreamExt;
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;
    use tokio::sync::oneshot;

    use crate::{
        event::ChatEvent,
        evt_loop,
        message::{ClientMessage, EventKind, Filter, GitNote, RelayMessage, SubscriptionId},
        msg::{Msg, MsgKind},
        p2p::spawn_local_p2p_relay_service_async,
    };
    use gnostr_asyncgit::{
        sync::{add_note, commit, show_note, stage_add_file, RepoPath},
        types::{generate_git_note_event, Event, PrivateKey},
        GitNote as AsyncGitNote,
    };
    use gnostr_p2p::keypair_from_seed;
    use libp2p::relay::client::Event as RelayClientEvent;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    enum RelayTraceEnvelope {
        EventKind(EventKind),
        RelayMessage(RelayMessage),
        SubscriptionId(SubscriptionId),
        ClientMessage(ClientMessage),
        GitNote(GitNote),
    }

    #[derive(NetworkBehaviour)]
    struct RelayProbeBehaviour {
        relay: relay::client::Behaviour,
        autonat: autonat::Behaviour,
        dcutr: dcutr::Behaviour,
        gossipsub: gossipsub::Behaviour,
        identify: identify::Behaviour,
        ping: ping::Behaviour,
        request_response: request_response::cbor::Behaviour<RelayTraceEnvelope, RelayTraceEnvelope>,
    }

    enum RelayProbeCommand {
        Dial {
            peer_id: PeerId,
            addr: Multiaddr,
            sender: oneshot::Sender<Result<(), String>>,
        },
        ListenOn {
            addr: Multiaddr,
            sender: oneshot::Sender<Result<(), String>>,
        },
        Request {
            peer_id: PeerId,
            message: RelayTraceEnvelope,
            sender: oneshot::Sender<Result<RelayTraceEnvelope, String>>,
        },
    }

    struct RelayProbePeer {
        peer_id: PeerId,
        command_tx: mpsc::Sender<RelayProbeCommand>,
        status_rx: mpsc::Receiver<String>,
    }

    impl RelayProbePeer {
        async fn dial(&mut self, peer_id: PeerId, addr: Multiaddr) -> Result<(), String> {
            let (sender, receiver) = oneshot::channel();
            self.command_tx
                .send(RelayProbeCommand::Dial {
                    peer_id,
                    addr,
                    sender,
                })
                .await
                .expect("relay probe peer to stay alive");
            receiver.await.map_err(|_| "dial response channel closed".to_string())?
        }

        async fn listen_on(&mut self, addr: Multiaddr) -> Result<(), String> {
            let (sender, receiver) = oneshot::channel();
            self.command_tx
                .send(RelayProbeCommand::ListenOn { addr, sender })
                .await
                .expect("relay probe peer to stay alive");
            receiver
                .await
                .map_err(|_| "listen response channel closed".to_string())?
        }

        async fn request(
            &mut self,
            peer_id: PeerId,
            message: RelayTraceEnvelope,
        ) -> Result<RelayTraceEnvelope, String> {
            let (sender, receiver) = oneshot::channel();
            self.command_tx
                .send(RelayProbeCommand::Request {
                    peer_id,
                    message,
                    sender,
                })
                .await
                .expect("relay probe peer to stay alive");
            receiver
                .await
                .map_err(|_| "request response channel closed".to_string())?
        }

        async fn wait_for_status_contains(&mut self, needle: &str, timeout: Duration) -> String {
            let deadline = tokio::time::Instant::now() + timeout;

            loop {
                let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
                let status = tokio::time::timeout(remaining, self.status_rx.recv())
                    .await
                    .expect("timeout waiting for relay status")
                    .expect("status channel closed");
                if status.contains(needle) {
                    return status;
                }
            }
        }
    }

    fn build_relay_circuit_addr(base: &Multiaddr, relay_peer_id: PeerId, target: PeerId) -> Multiaddr {
        base.clone()
            .with(libp2p::multiaddr::Protocol::P2p(relay_peer_id))
            .with(libp2p::multiaddr::Protocol::P2pCircuit)
            .with(libp2p::multiaddr::Protocol::P2p(target))
    }

    fn init_trace_repo() -> (PathBuf, RepoPath, git2::Oid) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("gnostr-chat-trace-{}-{unique}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create trace repo root");

        let repo = git2::Repository::init(&root).expect("init trace repo");
        {
            let mut config = repo.config().expect("trace config");
            config.set_str("user.name", "gnostr-trace").expect("set trace user");
            config
                .set_str("user.email", "trace@gnostr.org")
                .expect("set trace email");
        }

        let repo_path: RepoPath = root
            .to_str()
            .expect("trace repo path")
            .into();

        let trace_file = root.join("trace.txt");
        let mut file = fs::File::create(&trace_file).expect("trace file");
        writeln!(file, "trace repo {}", root.display())
        .expect("write trace file");

        stage_add_file(&repo_path, Path::new("trace.txt")).expect("stage trace file");
        let commit_id = commit(&repo_path, "trace commit").expect("trace commit");
        (root, repo_path, commit_id.into())
    }

    fn trace_filter_from_event(event: &Event) -> Filter {
        let mut tags = BTreeMap::new();
        for tag in &event.tags {
            let tagname = tag.tagname();
            if let Some(letter) = tagname.chars().next() {
                tags.entry(letter)
                    .or_insert_with(Vec::new)
                    .push(tag.value().to_string());
            }
        }

        let mut filter = Filter::default();
        filter.ids = vec![event.id.into()];
        filter.authors = vec![event.pubkey.into()];
        filter.kinds = vec![event.kind];
        filter.tags = tags;
        filter.since = Some(event.created_at);
        filter.until = Some(event.created_at);
        filter.limit = Some(event.tags.len());
        filter
    }

    fn relay_trace_envelopes() -> Vec<RelayTraceEnvelope> {
        let (_root, repo_path, commit_id) = init_trace_repo();
        let _note_id = add_note(
            &repo_path,
            commit_id,
            &format!("trace note for {commit_id}"),
            None,
            false,
        )
        .expect("trace note");
        let note = show_note(&repo_path, commit_id, None)
            .expect("show trace note")
            .expect("trace note exists");
        let git_note: AsyncGitNote = (&note).into();
        let private_key = PrivateKey::generate();
        let event = generate_git_note_event(&git_note, &private_key).expect("git note event");
        let subscription_id = SubscriptionId(event.id.as_hex_string());
        let filter = trace_filter_from_event(&event);

        vec![
            RelayTraceEnvelope::EventKind(event.kind),
            RelayTraceEnvelope::RelayMessage(RelayMessage::Event(
                subscription_id.clone(),
                Box::new(event.clone()),
            )),
            RelayTraceEnvelope::SubscriptionId(subscription_id.clone()),
            RelayTraceEnvelope::ClientMessage(ClientMessage::Req(subscription_id, vec![filter])),
            RelayTraceEnvelope::GitNote(git_note),
        ]
    }

    fn spawn_relay_probe_peer(seed: &str) -> RelayProbePeer {
        let keypair = keypair_from_seed(Some(seed.to_string()));
        let peer_id = keypair.public().to_peer_id();
        let (command_tx, mut command_rx) = mpsc::channel::<RelayProbeCommand>(8);
        let (status_tx, status_rx) = mpsc::channel::<String>(32);

        tokio::spawn(async move {
            let mut swarm = libp2p::SwarmBuilder::with_existing_identity(keypair)
                .with_tokio()
                .with_tcp(
                    tcp::Config::default(),
                    noise::Config::new,
                    yamux::Config::default,
                )
                .expect("tcp")
                .with_quic()
                .with_relay_client(noise::Config::new, yamux::Config::default)
                .expect("relay client")
                .with_behaviour(|key, relay_client| {
                    let local_peer_id = key.public().to_peer_id();
                    RelayProbeBehaviour {
                        relay: relay_client,
                        autonat: autonat::Behaviour::new(local_peer_id, autonat::Config::default()),
                        dcutr: dcutr::Behaviour::new(local_peer_id),
                        gossipsub: gossipsub::Behaviour::new(
                            gossipsub::MessageAuthenticity::Signed(key.clone()),
                            gossipsub::ConfigBuilder::default()
                                .heartbeat_interval(Duration::from_secs(10))
                                .validation_mode(gossipsub::ValidationMode::Strict)
                                .build()
                                .expect("gossipsub config"),
                        )
                        .expect("gossipsub"),
                        identify: identify::Behaviour::new(identify::Config::new(
                            "/ipfs/id/1.0.0".to_string(),
                            key.public(),
                        )),
                        ping: ping::Behaviour::new(ping::Config::new()),
                        request_response: request_response::cbor::Behaviour::new(
                            [
                                (
                                    libp2p::StreamProtocol::new("/relay-probe/1"),
                                    request_response::ProtocolSupport::Full,
                                ),
                            ],
                            request_response::Config::default(),
                        ),
                    }
                })
                .expect("behaviour")
                .build();

            swarm.listen_on("/ip4/127.0.0.1/tcp/0".parse().expect("tcp listen")).expect("listen tcp");
            swarm.listen_on("/ip4/127.0.0.1/udp/0/quic-v1".parse().expect("quic listen")).expect("listen quic");

            let mut pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), String>>> = HashMap::new();
            let mut pending_request: HashMap<request_response::OutboundRequestId, oneshot::Sender<Result<RelayTraceEnvelope, String>>> =
                HashMap::new();

            loop {
                tokio::select! {
                    Some(command) = command_rx.recv() => match command {
                        RelayProbeCommand::Dial { peer_id, addr, sender } => {
                            if pending_dial.contains_key(&peer_id) {
                                let _ = sender.send(Err("peer already pending dial".to_string()));
                                continue;
                            }
                            match swarm.dial(addr.clone()) {
                                Ok(()) => {
                                    pending_dial.insert(peer_id, sender);
                                }
                                Err(error) => {
                                    let _ = sender.send(Err(error.to_string()));
                                }
                            }
                        }
                        RelayProbeCommand::ListenOn { addr, sender } => {
                            match swarm.listen_on(addr.clone()) {
                                Ok(_) => {
                                    let _ = sender.send(Ok(()));
                                }
                                Err(error) => {
                                    let _ = sender.send(Err(error.to_string()));
                                }
                            }
                        }
                        RelayProbeCommand::Request { peer_id, message, sender } => {
                            let request_id = swarm
                                .behaviour_mut()
                                .request_response
                                .send_request(&peer_id, message);
                            pending_request.insert(request_id, sender);
                        }
                    },
                    event = swarm.select_next_some() => match event {
                        SwarmEvent::Behaviour(RelayProbeBehaviourEvent::Relay(RelayClientEvent::ReservationReqAccepted { relay_peer_id, .. })) => {
                            let _ = status_tx.send(format!("ReservationReqAccepted:{relay_peer_id}")).await;
                        }
                        SwarmEvent::Behaviour(RelayProbeBehaviourEvent::Relay(RelayClientEvent::OutboundCircuitEstablished { relay_peer_id, .. })) => {
                            let _ = status_tx.send(format!("OutboundCircuitEstablished:{relay_peer_id}")).await;
                        }
                        SwarmEvent::Behaviour(RelayProbeBehaviourEvent::Relay(RelayClientEvent::InboundCircuitEstablished { src_peer_id, .. })) => {
                            let _ = status_tx.send(format!("InboundCircuitEstablished:{src_peer_id}")).await;
                        }
                        SwarmEvent::Behaviour(RelayProbeBehaviourEvent::Dcutr(event)) => {
                            let _ = status_tx.send(format!("DCUtR:{event:?}")).await;
                        }
                        SwarmEvent::Behaviour(RelayProbeBehaviourEvent::RequestResponse(
                            request_response::Event::Message { message, .. },
                        )) => match message {
                            request_response::Message::Request { request, channel, .. } => {
                                swarm
                                    .behaviour_mut()
                                    .request_response
                                    .send_response(channel, request.clone())
                                    .expect("response channel to stay open");
                            }
                            request_response::Message::Response { request_id, response } => {
                                println!("relay probe received trace response for request {request_id:?}: {response:?}");
                                if let Some(sender) = pending_request.remove(&request_id) {
                                    let _ = sender.send(Ok(response));
                                }
                            }
                        },
                        SwarmEvent::Behaviour(RelayProbeBehaviourEvent::RequestResponse(
                            request_response::Event::OutboundFailure { request_id, error, .. },
                        )) => {
                            if let Some(sender) = pending_request.remove(&request_id) {
                                let _ = sender.send(Err(error.to_string()));
                            }
                        }
                        SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
                            if endpoint.is_dialer() {
                                if let Some(sender) = pending_dial.remove(&peer_id) {
                                    let _ = sender.send(Ok(()));
                                }
                            }
                        }
                        SwarmEvent::OutgoingConnectionError { peer_id: Some(peer_id), error, .. } => {
                            if let Some(sender) = pending_dial.remove(&peer_id) {
                                let _ = sender.send(Err(error.to_string()));
                            }
                        }
                        SwarmEvent::NewListenAddr { .. } => {}
                        _ => {}
                    }
                }
            }
        });

        RelayProbePeer {
            peer_id,
            command_tx,
            status_rx,
        }
    }

    async fn next_chat_message(
        recv: &mut mpsc::Receiver<ChatEvent>,
        timeout: Duration,
    ) -> ChatEvent {
        let deadline = tokio::time::Instant::now() + timeout;

        loop {
            let now = tokio::time::Instant::now();
            let remaining = deadline.saturating_duration_since(now);
            let event = tokio::time::timeout(remaining, recv.recv())
                .await
                .expect("Timeout waiting for chat event")
                .expect("Channel closed before receiving chat event");

            if matches!(event, ChatEvent::ChatMessage(_)) {
                return event;
            }
        }
    }

    fn real_nip34_message() -> (Msg, Event, git2::Oid) {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "gnostr-chat-nip34-{}-{unique}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("create nip34 repo root");

        let repo = git2::Repository::init(&root).expect("init nip34 repo");
        {
            let mut config = repo.config().expect("nip34 config");
            config
                .set_str("user.name", "gnostr-nip34")
                .expect("set nip34 user");
            config
                .set_str("user.email", "nip34@gnostr.org")
                .expect("set nip34 email");
        }

        let repo_path: RepoPath = root.to_str().expect("nip34 repo path").into();

        let note_file = root.join("nip34.txt");
        let mut file = fs::File::create(&note_file).expect("nip34 file");
        writeln!(file, "real nip34 commit {}", root.display()).expect("write nip34 file");

        stage_add_file(&repo_path, Path::new("nip34.txt")).expect("stage nip34 file");
        let commit_id = commit(&repo_path, "real nip34 commit").expect("nip34 commit");

        let note_body = format!("real note for {commit_id}");
        add_note(&repo_path, commit_id, &note_body, None, false).expect("nip34 note");
        let note = show_note(&repo_path, commit_id, None)
            .expect("show nip34 note")
            .expect("nip34 note exists");
        let git_note: AsyncGitNote = (&note).into();
        let private_key = PrivateKey::generate();
        let event = generate_git_note_event(&git_note, &private_key).expect("git note event");
        let msg = Msg {
            from: git_note.note.author.clone(),
            ..Msg::default()
        }
        .set_nostr_event(event.clone());

        (msg, event, commit_id.into())
    }

    #[tokio::test]
    #[ignore]
    async fn test_p2p_connectivity_two_nodes() {
        // Create channels for two chat instances
        let (send_tx1, send_rx1) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx1, _recv_rx1) = mpsc::channel::<ChatEvent>(100);
        let (_send_tx2, send_rx2) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<ChatEvent>(100);

        let topic = gossipsub::IdentTopic::new("test-p2p-topic-two-nodes");

        // Spawn the event loops for two peers
        tokio::spawn(evt_loop(send_rx1, recv_tx1, topic.clone()));
        tokio::spawn(evt_loop(send_rx2, recv_tx2, topic.clone()));

        // Give some time for peers to discover each other via mDNS.
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Send a message from peer 1
        let msg1_content = "Hello from peer 1";
        let msg1 = Msg {
            from: "peer1".to_string(),
            ..Msg::default()
        }
        .set_content(msg1_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx1
            .send(ChatEvent::ChatMessage(msg1))
            .await
            .unwrap();

        // Receive the message on peer 2's side
        let received_event = next_chat_message(&mut recv_rx2, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg) = received_event {
            assert_eq!(received_msg.from, "peer1");
            assert_eq!(received_msg.content[0], msg1_content);
            assert_eq!(received_msg.kind, MsgKind::Chat);
        } else {
            panic!("Received wrong event type on peer 2: {:?}", received_event);
        }

        // Send a message from peer 2
        let msg2_content = "Hello from peer 2";
        let msg2 = Msg {
            from: "peer2".to_string(),
            ..Msg::default()
        }
        .set_content(msg2_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx2
            .send(ChatEvent::ChatMessage(msg2))
            .await
            .unwrap();

        // Receive the message on peer 1's side
        let received_event_2 = next_chat_message(&mut recv_rx1, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg_2) = received_event_2 {
            assert_eq!(received_msg_2.from, "peer2");
            assert_eq!(received_msg_2.content[0], msg2_content);
            assert_eq!(received_msg_2.kind, MsgKind::Chat);
        } else {
            panic!(
                "Received wrong event type on peer 1: {:?}",
                received_event_2
            );
        }
    }

    #[tokio::test]
    #[cfg(feature = "long_tests")]
    #[ignore]
    async fn test_p2p_connectivity_two_nodes_with_local_relay() {
        let _relay = spawn_local_p2p_relay_service_async()
            .await
            .expect("local p2p relay service");
        tokio::time::sleep(Duration::from_secs(5)).await;

        let (send_tx1, send_rx1) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx1, mut recv_rx1) = mpsc::channel::<ChatEvent>(100);
        let (send_tx2, send_rx2) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<ChatEvent>(100);

        let topic = gossipsub::IdentTopic::new("test-p2p-topic-two-nodes-relay");

        tokio::spawn(evt_loop(send_rx1, recv_tx1, topic.clone()));
        tokio::spawn(evt_loop(send_rx2, recv_tx2, topic.clone()));

        tokio::time::sleep(Duration::from_secs(8)).await;

        let msg1_content = "Hello from relay-backed peer 1";
        let msg1 = Msg {
            from: "relay-peer1".to_string(),
            ..Msg::default()
        }
        .set_content(msg1_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx1
            .send(ChatEvent::ChatMessage(msg1))
            .await
            .unwrap();

        let received_event = next_chat_message(&mut recv_rx2, Duration::from_secs(15)).await;
        if let ChatEvent::ChatMessage(received_msg) = received_event {
            assert_eq!(received_msg.from, "relay-peer1");
            assert_eq!(received_msg.content[0], msg1_content);
            assert_eq!(received_msg.kind, MsgKind::Chat);
        } else {
            panic!("Received wrong event type on relay-backed peer 2: {:?}", received_event);
        }

        let msg2_content = "Hello from relay-backed peer 2";
        let msg2 = Msg {
            from: "relay-peer2".to_string(),
            ..Msg::default()
        }
        .set_content(msg2_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx2
            .send(ChatEvent::ChatMessage(msg2))
            .await
            .unwrap();

        let received_event_2 = next_chat_message(&mut recv_rx1, Duration::from_secs(15)).await;
        if let ChatEvent::ChatMessage(received_msg_2) = received_event_2 {
            assert_eq!(received_msg_2.from, "relay-peer2");
            assert_eq!(received_msg_2.content[0], msg2_content);
            assert_eq!(received_msg_2.kind, MsgKind::Chat);
        } else {
            panic!(
                "Received wrong event type on relay-backed peer 1: {:?}",
                received_event_2
            );
        }
    }

    #[tokio::test]
    #[cfg(feature = "long_tests")]
    #[ignore]
    async fn test_p2p_connectivity_two_nodes_with_local_relay_nip34_event() {
        let _relay = spawn_local_p2p_relay_service_async()
            .await
            .expect("local p2p relay service");
        tokio::time::sleep(Duration::from_secs(5)).await;

        let (send_tx1, send_rx1) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx1, mut recv_rx1) = mpsc::channel::<ChatEvent>(100);
        let (send_tx2, send_rx2) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<ChatEvent>(100);

        let topic = gossipsub::IdentTopic::new("test-p2p-topic-two-nodes-relay-nip34");

        tokio::spawn(evt_loop(send_rx1, recv_tx1, topic.clone()));
        tokio::spawn(evt_loop(send_rx2, recv_tx2, topic.clone()));

        tokio::time::sleep(Duration::from_secs(8)).await;

        let (msg1, event1, commit1) = real_nip34_message();
        send_tx1
            .send(ChatEvent::ChatMessage(msg1.clone()))
            .await
            .expect("send real nip34 message from peer 1");

        let received_event = next_chat_message(&mut recv_rx2, Duration::from_secs(15)).await;
        if let ChatEvent::ChatMessage(received_msg) = received_event {
            assert_eq!(received_msg.from, msg1.from);
            assert_eq!(received_msg.kind, MsgKind::NostrEvent);

            let received_nostr_event = received_msg
                .nostr_event
                .as_ref()
                .expect("real nostr event to survive transport");
            assert_eq!(received_nostr_event.kind, event1.kind);
            assert_eq!(received_nostr_event.content, event1.content);
            assert_eq!(received_nostr_event.id, event1.id);
            assert!(
                received_nostr_event
                    .tags
                    .iter()
                    .any(|tag| tag.tagname() == "commit" && tag.value() == commit1.to_string())
            );
            assert_eq!(
                received_msg.content.first(),
                Some(&serde_json::to_string(received_nostr_event).expect("event json"))
            );
        } else {
            panic!("received wrong event type on peer 2: {:?}", received_event);
        }
    }

    #[tokio::test]
    #[cfg(feature = "long_tests")]
    #[ignore]
    async fn test_p2p_relay_reservation_and_circuit_round_trip() {
        let relay = spawn_local_p2p_relay_service_async()
            .await
            .expect("local p2p relay service");
        let relay_addr = relay.listen_addr().clone();
        let relay_peer_id = relay.peer_id();
        let peer_one_reservation_addr =
            relay_addr.clone().with(libp2p::multiaddr::Protocol::P2p(relay_peer_id)).with(libp2p::multiaddr::Protocol::P2pCircuit);
        let peer_two_reservation_addr =
            relay_addr.clone().with(libp2p::multiaddr::Protocol::P2p(relay_peer_id)).with(libp2p::multiaddr::Protocol::P2pCircuit);

        let mut peer_one = spawn_relay_probe_peer("chat-relay-peer-one");
        let mut peer_two = spawn_relay_probe_peer("chat-relay-peer-two");

        peer_one
            .listen_on(peer_one_reservation_addr.clone())
            .await
            .expect("peer one relay listen");
        peer_two
            .listen_on(peer_two_reservation_addr.clone())
            .await
            .expect("peer two relay listen");

        let peer_one_status = peer_one
            .wait_for_status_contains("ReservationReqAccepted", Duration::from_secs(20))
            .await;
        let peer_two_status = peer_two
            .wait_for_status_contains("ReservationReqAccepted", Duration::from_secs(20))
            .await;
        assert!(
            peer_one_status.contains("ReservationReqAccepted"),
            "peer one relay status: {peer_one_status}"
        );
        assert!(
            peer_two_status.contains("ReservationReqAccepted"),
            "peer two relay status: {peer_two_status}"
        );

        let peer_two_circuit = build_relay_circuit_addr(&relay_addr, relay_peer_id, peer_two.peer_id);
        peer_one
            .dial(peer_two.peer_id, peer_two_circuit.clone())
            .await
            .expect("peer one relay circuit dial");

        let circuit_status = peer_one
            .wait_for_status_contains("OutboundCircuitEstablished", Duration::from_secs(20))
            .await;
        assert!(
            circuit_status.contains("OutboundCircuitEstablished"),
            "peer one circuit status: {circuit_status}"
        );

        for (index, envelope) in relay_trace_envelopes().into_iter().enumerate() {
            println!("relay probe sending trace #{index}: {envelope:?}");
            let response = peer_one
                .request(peer_two.peer_id, envelope.clone())
                .await
                .expect("relay request to round-trip");
            println!("relay probe received trace #{index}: {response:?}");
            assert_eq!(response, envelope);
        }

        let punch_status = peer_two
            .wait_for_status_contains("InboundCircuitEstablished", Duration::from_secs(20))
            .await;
        assert!(
            punch_status.contains("InboundCircuitEstablished") || punch_status.contains("DCUtR"),
            "peer two relay/punch status: {punch_status}"
        );
    }

    #[tokio::test]
    #[cfg(feature = "long_tests")]
    #[ignore]
    async fn test_p2p_connectivity_three_nodes() {
        // Create channels for three chat instances
        let (send_tx1, send_rx1) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx1, mut _recv_rx1) = mpsc::channel::<ChatEvent>(100);
        let (send_tx2, send_rx2) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx2, mut recv_rx2) = mpsc::channel::<ChatEvent>(100);
        let (send_tx3, send_rx3) = mpsc::channel::<ChatEvent>(100);
        let (recv_tx3, mut recv_rx3) = mpsc::channel::<ChatEvent>(100);

        let topic = gossipsub::IdentTopic::new("test-p2p-topic-three-nodes");

        // Spawn the event loops for three peers
        tokio::spawn(evt_loop(send_rx1, recv_tx1, topic.clone()));
        tokio::spawn(evt_loop(send_rx2, recv_tx2, topic.clone()));
        tokio::spawn(evt_loop(send_rx3, recv_tx3, topic.clone()));

        // Give some time for peers to discover each other.
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Send a message from peer 1
        let msg1_content = "Hello from peer 1 to all";
        let msg1 = Msg {
            from: "peer1".to_string(),
            ..Msg::default()
        }
        .set_content(msg1_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        send_tx1
            .send(ChatEvent::ChatMessage(msg1))
            .await
            .unwrap();

        // Peer 2 should receive the message
        let received_event_2 = next_chat_message(&mut recv_rx2, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg) = received_event_2 {
            assert_eq!(received_msg.from, "peer1");
            assert_eq!(received_msg.content[0], msg1_content);
        } else {
            panic!(
                "Received wrong event type on peer 2: {:?}",
                received_event_2
            );
        }

        // Peer 3 should also receive the message
        let received_event_3 = next_chat_message(&mut recv_rx3, Duration::from_secs(10)).await;

        if let ChatEvent::ChatMessage(received_msg) = received_event_3 {
            assert_eq!(received_msg.from, "peer1");
            assert_eq!(received_msg.content[0], msg1_content);
        } else {
            panic!(
                "Received wrong event type on peer 3: {:?}",
                received_event_3
            );
        }

        // Test multi-sender scenarios: Send messages from peers 2 and 3
        let msg2_content = "Response from peer 2";
        let msg2 = Msg {
            from: "peer2".to_string(),
            ..Msg::default()
        }
        .set_content(msg2_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        let msg3_content = "Response from peer 3";
        let msg3 = Msg {
            from: "peer3".to_string(),
            ..Msg::default()
        }
        .set_content(msg3_content.to_string(), 0)
        .set_kind(MsgKind::Chat);

        // Send messages concurrently from peers 2 and 3
        let (tx2_result, tx3_result) = tokio::join!(
            send_tx2.send(ChatEvent::ChatMessage(msg2)),
            send_tx3.send(ChatEvent::ChatMessage(msg3))
        );
        tx2_result.unwrap();
        tx3_result.unwrap();

        // Peer 1 should receive messages from both peers 2 and 3
        let mut received_messages = Vec::new();
        for _ in 0..2 {
            let received_event = next_chat_message(&mut _recv_rx1, Duration::from_secs(10)).await;

            if let ChatEvent::ChatMessage(msg) = received_event {
                received_messages.push(msg);
            } else {
                panic!("Received wrong event type on peer 1: {:?}", received_event);
            }
        }

        // Verify we received both messages (order may vary due to network timing)
        let mut received_from_2 = false;
        let mut received_from_3 = false;
        for msg in &received_messages {
            if msg.from == "peer2" {
                assert_eq!(msg.content[0], msg2_content);
                received_from_2 = true;
            } else if msg.from == "peer3" {
                assert_eq!(msg.content[0], msg3_content);
                received_from_3 = true;
            }
        }
        assert!(received_from_2, "Did not receive message from peer 2");
        assert!(received_from_3, "Did not receive message from peer 3");
    }
}
