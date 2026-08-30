//
//  P2PService.swift
//  libp2p-app-template
//

import CryptoKit
import Foundation
import LibP2P
import Multihash
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

// MARK: - History Sync Types

private struct HistoryRequest: Codable {
    let topic: String
    let limit: Int
}

private struct HistoryResponse: Codable {
    let messages: [HistoryMessage]
}

private struct HistoryMessage: Codable, Hashable {
    let id: String
    let topic: String
    let kind: String
    let author: String
    let text: String
    let timestamp: TimeInterval
}

// MARK: - History Persistence

private actor HistoryPersistence {
    private let directory: URL

    init() {
        let docs = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first
        let dir = docs?.appendingPathComponent("ChatHistory", isDirectory: true)
        if let dir, !FileManager.default.fileExists(atPath: dir.path) {
            try? FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
        }
        self.directory = dir ?? URL(fileURLWithPath: "/tmp/chat-history")
    }

    func save(topic: String, messages: [HistoryMessage]) {
        let url = fileURL(for: topic)
        do {
            let data = try JSONEncoder().encode(messages)
            try data.write(to: url, options: .atomic)
        } catch {
            // Silently fail to avoid sluggish UI
        }
    }

    func load(topic: String) -> [HistoryMessage] {
        let url = fileURL(for: topic)
        guard FileManager.default.fileExists(atPath: url.path) else { return [] }
        do {
            let data = try Data(contentsOf: url)
            return try JSONDecoder().decode([HistoryMessage].self, from: data)
        } catch {
            return []
        }
    }

    private func fileURL(for topic: String) -> URL {
        let safe = SHA256.hash(data: Data(topic.utf8)).compactMap { String(format: "%02x", $0) }.joined()
        return directory.appendingPathComponent("\(safe).json")
    }
}

// MARK: - History Store

private actor HistoryStore {
    private var messagesByTopic: [String: [HistoryMessage]] = [:]
    private var allIDs: Set<String> = []
    private let maxMessagesPerTopic = 200

    func add(topic: String, kind: String, author: String, text: String) -> HistoryMessage {
        let timestamp = Date().timeIntervalSince1970
        let id = HistoryStore.stableID(author: author, text: text, timestamp: timestamp)
        let msg = HistoryMessage(
            id: id,
            topic: topic,
            kind: kind,
            author: author,
            text: text,
            timestamp: timestamp
        )

        if allIDs.insert(id).inserted {
            messagesByTopic[topic, default: []].append(msg)
            trim(topic: topic)
        }
        return msg
    }

    func history(for topic: String, limit: Int) -> [HistoryMessage] {
        let msgs = messagesByTopic[topic, default: []]
        return Array(msgs.suffix(limit))
    }

    func merge(_ messages: [HistoryMessage]) -> [HistoryMessage] {
        var newMessages: [HistoryMessage] = []
        for msg in messages {
            if allIDs.insert(msg.id).inserted {
                messagesByTopic[msg.topic, default: []].append(msg)
                newMessages.append(msg)
            }
        }
        for topic in Set(newMessages.map(\.topic)) {
            trim(topic: topic)
        }
        return newMessages
    }

    func load(topic: String, messages: [HistoryMessage]) {
        for msg in messages {
            guard allIDs.insert(msg.id).inserted else { continue }
            messagesByTopic[msg.topic, default: []].append(msg)
        }
        trim(topic: topic)
    }

    func allMessages(for topic: String) -> [HistoryMessage] {
        messagesByTopic[topic, default: []]
    }

    private func trim(topic: String) {
        guard messagesByTopic[topic]!.count > maxMessagesPerTopic else { return }
        let toRemove = messagesByTopic[topic]!.count - maxMessagesPerTopic
        for msg in messagesByTopic[topic]!.prefix(toRemove) {
            allIDs.remove(msg.id)
        }
        messagesByTopic[topic]!.removeFirst(toRemove)
    }

    static func stableID(author: String, text: String, timestamp: TimeInterval) -> String {
        let input = "\(author)|\(text)|\(String(format: "%.3f", timestamp))"
        return SHA256.hash(data: Data(input.utf8)).compactMap { String(format: "%02x", $0) }.joined()
    }
}

