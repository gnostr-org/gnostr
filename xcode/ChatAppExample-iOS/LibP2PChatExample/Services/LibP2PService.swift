//
//  LibP2PService.swift
//  LibP2PChatExample
//
//  Created by Brandon Toms on 5/29/22.
//

import Foundation
import LibP2P
import LibP2PNoise
import LibP2PMPLEX
import LibP2PRelay
import LibP2PAutoNAT
import LibP2PDCUtR
import LibP2PDNSAddr
import LibP2PMDNS
import LibP2PKadDHT
import LibP2PPubSub

/// Any class that conforms to the ChatDelegate can register themselves on the LibP2PService to get notified of Chat events
protocol ChatDelegate {
    func on(message:String, from:PeerID)
    func on(nickname:String, from:PeerID)
}

protocol TopicDelegate {
    func on(topicMessage message: String, from: PeerID, topic: String)
    func on(topicPeerJoined peer: PeerID, topic: String)
}

/// We extend the Request struct with a computed var that provides access to a shared instance of our LibP2PService.
/// This allows us to interact with the LibP2PService without our Route handlers.
extension Request {
    var myService: LibP2PService { LibP2PService.shared }
}

/// We create a simple LibP2PService Singleton that is responsible for...
/// - starting and stoping libp2p
/// - configuring our libp2p networking stack
/// - registering our Route handlers
/// - listening for peer discovery events
/// - sending messages to connected peers
class LibP2PService {
    static let shared = LibP2PService()

    private enum LifecycleState {
        case stopped
        case starting
        case running
        case stopping
    }

    public enum PeerConnectionState: Equatable {
        case disconnected
        case dialing
        case connected
    }

    private var app:Application
    private let peerID: PeerID
    private var lna: LocalNetworkAuthorization?
    
    internal var delegate:ChatDelegate? = nil {
        didSet { installRuntimeHandlersIfNeeded() }
    }
    
    private var pingTask:RepeatedTask? = nil
    private var runtimeHandlersInstalled = false
    private var lifecycleState: LifecycleState = .stopped
    private var topologyRegistrations: [TopologyRegistration] = []
    private var subscribedTopics: Set<String> = []
    private var topicSubscriptions: [String: PubSub.SubscriptionHandler] = [:]
    private let defaultTopic = "gnostr"
    private var discoveredPeerAddresses: [String: Multiaddr] = [:]
    private let discoveredPeerAddressesQueue = DispatchQueue(label: "LibP2PService.discoveredPeerAddresses")
    @Published private var peerConnectionStates: [String: PeerConnectionState] = [:]

    internal var topicDelegate: TopicDelegate? = nil
    
    public var savedPeerID:PeerID? {
        if let pid = UserDefaults.standard.data(forKey: "MyPeerID") {
            return try? PeerID(marshaledPrivateKey: pid)
        } else if let pem = UserDefaults.standard.string(forKey: "MyPeerID") {
            return try? PeerID(pem: pem, password: "Test123")
        } else {
            return nil
        }
    }
    
    private init() {
        let peerID: PeerID
        if let existingPeerID = UserDefaults.standard.data(forKey: "MyPeerID") {
            peerID = try! PeerID(marshaledPrivateKey: existingPeerID)
        } else if let existingPeerID = UserDefaults.standard.string(forKey: "MyPeerID") {
            peerID = try! PeerID(pem: existingPeerID, password: "Test123")
        } else {
            peerID = try! PeerID(.Ed25519)
            if let pem = try? peerID.exportKeyPair(as: .privatePEMString(encryptedWithPassword: "Test123")) {
                UserDefaults.standard.set(String(pem), forKey: "MyPeerID")
            }
        }
        self.peerID = peerID
        self.app = Self.makeApplication(peerID: peerID)
        self.lna = LocalNetworkAuthorization()
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        app.connectionManager.setIdleTimeout(.seconds(30))
        app.security.use(.noise)
        app.muxers.use(.mplex)
        app.relay.use(.relay)
        app.autonat.use(.autonat)
        app.dcutr.use(.dcutr)
        app.discovery.use(.bootstrap(Self.bootstrapPeers))
        app.dht.use(.kadDHT(mode: .client, bootstrapPeers: Self.bootstrapPeers))
        app.pubsub.use(.gossipsub)
        app.resolvers.use(.dnsaddr)
        app.discovery.use(.mdns)
        app.servers.use(.tcp(host: "0.0.0.0", port: Self.listenPort))
        try! routes(app)
        return app
    }

    private static var bootstrapPeers: [PeerInfo] {
        BootstrapPeerDiscovery.IPFSBootNodes
    }

    private static var listenPort: Int {
#if targetEnvironment(simulator)
        return 10001
#else
        return 0
#endif
    }

