//
//  P2PService.swift
//  GnostrP2P-KitchenSink
//

import CryptoKit
import Foundation
import LibP2P
import LibP2PDCUtR
import LibP2PKadDHT
import LibP2PMDNS
import LibP2PNoise
import LibP2PYAMUX
import LibP2PPubSub
import SwiftUI

#if os(iOS)
    import UIKit
#endif

@MainActor
final class P2PService: ObservableObject {
    enum State: String {
        case stopped
        case starting
        case running
        case stopping
    }

    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }

    struct PeerSummary: Identifiable, Hashable {
        var id: String { peerID }
        let peerID: String
        let addresses: [String]
    }

    struct ChatEntry: Identifiable, Hashable {
        let id = UUID()
        let topic: String
        let kind: String
        let author: String
        let text: String
        let isLocal: Bool
        let timestamp: Date
    }

    struct RustChatMessage: Codable {
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

    @Published private(set) var listenAddresses: [String] = []
    @Published private(set) var discoveredPeers: [PeerSummary] = []
    @Published private(set) var chatMessages: [ChatEntry] = []
    @Published private(set) var activityLog: [String] = []
    @Published private(set) var lastError: String?
    @Published private(set) var state: State = .stopped
    @Published var chatDisplayName = ""
    @Published var chatTopic = "gnostr-dev"
    @Published var chatDraftMessage = ""
    @Published var draftMessage = "Hello from GnostrP2P Kitchen Sink"

    private var app: Application?
    private var runTask: Task<Void, Never>?
    private var heartbeatTask: Task<Void, Never>?
    private var chatSubscription: PubSub.SubscriptionHandler?
    private var chatSubscribedTopic: String?
    private var dialedPeerIDs = Set<String>()
    private var gossipPeerIDs = Set<String>()

    let peerID: PeerID

    init() {
        peerID = Self.makePeerID()
        chatDisplayName = "ios-\(peerID.b58String.prefix(8))"
    }

    var runtimeProfile: String {
        Self.runtimeProfile.rawValue
    }

    var listenPort: Int {
        Self.listenPort
    }

    var peerIDString: String {
        peerID.b58String
    }

    var isRunning: Bool {
        state == .running
    }

    func clearActivityLog() {
        activityLog.removeAll()
    }

    func clearChatMessages() {
        chatMessages.removeAll()
    }

    func joinChatTopic() {
        guard let app else {
            log("Chat topic join requires the node to be running")
            return
        }

        let topic = chatTopic.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !topic.isEmpty else {
            lastError = "Chat topic cannot be empty"
            log("Chat topic cannot be empty")
            return
        }

        chatSubscription?.unsubscribe()
        chatSubscription = nil

        do {
            let subscription = try app.pubsub.gossipsub.subscribe(
                .init(
                    topic: topic,
                    signaturePolicy: .strictSign,
                    validator: .acceptAll,
                    messageIDFunc: .concatFromAndSequenceFields
                )
            )
            let eventLoop = app.eventLoopGroup.next()
            subscription.on = { [weak self] event in
                switch event {
                case .newPeer(let peer):
                    Task { @MainActor in
                        guard let self else { return }
                        self.gossipPeerIDs.insert(peer.b58String)
                        self.log("Peer joined gossip topic \(topic): \(peer.b58String)")
                    }
                case .data(let message):
                    let author = message.from.asString(base: .base58btc)
                    let decoded = Self.decodeRustChatMessage(from: message.data)
                    let text = decoded?.content.first ?? String(data: message.data, encoding: .utf8) ?? "Not UTF-8 data"
                    let sender = decoded?.from ?? author
                    let kind = decoded?.kind ?? "Raw"
                    Task { @MainActor in
                        guard let self else { return }
                        if kind == "Heartbeat" {
                            self.log("Peer heartbeat on \(topic) from \(author): \(text)")
                        } else {
                            self.log("Chat message on \(topic) from \(author)")
                        }
                        self.chatMessages.insert(
                            ChatEntry(
                                topic: topic,
                                kind: kind,
                                author: sender,
                                text: text,
                                isLocal: sender == self.chatDisplayName || author == self.peerID.b58String,
                                timestamp: Date()
                            ),
                            at: 0
                        )
                        self.chatMessages = Array(self.chatMessages.prefix(200))
                    }
                case .error(let error):
                    Task { @MainActor in
                        self?.lastError = error.localizedDescription
                        self?.log("Chat topic error: \(error.localizedDescription)")
                    }
                }
                return eventLoop.makeSucceededVoidFuture()
            }

            chatSubscription = subscription
            chatSubscribedTopic = topic
            clearChatMessages()
            gossipPeerIDs.removeAll()
            log("Joined chat topic \(topic)")
            startHeartbeatLoop(topic: topic, app: app)
        } catch {
            lastError = error.localizedDescription
            log("Failed to join chat topic \(topic): \(error.localizedDescription)")
        }
    }

    func sendChatMessage() {
        guard app != nil else {
            log("Chat send requires the node to be running")
            return
        }

        let topic = chatTopic.trimmingCharacters(in: .whitespacesAndNewlines)
        let message = chatDraftMessage.trimmingCharacters(in: .whitespacesAndNewlines)

        guard !topic.isEmpty else {
            lastError = "Chat topic cannot be empty"
            log("Chat topic cannot be empty")
            return
        }

        guard !message.isEmpty else {
            return
        }

        if chatSubscribedTopic != topic || chatSubscription == nil {
            joinChatTopic()
        }

        guard chatSubscribedTopic == topic else { return }
        guard let data = Self.encodeRustChatMessage(from: chatDisplayName, text: message) else {
            lastError = "Failed to encode chat message"
            log("Failed to encode chat message for topic \(topic)")
            return
        }

        chatSubscription?.publish(data)
        chatDraftMessage = ""
        log("Sent chat message on \(topic)")
    }

    func start() {
        guard runTask == nil else { return }

        lastError = nil
        state = .starting
        log("Starting libp2p node")

        let app = Self.makeApplication(peerID: peerID)
        self.app = app
        joinChatTopic()

        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            let peerID = peer.peer.b58String
            let addresses = peer.addresses.map(\.description)
            DispatchQueue.main.async { [weak self] in
                self?.recordDiscoveredPeer(peerID: peerID, addresses: addresses)
                self?.dialDiscoveredPeer(peer)
            }
        }

        app.events.on(self, event: .disconnected({ [weak self] connection, peer in
            Task { @MainActor in
                self?.handleDisconnectedPeer(connection: connection, peer: peer)
            }
        }))

        app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) { [weak self, weak app] in
            guard let self, let app else { return }
            let addresses = app.listenAddresses.compactMap { address -> String? in
                guard let fullAddress = try? address.encapsulate(proto: .p2p, address: app.peerID.b58String) else {
                    return nil
                }
                return fullAddress.description
            }

            Task { @MainActor in
                self.listenAddresses = addresses
                if !addresses.isEmpty {
                    self.log("Listening on: \(addresses.joined(separator: ", "))")
                }
                if self.state == .starting {
                    self.state = .running
                    self.log("Node is running")
                }
            }
        }

        runTask = Task.detached(priority: .background) { [weak self, app] in
            do {
                try await app.execute()
            } catch {
                await MainActor.run { [weak self] in
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run { [weak self] in
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.listenAddresses = []
                self?.chatSubscription = nil
                self?.chatSubscribedTopic = nil
                self?.heartbeatTask?.cancel()
                self?.heartbeatTask = nil
                self?.dialedPeerIDs.removeAll()
                self?.gossipPeerIDs.removeAll()
                self?.log("Node stopped")
            }
        }
    }

    func stop() {
        guard let app else { return }

        state = .stopping
        log("Stopping libp2p node")
        chatSubscription?.unsubscribe()
        chatSubscription = nil
        chatSubscribedTopic = nil
        heartbeatTask?.cancel()
        heartbeatTask = nil
        self.app = nil
        self.runTask = nil

        Task.detached(priority: .background) { [weak self] in
            do {
                try await app.asyncShutdown()
            } catch {
                await MainActor.run { [weak self] in
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run { [weak self] in
                self?.listenAddresses = []
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.chatSubscription = nil
                self?.chatSubscribedTopic = nil
                self?.heartbeatTask = nil
                self?.dialedPeerIDs.removeAll()
                self?.gossipPeerIDs.removeAll()
                self?.log("Node stopped")
            }
        }
    }

    func restart() {
        stop()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            self.start()
        }
    }

    func sendLocalPing() {
        sendChatText(draftMessage, kind: "Heartbeat")
    }

    private func recordDiscoveredPeer(peerID: String, addresses: [String]) {
        let peer = PeerSummary(peerID: peerID, addresses: addresses)
        if !discoveredPeers.contains(peer) {
            discoveredPeers.append(peer)
            log("Discovered peer \(peerID)")
        }
    }

    private static func encodeRustChatMessage(from sender: String, text: String, kind: String = "Chat") -> Data? {
        let message = RustChatMessage(from: sender, content: text, kind: kind)
        return try? JSONEncoder().encode(message)
    }

    private static func decodeRustChatMessage(from data: Data) -> RustChatMessage? {
        try? JSONDecoder().decode(RustChatMessage.self, from: data)
    }

    private func handleDisconnectedPeer(connection: Connection, peer: PeerID?) {
        guard let peer else {
            log("Peer disconnected: \(connection.remoteAddr?.description ?? "unknown")")
            return
        }

        let peerID = peer.b58String
        dialedPeerIDs.remove(peerID)
        let wasTopicPeer = gossipPeerIDs.remove(peerID) != nil
        log(wasTopicPeer ? "Peer left gossip topic \(chatTopic): \(peerID)" : "Peer disconnected: \(peerID)")
    }

    private func startHeartbeatLoop(topic: String, app: Application) {
        heartbeatTask?.cancel()
        heartbeatTask = Task.detached(priority: .background) { [weak self] in
            var counter = 0
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 1_000_000_000)
                guard !Task.isCancelled else { break }
                let hasPeers = await MainActor.run { [weak self] in
                    guard let self else { return false }
                    return !self.gossipPeerIDs.isEmpty
                }
                guard hasPeers else { continue }
                counter += 1
                let message = RustChatMessage(
                    from: app.peerID.b58String,
                    content: "heartbeat #\(counter) from \(app.peerID.b58String)",
                    kind: "Heartbeat"
                )
                guard let data = try? JSONEncoder().encode(message) else { continue }
                _ = app.pubsub.gossipsub.publish(
                    topic: topic,
                    bytes: data.byteArray,
                    on: app.eventLoopGroup.next()
                )
                await MainActor.run { [weak self] in
                    self?.log("Published heartbeat #\(counter) to \(topic)")
                }
            }
        }
    }

    private func sendChatText(_ text: String, kind: String = "Chat") {
        guard let app else {
            log("Chat send requires the node to be running")
            return
        }

        let topic = chatTopic.trimmingCharacters(in: .whitespacesAndNewlines)
        let message = text.trimmingCharacters(in: .whitespacesAndNewlines)

        guard !topic.isEmpty else {
            lastError = "Chat topic cannot be empty"
            log("Chat topic cannot be empty")
            return
        }

        guard !message.isEmpty else {
            return
        }

        if chatSubscribedTopic != topic || chatSubscription == nil {
            joinChatTopic()
        }

        guard chatSubscribedTopic == topic else { return }
        guard let data = Self.encodeRustChatMessage(from: chatDisplayName, text: message, kind: kind) else {
            lastError = "Failed to encode chat message"
            log("Failed to encode chat message for topic \(topic)")
            return
        }

        chatSubscription?.publish(data)
        if kind == "Chat" {
            chatDraftMessage = ""
        }
        log("Sent \(kind.lowercased()) on \(topic)")
    }

    private func dialDiscoveredPeer(_ peer: PeerInfo) {
        guard let app else { return }
        let peerID = peer.peer.b58String
        guard peerID != self.peerID.b58String else { return }
        guard !dialedPeerIDs.contains(peerID) else { return }
        dialedPeerIDs.insert(peerID)

        do {
            try app.newStream(to: peer, forProtocol: "/ipfs/id/1.0.0")
            log("Dialing discovered peer \(peerID)")
        } catch {
            lastError = error.localizedDescription
            log("Failed to dial discovered peer \(peerID): \(error.localizedDescription)")
        }
    }

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        activityLog.insert("[\(formatter.string(from: Date()))] \(message)", at: 0)
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.gossipsub(emitSelf: false))
        app.dcutr.use(.dcutr)
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.listen(.tcp(host: "0.0.0.0", port: Self.listenPort))
        return app
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

        switch runtimeProfile {
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

    private static func makePeerID() -> PeerID {
        let seed = Data(SHA256.hash(data: Data(instanceSeed().utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private static func instanceSeed() -> String {
        let defaults = UserDefaults.standard
        let key = "GnostrP2PKitchenSink.peerSeed"

        if let existing = defaults.string(forKey: key), !existing.isEmpty {
            return existing
        }

        let seed = UUID().uuidString
        defaults.set(seed, forKey: key)
        return seed
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}