// MARK: - P2P Service

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
        let id: String
        let topic: String
        let kind: String
        let author: String
        let text: String
        let isLocal: Bool
        let timestamp: Date

        fileprivate init(from msg: HistoryMessage, isLocal: Bool) {
            self.id = msg.id
            self.topic = msg.topic
            self.kind = msg.kind
            self.author = msg.author
            self.text = msg.text
            self.isLocal = isLocal
            self.timestamp = Date(timeIntervalSince1970: msg.timestamp)
        }
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
    @Published var chatTopic = "libp2p-dev"
    @Published var chatDraftMessage = ""
    @Published var draftMessage = "Hello from LibP2P App Template"

    private var app: Application?
    private var runTask: Task<Void, Never>?
    private var chatSubscription: PubSub.SubscriptionHandler?
    private var chatSubscribedTopic: String?
    private var dialedPeerIDs = Set<String>()
    private let historyStore = HistoryStore()
    private let historyPersistence = HistoryPersistence()

    let peerID: PeerID

    init() {
        peerID = Self.makePeerID(for: Self.runtimeProfile)
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

        // If already subscribed to this topic, just reload history (no-op otherwise)
        if chatSubscribedTopic == topic, chatSubscription != nil {
            log("Already joined topic \(topic)")
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
                        self.log("Chat peer for \(topic): \(peer.b58String)")
                        await self.requestHistory(from: peer, topic: topic)
                    }
                case .data(let message):
                    let author = message.from.asString(base: .base58btc)
                    let decoded = Self.decodeRustChatMessage(from: message.data)
                    let text = decoded?.content.first ?? String(data: message.data, encoding: .utf8) ?? "Not UTF-8 data"
                    let sender = decoded?.from ?? author
                    let kind = decoded?.kind ?? "Raw"
                    Task { @MainActor in
                        guard let self else { return }
                        let msg = await self.historyStore.add(topic: topic, kind: kind, author: sender, text: text)
                        self.chatMessages.insert(
                            ChatEntry(from: msg, isLocal: sender == self.chatDisplayName || author == self.peerID.b58String),
                            at: 0
                        )
                        self.chatMessages = Array(self.chatMessages.prefix(200))
                        self.log("Chat message on \(topic) from \(author)")
                        self.persistHistory(for: topic)
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

            // Load persisted history asynchronously so the UI isn't blocked
            Task { @MainActor in
                let persisted = await self.historyPersistence.load(topic: topic)
                if !persisted.isEmpty {
                    await self.historyStore.load(topic: topic, messages: persisted)
                    let loaded = await self.historyStore.history(for: topic, limit: 200)
                    self.chatMessages = loaded.map { msg in
                        ChatEntry(from: msg, isLocal: msg.author == self.chatDisplayName || msg.author == self.peerID.b58String)
                    }.reversed()
                    self.log("Loaded \(loaded.count) persisted messages for \(topic)")
                } else {
                    self.chatMessages.removeAll()
                    self.log("Joined chat topic \(topic)")
                }

                // Request history from all already-discovered peers
                for peerInfo in self.discoveredPeers {
                    if let mh = try? Multihash(b58String: peerInfo.peerID),
                       let peerID = try? PeerID(fromBytesID: mh.value) {
                        await self.requestHistory(from: peerID, topic: topic)
                    }
                }
            }
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

        // Add to local history immediately so peers can request it right away
        Task { @MainActor in
            let msg = await self.historyStore.add(topic: topic, kind: "Chat", author: self.chatDisplayName, text: message)
            self.chatMessages.insert(ChatEntry(from: msg, isLocal: true), at: 0)
            self.chatMessages = Array(self.chatMessages.prefix(200))
            self.persistHistory(for: topic)
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

        let app = Self.makeApplication(peerID: peerID, historyStore: historyStore)
        self.app = app

        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            let peerID = peer.peer.b58String
            let addresses = peer.addresses.map(\.description)
            DispatchQueue.main.async { [weak self] in
                self?.recordDiscoveredPeer(peerID: peerID, addresses: addresses)
                self?.dialDiscoveredPeer(peer)
            }
        }

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
                    self.joinChatTopic()
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
                self?.dialedPeerIDs.removeAll()
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
                self?.dialedPeerIDs.removeAll()
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
        log("Ping: \(draftMessage)")
    }

    private func persistHistory(for topic: String) {
        Task.detached(priority: .background) { [historyStore, historyPersistence] in
            let messages = await historyStore.allMessages(for: topic)
            await historyPersistence.save(topic: topic, messages: messages)
        }
    }

    private func requestHistory(from peer: PeerID, topic: String) async {
        guard let app else { return }
        guard peer.b58String != self.peerID.b58String else { return }

        let request = HistoryRequest(topic: topic, limit: 50)
        guard let data = try? JSONEncoder().encode(request) else { return }

        do {
            let responseData = try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Data, Error>) in
                app.newRequest(
                    to: peer,
                    forProtocol: "/libp2p-app-template/chat-history/1.0.0",
                    withRequest: data,
                    withTimeout: .seconds(5)
                ).whenComplete { result in
                    continuation.resume(with: result)
                }
            }

            let response = try JSONDecoder().decode(HistoryResponse.self, from: responseData)
            let newMessages = await historyStore.merge(response.messages)

            for msg in newMessages.sorted(by: { $0.timestamp < $1.timestamp }) {
                let isLocal = msg.author == self.chatDisplayName || msg.author == self.peerID.b58String
                chatMessages.insert(ChatEntry(from: msg, isLocal: isLocal), at: 0)
            }
            if !newMessages.isEmpty {
                chatMessages = Array(chatMessages.prefix(200))
                log("Merged \(newMessages.count) historical messages from \(peer.b58String)")
                persistHistory(for: topic)
            }
        } catch {
            log("History request failed for \(peer.b58String): \(error.localizedDescription)")
        }
    }

    private func recordDiscoveredPeer(peerID: String, addresses: [String]) {
        let peer = PeerSummary(peerID: peerID, addresses: addresses)
        if !discoveredPeers.contains(peer) {
            discoveredPeers.append(peer)
            log("Discovered peer \(peerID)")
        }
    }

    private static func encodeRustChatMessage(from sender: String, text: String) -> Data? {
        let message = RustChatMessage(from: sender, content: text)
        return try? JSONEncoder().encode(message)
    }

    private static func decodeRustChatMessage(from data: Data) -> RustChatMessage? {
        try? JSONDecoder().decode(RustChatMessage.self, from: data)
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

    private static func makeApplication(peerID: PeerID, historyStore: HistoryStore) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.gossipsub(emitSelf: true))
        app.dcutr.use(.dcutr)
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.listen(.tcp(host: "0.0.0.0", port: Self.listenPort))

        // Register chat history sync protocol handler
        app.on("libp2p-app-template", "chat-history", "1.0.0") { req -> EventLoopFuture<Data> in
            let promise = req.eventLoop.makePromise(of: Data.self)
            Task {
                let payloadData = req.payload.getBytes(at: 0, length: req.payload.readableBytes).map { Data($0) } ?? Data()
                let request = try? JSONDecoder().decode(HistoryRequest.self, from: payloadData)
                let history = await historyStore.history(
                    for: request?.topic ?? "",
                    limit: request?.limit ?? 50
                )
                let response = try? JSONEncoder().encode(HistoryResponse(messages: history))
                promise.succeed(response ?? Data())
            }
            return promise.futureResult
        }

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

    private static func makePeerID(for profile: RuntimeProfile) -> PeerID {
        let seed = Data(SHA256.hash(data: Data("libp2p-app-template.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}