    private func installRuntimeHandlersIfNeeded() {
        guard !self.runtimeHandlersInstalled, let delegate = self.delegate else { return }

        self.app.discovery.onPeerDiscovered(self.app) { peer in
            self.app.logger.notice("We discovered a peer: \(peer)")
            self.app.peers.add(key: peer.peer, on: nil).flatMap { _ in
                self.app.peers.add(addresses: peer.addresses, toPeer: peer.peer, on: nil)
            }.whenComplete { _ in
                self.app.connections.getConnectionsToPeer(peer: peer.peer, on: nil).whenSuccess { conns in
                if conns.isEmpty {
                    self.app.logger.notice("Dialing peer \(peer.peer) using peerstore addresses")
                    do {
                        try self.app.newStream(to: peer.peer, forProtocol: "/chat/1.0.0")
                    } catch {
                        self.app.logger.error("Failed to dial peer \(peer.peer): \(error)")
                    }
                } else {
                    self.markPeerConnected(peer.peer)
                }
                }
            }
        }

        self.app.events.on(self, event: .disconnected({ conn, peerID in
            if let peerID = peerID { let _ = self.app.peers.removeAllAddresses(forPeer: peerID) }
        }))

        self.pingTask?.cancel()
        self.pingTask = self.app.eventLoopGroup.any().scheduleRepeatedTask(initialDelay: .seconds(15), delay: .seconds(15), { _ in
            self.pingDiscoveredUsers()
        })

        self.runtimeHandlersInstalled = true
        self.app.logger.notice("Installed runtime handlers for delegate \(String(describing: delegate))")
    }
    
    public func deletePeerID() {
        UserDefaults.standard.removeObject(forKey: "MyPeerID")
    }
    
    public func register(register:AnyObject, event: EventBus.EventHandler) {
        self.app.events.on(register, event: event)
    }
    
    public func topology(_ reg:TopologyRegistration) {
        self.topologyRegistrations.append(reg)
        self.app.topology.register(reg)
    }

    private func reinstallTopologyRegistrations() {
        for registration in self.topologyRegistrations {
            self.app.topology.register(registration)
        }
    }

    private func reinstallTopicSubscriptions() {
        for topic in self.subscribedTopics {
            self.joinTopicIfNeeded(topic)
        }
    }

    public func connectionState(for peerID: PeerID) -> PeerConnectionState {
        self.peerConnectionStates[peerID.b58String] ?? .disconnected
    }

    private func recordDiscoveredAddress(_ address: Multiaddr, for peerID: PeerID) {
        self.discoveredPeerAddressesQueue.sync {
            self.discoveredPeerAddresses[peerID.b58String] = address
        }
    }

    private func discoveredAddress(for peerID: PeerID) -> Multiaddr? {
        self.discoveredPeerAddressesQueue.sync {
            self.discoveredPeerAddresses[peerID.b58String]
        }
    }

