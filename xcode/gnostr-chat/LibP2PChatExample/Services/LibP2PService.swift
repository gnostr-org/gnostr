//
//  LibP2PService.swift
//  LibP2PChatExample
//
//  Created by Brandon Toms on 5/29/22.
//

import Foundation
import Crypto
import Combine
import LibP2P
import LibP2PNoise
import LibP2PMPLEX
import LibP2PRelay
import LibP2PAutoNAT
import LibP2PDCUtR
import LibP2PMDNS
import LibP2PKadDHT
import LibP2PPubSub
#if os(iOS)
import UIKit
#endif

/// Any class that conforms to the ChatDelegate can register themselves on the LibP2PService to get notified of Chat events
protocol ChatDelegate {
    func on(message:String, from:PeerID)
    func on(nickname:String, from:PeerID)
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
class LibP2PService: ObservableObject {
    static let shared = LibP2PService()

    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }

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

    private struct RustChatMessage: Codable {
        let from: String
        let content: [String]
        let kind: String
        let commitId: String
        let nostrEvent: [String: String]?
        let messageId: String?
        let sequenceNum: Int?
        let totalChunks: Int?

        init(from fromLabel: String, content: String, kind: String = "Chat") {
            self.from = fromLabel
            self.content = [content]
            self.kind = kind
            self.commitId = String(repeating: "0", count: 40)
            self.nostrEvent = nil
            self.messageId = nil
            self.sequenceNum = nil
            self.totalChunks = nil
        }

        enum CodingKeys: String, CodingKey {
            case from
            case content
            case kind
            case commitId = "commit_id"
            case nostrEvent = "nostr_event"
            case messageId = "message_id"
            case sequenceNum = "sequence_num"
            case totalChunks = "total_chunks"
        }
    }

    private var app:Application
    private let peerID: PeerID
    private var lna: LocalNetworkAuthorization?
    private var chatDisplayName: String
    private let chatTopic = "gnostr-dev"
    private var chatSubscription: PubSub.SubscriptionHandler?
    private var chatSubscribedTopic: String?
    
    internal var delegate:ChatDelegate? = nil {
        didSet { installRuntimeHandlersIfNeeded() }
    }
    
    private var pingTask:RepeatedTask? = nil
    private var runtimeHandlersInstalled = false
    private var lifecycleState: LifecycleState = .stopped
    private var topologyRegistrations: [TopologyRegistration] = []
    private var discoveredPeerAddresses: [String: Multiaddr] = [:]
    private let discoveredPeerAddressesQueue = DispatchQueue(label: "LibP2PService.discoveredPeerAddresses")
    @Published private var peerConnectionStates: [String: PeerConnectionState] = [:]
    
    public var savedPeerID:PeerID? {
        Self.loadStoredPeerID(for: Self.runtimeProfile)
    }
    
    private init() {
        self.peerID = Self.loadOrCreatePeerID(for: Self.runtimeProfile)
        self.chatDisplayName = "ios-\(self.peerID.b58String.prefix(8))"
        self.app = Self.makeApplication(peerID: self.peerID)
        self.lna = LocalNetworkAuthorization()
        self.app.logger.notice("Resolved runtime profile: \(Self.runtimeProfile.rawValue) on port \(Self.listenPort)")
    }

