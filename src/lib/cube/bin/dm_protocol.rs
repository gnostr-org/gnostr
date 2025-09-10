use getopts::Options;
use libp2p::core::transport::{upgrade, PortUse};
use libp2p::futures::future::BoxFuture;
use libp2p::futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::swarm::{ConnectionDenied, ConnectionId, DialFailure, FromSwarm, NetworkBehaviour, NotifyHandler, OneShotHandler, Swarm, SwarmEvent, THandler, THandlerInEvent, THandlerOutEvent, ToSwarm};
use libp2p::{core::Endpoint, core::UpgradeInfo, identity, mdns, noise, swarm, tcp, yamux, InboundUpgrade, Multiaddr, OutboundUpgrade, PeerId, StreamProtocol, Transport};
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{env, io, iter};
use tokio::io::{AsyncBufReadExt};

const PROTOCOL: StreamProtocol = StreamProtocol::new("/direct/1.0");
const MAX_BUF_SIZE: usize = 2_097_152;

#[derive(Default, Clone, Debug)]
pub struct DirectMessage(String);

impl UpgradeInfo for DirectMessage {
    type Info = StreamProtocol;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(PROTOCOL)
    }
}

impl<TSocket> OutboundUpgrade<TSocket> for DirectMessage
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = ();
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_outbound(self, mut socket: TSocket, _info: Self::Info) -> Self::Future {
        Box::pin(async move {
            let message = self.0.as_bytes();
            socket.write_all(message).await?;
            socket.flush().await?;
            Ok(())
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct DirectMessageProtocol;

impl UpgradeInfo for DirectMessageProtocol {
    type Info = StreamProtocol;
    type InfoIter = iter::Once<Self::Info>;

    fn protocol_info(&self) -> Self::InfoIter {
        iter::once(PROTOCOL)
    }
}

impl<TSocket> InboundUpgrade<TSocket> for DirectMessageProtocol
where
    TSocket: AsyncRead + AsyncWrite + Send + Unpin + 'static,
{
    type Output = DirectMessage;
    type Error = io::Error;
    type Future = BoxFuture<'static, Result<Self::Output, Self::Error>>;

    fn upgrade_inbound(self, mut socket: TSocket, _: Self::Info) -> Self::Future {
        Box::pin(async move {
            let mut buffer = vec![0u8; MAX_BUF_SIZE];
            let n = socket.read(&mut buffer).await?;
            let message = String::from_utf8_lossy(&buffer[..n]).to_string();
            Ok(DirectMessage(message))
        })
    }
}

// Define our custom behavior
#[derive(NetworkBehaviour)]
#[behaviour(out_event = "MyBehaviourEvent")]
struct MyBehaviour {
    mdns: mdns::async_io::Behaviour,
    direct_message: DirectMessageBehaviour,
}

#[derive(Debug)]
enum MyBehaviourEvent {
    Mdns(mdns::Event),
    DirectMessage(DirectMessageEvent),
}

impl From<mdns::Event> for MyBehaviourEvent {
    fn from(event: mdns::Event) -> Self {
        MyBehaviourEvent::Mdns(event)
    }
}

impl From<DirectMessageEvent> for MyBehaviourEvent {
    fn from(event: DirectMessageEvent) -> Self {
        MyBehaviourEvent::DirectMessage(event)
    }
}

// Custom protocol for direct messaging
#[derive(Default)]
struct DirectMessageBehaviour {
    events: VecDeque<DirectMessageEvent>,
    pending_messages: VecDeque<(PeerId, String)>,
    connected_peers: HashMap<PeerId, ConnectionId>,
}

#[derive(Debug)]
enum DirectMessageEvent {
    MessageReceived { peer: Option<PeerId>, message: String },
    MessageSent { peer: Option<PeerId> },
}

impl From<DirectMessage> for DirectMessageEvent {
    fn from(msg: DirectMessage) -> Self {
        DirectMessageEvent::MessageReceived {
            peer: None,
            message: msg.0,
        }
    }
}

impl From<()> for DirectMessageEvent {
    fn from(_: ()) -> Self {
        DirectMessageEvent::MessageSent { peer: None }
    }
}

impl DirectMessageBehaviour {
    fn send_message(&mut self, peer: &PeerId, message: String) {
        self.pending_messages.push_back((*peer, message));
    }

    fn on_connection_established(&mut self, peer: PeerId, connection: ConnectionId) {
        self.connected_peers.insert(peer, connection);
    }

    fn on_connection_closed(&mut self, peer: &PeerId) {
        self.connected_peers.remove(peer);
    }

    fn on_dial_failure(
        &mut self,
        DialFailure {
            connection_id,
            peer_id,
            error
        }: DialFailure,
    ) {
        let Some(peer_id) = peer_id else {
            return;
        };

        if self.connected_peers.contains_key(&peer_id) {
            return;
        }
    }
}

impl NetworkBehaviour for DirectMessageBehaviour {
    type ConnectionHandler = OneShotHandler<DirectMessageProtocol, DirectMessage, DirectMessageEvent>;
    type ToSwarm = DirectMessageEvent;

    fn handle_established_inbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        _local_addr: &Multiaddr,
        _remote_addr: &Multiaddr,
    ) -> Result<Self::ConnectionHandler, ConnectionDenied> {
        println!("Inbound connection established with peer: {:?}", peer);
        self.on_connection_established(peer, connection_id);
        Ok(OneShotHandler::default())
    }

    fn handle_established_outbound_connection(
        &mut self,
        connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
        port_use: PortUse,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        println!("Outbound connection established with peer: {:?}", peer);
        self.on_connection_established(peer, connection_id);
        Ok(OneShotHandler::default())
    }

    fn on_swarm_event(&mut self, event: FromSwarm) {
        match event {
            FromSwarm::ConnectionClosed(info) => {
                println!("Connection closed with peer: {:?}", info.peer_id);
                self.on_connection_closed(&info.peer_id);
            }
            FromSwarm::ConnectionEstablished(info) => {
                println!("Connection established with peer: {:?}", info.peer_id);
            }
            FromSwarm::DialFailure(info) => {
                self.on_dial_failure(info);
            }
            _ => {
                println!("Unhandled event: {:?}", event);
            }
        }
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        _connection_id: ConnectionId,
        event: THandlerOutEvent<Self>,
    ) {
        match event {
            Ok(DirectMessageEvent::MessageReceived { message, .. }) => {
                self.events.push_back(DirectMessageEvent::MessageReceived {
                    peer: Some(peer_id),
                    message,
                });
            }
            Ok(DirectMessageEvent::MessageSent { .. }) => {
                self.events.push_back(DirectMessageEvent::MessageSent { peer: Some(peer_id) });
            }
            Err(e) => {
                eprintln!("Error in connection handler: {:?}", e);
            }
        }
    }

    fn poll(
        &mut self,
        _cx: &mut Context<'_>,
    ) -> Poll<ToSwarm<Self::ToSwarm, THandlerInEvent<Self>>> {
        if let Some(event) = self.events.pop_front() {
            return Poll::Ready(ToSwarm::GenerateEvent(event));
        }
        if let Some((peer, message)) = self.pending_messages.pop_front() {
            if let Some(&connection_id) = self.connected_peers.get(&peer) {
                return Poll::Ready(ToSwarm::NotifyHandler {
                    peer_id: peer,
                    handler: NotifyHandler::One(connection_id),
                    event: DirectMessage(message),
                });
            }
        }
        Poll::Pending
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();

    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("p", "port", "set the port number", "PORT");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!("{}", f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return Ok(());
    }

    let port = matches.opt_str("p")
        .map(|p| p.parse::<u16>().expect("Port must be a number"))
        .unwrap_or(1080); // Default port if not specified

    println!("Using port: {}", port);

    let local_key = identity::Keypair::generate_ed25519();
    let local_peer_id = PeerId::from(local_key.public());
    println!("Local peer id: {:?}", local_peer_id);

    let transport = tcp::async_io::Transport::new(tcp::Config::default())
        .upgrade(upgrade::Version::V1)
        .authenticate(noise::Config::new(&local_key).unwrap())
        .multiplex(yamux::Config::default())
        .boxed();

    let behaviour = MyBehaviour {
        mdns: mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id)?,
        direct_message: DirectMessageBehaviour::default(),
    };

    let cfg = swarm::Config::with_tokio_executor();
    let cfg = cfg.with_idle_connection_timeout(Duration::from_secs(600));
    let mut swarm = Swarm::new(transport, behaviour, local_peer_id, cfg);

    let s = format!("/ip4/127.0.0.1/tcp/{}", port);
    swarm.listen_on(s.parse()?)?;

    // Read full lines from stdin
    let mut stdin = tokio::io::BufReader::new(tokio::io::stdin()).lines();

    // Event loop
    loop {
        tokio::select! {
            line = stdin.next_line() => {
                let line = line?.expect("stdin closed");
                if let Some((peer_id, message)) = line.split_once(' ') {
                    if let Ok(peer_id) = peer_id.parse() {
                        swarm.behaviour_mut().direct_message.send_message(&peer_id, message.to_string());
                        for p in swarm.connected_peers() {
                            println!("PEER {:?}", p);
                        }
                        for (peer_id, _) in swarm.behaviour().direct_message.connected_peers.iter() {
                            println!("Peer<-: {peer_id}");
                        }
                        println!("Sending message to {:?}: {}", peer_id, message);
                    }
                }
            }
            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr { address, .. } => {
                        println!("Listening on {:?}", address);
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                        for (peer_id, multiaddr) in list {
                            println!("mDNS discovered a new peer: {peer_id}");
                            swarm.add_peer_address(peer_id, multiaddr.clone());
                            swarm.dial(multiaddr).expect("TODO: panic message");
                        }
                    }
                    SwarmEvent::Behaviour(MyBehaviourEvent::DirectMessage(event)) => {
                        match event {
                            DirectMessageEvent::MessageReceived {peer, message} => {
                                println!("Received message from {:?}: {:?}", peer, message);
                            }
                            DirectMessageEvent::MessageSent { peer } => {
                                println!("Sent message to {:?}", peer);
                            }
                        }
                    }
                    _ => {
                        println!("Swarm event: {:?}", event);
                    }
                }
            }
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