    private func markPeerConnected(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .connected
        }
    }

    private func markPeerDialing(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .dialing
        }
    }

    private func markPeerDisconnected(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .disconnected
        }
    }

    private var needsLocalNetworkAuthorization: Bool {
#if os(iOS) && !targetEnvironment(macCatalyst)
        return true
#else
        return false
#endif
    }

    private func joinTopicIfNeeded(_ topic: String) {
        guard self.topicSubscriptions[topic] == nil else { return }

        do {
            let subscription = try self.app.pubsub.gossipsub.subscribe(
                .init(
                    topic: topic,
                    signaturePolicy: .strictSign,
                    validator: .acceptAll,
                    messageIDFunc: .concatFromAndSequenceFields
                )
            )

            let eventLoop = self.app.eventLoopGroup.next()
            subscription.on = { [weak self] event in
                guard let self else { return eventLoop.makeSucceededVoidFuture() }
                switch event {
                case .newPeer(let peer):
                    self.app.logger.notice("[LibP2PTopic] \(topic) discovered peer: \(peer.b58String)")
                    self.topicDelegate?.on(topicPeerJoined: peer, topic: topic)
                case .data(let message):
                    guard let fromPeer = try? PeerID(fromBytesID: Array(message.from)) else {
                        self.app.logger.warning("[LibP2PTopic] Invalid sender for topic \(topic)")
                        return eventLoop.makeSucceededVoidFuture()
                    }
                    let text = String(data: message.data, encoding: .utf8) ?? ""
                    self.topicDelegate?.on(topicMessage: text, from: fromPeer, topic: topic)
                case .error(let error):
                    self.app.logger.error("[LibP2PTopic] \(topic) subscription error: \(error)")
                }
                return eventLoop.makeSucceededVoidFuture()
            }

            self.topicSubscriptions[topic] = subscription
            self.subscribedTopics.insert(topic)
            self.app.logger.notice("[LibP2PTopic] Joined gossip topic \(topic)")
            self.topicDelegate?.on(topicPeerJoined: self.app.peerID, topic: topic)
            self.app.pubsub.gossipsub.getPeersSubscribed(to: topic, on: eventLoop).whenSuccess { peers in
                for peer in peers {
                    self.topicDelegate?.on(topicPeerJoined: peer, topic: topic)
                }
            }
        } catch {
            self.app.logger.error("[LibP2PTopic] Failed to join topic \(topic): \(error)")
        }
    }
    
    public func start() async throws {
        if self.needsLocalNetworkAuthorization {
            guard await self.lna?.requestAuthorization() ?? true else {
                throw CocoaError(.userCancelled)
            }
        }
        guard self.lifecycleState != .running && self.lifecycleState != .starting else { return }
        self.lifecycleState = .starting
        if self.app.didShutdown {
            self.app = Self.makeApplication(peerID: self.peerID)
            self.lna = self.needsLocalNetworkAuthorization ? LocalNetworkAuthorization() : nil
            self.runtimeHandlersInstalled = false
            self.topicSubscriptions = [:]
            self.reinstallTopologyRegistrations()
            self.reinstallTopicSubscriptions()
        }
        self.installRuntimeHandlersIfNeeded()
        self.joinTopicIfNeeded(self.defaultTopic)
        do {
            try app.start()
            self.app.logger.notice("LibP2P Started!")
            self.lifecycleState = .running
        } catch {
            self.lifecycleState = .stopped
            throw error
        }
    }
    
    public func stop() {
        guard self.lifecycleState == .running else { return }
        self.lifecycleState = .stopping
        self.pingTask?.cancel()
        app.shutdown()
        self.runtimeHandlersInstalled = false
        self.topicSubscriptions = [:]
        self.lifecycleState = .stopped
    }
    
    public func send(message:String, to peer:PeerID) {
        guard self.app.isRunning else { print("LibP2P needs to be running in order to send messages!"); return }
        let request = Data(message.utf8)
        let protocolID = "/chat/1.0.0"

        // Reuse an existing live connection whenever possible so chat messages do not pay
        // discovery/dial setup on every send.
        self.app.connections.getBestConnectionForPeer(peer: peer, on: self.app.eventLoopGroup.next()).flatMap { connection -> EventLoopFuture<Data> in
            if let remoteAddr = connection?.remoteAddr {
                return self.app.newRequest(
                    to: remoteAddr,
                    forProtocol: protocolID,
                    withRequest: request,
                    style: .noResponseExpected,
                    withHandlers: .inherit,
                    withTimeout: .seconds(10)
                )
            }

            return self.app.newRequest(
                to: peer,
                forProtocol: protocolID,
                withRequest: request,
                style: .noResponseExpected,
                withHandlers: .inherit,
                withTimeout: .seconds(10)
            )
        }.whenComplete { result in
            switch result {
            case .failure(let error):
                self.app.logger.error("Error: \(error)")
            case .success:
                self.app.logger.trace("Sent message to peer: \(peer)")
            }
        }
    }

    public func publish(message: String, to topic: String) {
        guard self.app.isRunning else { print("LibP2P needs to be running in order to publish messages!"); return }
        self.joinTopicIfNeeded(topic)
        guard let subscription = self.topicSubscriptions[topic] else {
            self.app.logger.error("[LibP2PTopic] No subscription available for topic \(topic)")
            return
        }
        subscription.publish(Data(message.utf8))
    }
    
    public func isConnectedTo(peer:PeerID) async -> Bool {
        await withCheckedContinuation { continuation in
            self.app.connections.connectedness(peer: peer, on: nil).whenComplete { result in
                switch result {
                case .failure(_):
                    return continuation.resume(returning: false)
                case .success(let connectedness):
                    return continuation.resume(returning: connectedness == .Connected)
                }
            }
        }
    }
    
    /// This recurring task acts as a Keep-Alive service for Peers that support the `/chat/1.0.0` protocol
    /// We Ping these peers at an interval that's shorter than our Idle Timeout set above (30 seconds) in order to keep the Connection alive
    public func pingDiscoveredUsers() {
        let _ = self.app.peers.getPeers(supportingProtocol: .init("chat/1.0.0")! ).map { peers in
            return peers.compactMap { peerID in
                self.app.connections.connectedness(peer: try! PeerID(cid: peerID), on: nil).map { connectedness -> (String, EventLoopFuture<TimeAmount>)? in
                    switch connectedness {
                    case .Connected:
                        return (peerID, self.app.identify.ping(peer: try! PeerID(cid: peerID)).always { result in
                            self.app.logger.debug("Ping Result: \(result)")
                        })
                    default:
                        return nil
                    }
                }
            }.flatten(on: self.app.eventLoopGroup.any())
        }
    }
}