    private static var runtimeProfile: RuntimeProfile {
        #if targetEnvironment(macCatalyst)
        return .macCatalyst
        #elseif os(macOS)
        return .macOS
        #elseif os(iOS)
        if ProcessInfo.processInfo.isiOSAppOnMac {
            return .madeForiPad
        }
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return .iPad
        case .mac:
            return .madeForiPad
        default:
            return .iPhone
        }
        #else
        return .iPhone
        #endif
    }

    private static var listenPort: Int {
        if let value = ProcessInfo.processInfo.environment["P2P_LISTEN_PORT"],
           let port = Int(value),
           port > 0 {
            return port
        }
        switch Self.runtimeProfile {
        case .macOS:
            return 10000
        case .iPhone:
            return 10001
        case .iPad:
            return 10002
        case .macCatalyst:
            return 10003
        case .madeForiPad:
            return 10004
        }
    }

    private static var peerIDStorageKey: String {
        "MyPeerID.\(Self.runtimeProfile.rawValue)"
    }

    private static func loadStoredPeerID(for profile: RuntimeProfile) -> PeerID? {
        let key = "MyPeerID.\(profile.rawValue)"
        guard let data = UserDefaults.standard.data(forKey: key) else { return nil }
        return try? PeerID(marshaledPeerID: data)
    }

    private static func loadOrCreatePeerID(for profile: RuntimeProfile) -> PeerID {
        let key = "MyPeerID.\(profile.rawValue)"
        if let saved = Self.loadStoredPeerID(for: profile) {
            return saved
        }

        let seed = Data(SHA256.hash(data: Data("gnostr-chat.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        let peerID = try! PeerID(marshaledPrivateKey: privateKey.marshal())
        let marshaled = try! peerID.marshal(includingPrivateKey: true)
        UserDefaults.standard.set(Data(marshaled), forKey: key)
        return peerID
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        // Keep connections open long enough for pubsub chatter and simulator pauses.
        app.connectionManager.setIdleTimeout(.seconds(300))
        app.security.use(.noise)
        app.muxers.use(.mplex)
        app.relay.use(.relay)
        app.autonat.use(.autonat)
        app.dcutr.use(.dcutr)
        app.pubsub.use(.gossipsub)
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.servers.use(.tcp(host: "0.0.0.0", port: Self.listenPort))
        try! routes(app)
        return app
    }

    public func updateChatDisplayName(_ displayName: String) {
        let trimmed = displayName.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        self.chatDisplayName = trimmed
    }

    public func connectionState(for peerID: PeerID) -> PeerConnectionState {
        self.peerConnectionStates[peerID.b58String] ?? .disconnected
    }

    public func markPeerConnected(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .connected
        }
    }

    public func markPeerDialing(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .dialing
        }
    }

    public func markPeerDisconnected(_ peerID: PeerID) {
        DispatchQueue.main.async {
            self.peerConnectionStates[peerID.b58String] = .disconnected
        }
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

    private func isDialableTCPAddress(_ address: Multiaddr) -> Bool {
        guard let tcp = address.tcpAddress else { return false }
        return tcp.ip4
    }

    private func dial(peerID: PeerID, address: Multiaddr) {
        self.app.logger.notice("Dialing peer \(peerID) at \(address)")
        do {
            try self.app.newStream(to: address, forProtocol: "/ipfs/id/1.0.0")
        } catch {
            self.app.logger.error("Failed to dial peer \(peerID): \(error)")
            self.markPeerDisconnected(peerID)
        }
    }

    private func redial(peerID: PeerID) {
        guard let address = self.discoveredAddress(for: peerID),
              self.isDialableTCPAddress(address) else {
            self.app.logger.warning("No stored address available for peer \(peerID.b58String); cannot redial")
            self.markPeerDisconnected(peerID)
            return
        }
        self.markPeerDialing(peerID)
        self.dial(peerID: peerID, address: address)
    }

    private func installRuntimeHandlersIfNeeded() {
        guard !self.runtimeHandlersInstalled, let delegate = self.delegate else { return }

        self.app.discovery.onPeerDiscovered(self.app) { peer in
            self.app.logger.notice("We discovered a peer: \(peer)")
            self.app.connections.getConnectionsToPeer(peer: peer.peer, on: nil).whenSuccess { conns in
                if conns.isEmpty {
                    guard let address = peer.addresses.first(where: { self.isDialableTCPAddress($0) }) else {
                        self.app.logger.warning("No dialable IPv4 TCP address found for peer \(peer.peer)")
                        return
                    }
                    self.recordDiscoveredAddress(address, for: peer.peer)
                    self.markPeerDialing(peer.peer)
                    self.dial(peerID: peer.peer, address: address)
                    delegate.on(nickname: peer.peer.shortDescription, from: peer.peer)
                } else {
                    self.markPeerConnected(peer.peer)
                }
            }
        }

        self.app.events.on(self, event: .disconnected({ _, peerID in
            guard let peerID = peerID else { return }
            self.app.logger.notice("Disconnected from peer \(peerID.b58String); scheduling redial")
            self.markPeerDialing(peerID)
            self.app.eventLoopGroup.any().scheduleTask(in: .seconds(1)) {
                self.redial(peerID: peerID)
            }
        }))

        self.pingTask?.cancel()
        self.pingTask = self.app.eventLoopGroup.any().scheduleRepeatedTask(initialDelay: .seconds(15), delay: .seconds(15), { _ in
            self.pingDiscoveredUsers()
        })

        self.runtimeHandlersInstalled = true
        self.app.logger.notice("Installed runtime handlers for delegate \(String(describing: delegate))")
    }

    private func joinChatTopic() {
        guard self.chatSubscribedTopic != self.chatTopic else { return }

        do {
            let subscription = try self.app.pubsub.gossipsub.subscribe(
                .init(
                    topic: self.chatTopic,
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
                    self.app.logger.notice("Chat peer for \(self.chatTopic): \(peer.b58String)")
                    self.delegate?.on(nickname: peer.shortDescription, from: peer)
                case .data(let message):
                    guard let fromPeer = try? PeerID(fromBytesID: Array(message.from)) else {
                        self.app.logger.warning("Received chat message with invalid sender peer ID")
                        return eventLoop.makeSucceededVoidFuture()
                    }
                    let sender = fromPeer.b58String
                    let decoded = Self.decodeRustChatMessage(from: message.data)
                    let text = decoded?.content.first ?? String(data: message.data, encoding: .utf8) ?? "Not UTF-8 data"
                    let nickname = decoded?.from ?? sender
                    self.delegate?.on(nickname: nickname, from: fromPeer)
                    self.delegate?.on(message: text, from: fromPeer)
                case .error(let error):
                    self.app.logger.error("Chat topic error: \(error)")
                }
                return eventLoop.makeSucceededVoidFuture()
            }

            self.chatSubscription = subscription
            self.chatSubscribedTopic = self.chatTopic
            self.app.logger.notice("Joined chat topic \(self.chatTopic)")
        } catch {
            self.app.logger.error("Failed to join chat topic \(self.chatTopic): \(error)")
        }
    }
    
    public func deletePeerID() {
        UserDefaults.standard.removeObject(forKey: Self.peerIDStorageKey)
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
    
    public func start() async throws {
        guard self.lifecycleState == .stopped else { return }
        self.lifecycleState = .starting
        guard await self.lna?.requestAuthorization() ?? true else {
            self.lifecycleState = .stopped
            throw CocoaError(.userCancelled)
        }
        if self.app.didShutdown {
            self.app = Self.makeApplication(peerID: self.peerID)
            self.lna = LocalNetworkAuthorization()
            self.runtimeHandlersInstalled = false
            self.reinstallTopologyRegistrations()
        }
        self.installRuntimeHandlersIfNeeded()
        self.joinChatTopic()
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
        self.chatSubscription?.unsubscribe()
        self.chatSubscription = nil
        self.chatSubscribedTopic = nil
        app.shutdown()
        self.runtimeHandlersInstalled = false
        self.lifecycleState = .stopped
    }
    
    public func send(message:String, to peer:PeerID) {
        guard self.app.isRunning else { print("LibP2P needs to be running in order to send messages!"); return }
        guard let data = Self.encodeRustChatMessage(from: self.chatDisplayName, text: message) else {
            self.app.logger.error("Failed to encode chat message")
            return
        }
        self.joinChatTopic()
        self.chatSubscription?.publish(data)
        self.app.logger.trace("Published chat message on \(self.chatTopic) for peer \(peer)")
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
    
    /// This recurring task acts as a Keep-Alive service for Peers that support the `/chat/1.0.0` protocol.
    /// It keeps direct chat connections from idling out while pubsub traffic is sparse.
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

    private static func encodeRustChatMessage(from sender: String, text: String) -> Data? {
        let message = RustChatMessage(from: sender, content: text)
        return try? JSONEncoder().encode(message)
    }

    private static func decodeRustChatMessage(from data: Data) -> RustChatMessage? {
        try? JSONDecoder().decode(RustChatMessage.self, from: data)
    }
}
