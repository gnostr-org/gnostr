//
//  ChatService.swift
//  GnostrP2P-KitchenSink
//

import Foundation
import LibP2P
import LibP2PPubSub
import SwiftUI

@MainActor
final class ChatService: ObservableObject {
    @Published var selectedTopicId: String?
    @Published var composerText: String = ""
    @Published var displayName: String = ""
    @Published private(set) var lastError: String?
    @Published private(set) var logs: [String] = []

    let node: P2PNode
    let store: ChatStore
    let tracker: ReceiptTracker

    private var subscriptions: [String: PubSub.SubscriptionHandler] = [:]
    private var pendingTopicJoins: [String] = []
    private var heartbeatTask: Task<Void, Never>?

    private let defaultTopic = "universal-connectivity"

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        let line = "[\(formatter.string(from: Date()))] ChatService: \(message)"
        print(line)
        logs.insert(line, at: 0)
        if logs.count > 500 {
            logs = Array(logs.prefix(500))
        }
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss.SSS"
        return formatter
    }()

    init(node: P2PNode, store: ChatStore, tracker: ReceiptTracker) {
        self.node = node
        self.store = store
        self.tracker = tracker
        self.displayName = node.displayName

        node.delegate = self
        tracker.delegate = self

        // Ensure the default topic exists in the store; it will be joined once the node starts.
        if store.topic(named: defaultTopic) == nil {
            store.addTopic(named: defaultTopic)
        }
        if selectedTopicId == nil {
            selectedTopicId = store.topic(named: defaultTopic)?.id
        }
    }

    // MARK: - Lifecycle

    func start() {
        log("start()")
        node.start()
    }

    func stop() {
        log("stop()")
        unsubscribeAll()
        heartbeatTask?.cancel()
        heartbeatTask = nil
        node.stop()
    }

    func clearLogs() {
        logs.removeAll()
    }

    // MARK: - Topic management

    @discardableResult
    func joinTopic(named name: String) -> ChatTopic? {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            lastError = "Topic name cannot be empty"
            log("joinTopic rejected: empty name")
            return nil
        }

        log("joinTopic named=\(trimmed)")
        let topic = store.addTopic(named: trimmed)
        subscribe(to: topic)
        selectedTopicId = topic.id
        return topic
    }

    func leaveTopic(id: String) {
        log("leaveTopic id=\(id)")
        guard let topic = store.topic(id: id) else { return }
        announceLeave(on: topic)
        unsubscribe(from: id)
        store.setTopicJoined(id: id, joined: false)
        subscriptions.removeValue(forKey: id)
        if selectedTopicId == id {
            selectedTopicId = store.topics.first?.id
        }
    }

    func selectTopic(id: String) {
        log("selectTopic id=\(id)")
        selectedTopicId = id
    }

    func deleteTopic(id: String) {
        log("deleteTopic id=\(id)")
        leaveTopic(id: id)
        store.removeTopic(id: id)
    }

    private func unsubscribeAll() {
        let ids = Array(subscriptions.keys)
        log("unsubscribeAll count=\(ids.count)")
        for id in ids {
            subscriptions[id]?.unsubscribe()
            subscriptions.removeValue(forKey: id)
        }
        for topic in store.topics where topic.isJoined {
            store.setTopicJoined(id: topic.id, joined: false)
        }
    }

    // MARK: - Messaging

    func sendMessage(_ text: String) {
        guard let topicId = selectedTopicId, let topic = store.topic(id: topicId) else {
            lastError = "No topic selected"
            return
        }
        guard node.isRunning else {
            lastError = "Node is not running"
            return
        }

        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        let envelope = ChatEnvelope(
            kind: .chat,
            senderId: node.peerIDString,
            senderName: displayName.isEmpty ? node.displayName : displayName,
            topic: topic.name,
            text: trimmed
        )

        log("sendMessage topic=\(topic.name) envelopeId=\(envelope.id) senderId=\(envelope.senderId) textLength=\(trimmed.count)")
        publish(envelope: envelope, topic: topic)

        let message = ChatMessage.fromEnvelope(envelope, topicId: topic.id, isLocal: true, status: .sending)
        store.addMessage(message)
        tracker.track(envelope, topic: topic.name)

        composerText = ""
    }

    func sendRawMessage(_ text: String) {
        guard let topicId = selectedTopicId, let topic = store.topic(id: topicId) else {
            lastError = "No topic selected"
            return
        }
        guard node.isRunning else {
            lastError = "Node is not running"
            return
        }

        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        guard let data = trimmed.data(using: .utf8) else {
            lastError = "Failed to encode raw message"
            log("sendRawMessage encode failure topic=\(topic.name)")
            return
        }

        if let subscription = subscriptions[topic.id] {
            log("sendRawMessage topic=\(topic.name) bytes=\(data.count) textLength=\(trimmed.count)")
            subscription.publish(data)

            let envelope = ChatEnvelope(
                kind: .raw,
                senderId: node.peerIDString,
                senderName: displayName.isEmpty ? node.displayName : displayName,
                topic: topic.name,
                text: trimmed
            )
            let message = ChatMessage.fromEnvelope(envelope, topicId: topic.id, isLocal: true, status: .sent)
            store.addMessage(message)
        } else {
            lastError = "Not subscribed to \(topic.name)"
            log("sendRawMessage failure (not subscribed) topic=\(topic.name)")
        }

        composerText = ""
    }

    func sendHeartbeat() {
        guard let topicId = selectedTopicId, let topic = store.topic(id: topicId) else {
            lastError = "No topic selected"
            return
        }
        guard node.isRunning else {
            lastError = "Node is not running"
            return
        }

        let addressList = node.listenAddresses.isEmpty ? "no-known-addresses" : node.listenAddresses.joined(separator: ", ")
        let text = "heartbeat from \(node.peerIDString) on \(addressList)"
        let envelope = ChatEnvelope(
            kind: .heartbeat,
            senderId: node.peerIDString,
            senderName: displayName.isEmpty ? node.displayName : displayName,
            topic: topic.name,
            text: text
        )

        log("sendHeartbeat topic=\(topic.name) envelopeId=\(envelope.id) senderId=\(envelope.senderId)")
        publish(envelope: envelope, topic: topic)
    }

    func retryMessage(envelopeId: String) {
        log("retryMessage envelopeId=\(envelopeId)")
        guard let message = store.message(envelopeId: envelopeId, isLocal: true),
              let topic = store.topic(id: message.topicId) else { return }
        let envelope = ChatEnvelope(
            id: message.envelopeId,
            kind: message.kind,
            senderId: message.senderId,
            senderName: message.senderName,
            topic: topic.name,
            text: message.text,
            timestamp: message.timestamp
        )
        store.updateMessageStatus(envelopeId: envelopeId, isLocal: true, status: .sending)
        tracker.track(envelope, topic: topic.name)
        publish(envelope: envelope, topic: topic)
    }

    // MARK: - Subscription handling

    private func subscribe(to topic: ChatTopic) {
        guard let app = node.application else {
            if !pendingTopicJoins.contains(topic.name) {
                pendingTopicJoins.append(topic.name)
                log("subscribe deferred (node not ready) topic=\(topic.name)")
            }
            return
        }
        guard subscriptions[topic.id] == nil else {
            log("subscribe skipped (already subscribed) topic=\(topic.name) id=\(topic.id)")
            return
        }

        log("subscribe topic=\(topic.name) id=\(topic.id)")
        do {
            let subscription = try app.pubsub.gossipsub.subscribe(
                .init(
                    topic: topic.name,
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
                        let peerID = peer.b58String
                        guard peerID != self.node.peerIDString else { return }
                        self.log("subscription.newPeer topic=\(topic.name) peerID=\(peerID)")
                        self.store.recordPeer(id: peerID, addresses: [], displayName: nil, isConnected: true)
                    }
                case .data(let message):
                    Task { @MainActor in
                        guard let self else { return }
                        let author = message.from.asString(base: .base58btc)
                        self.log("subscription.data topic=\(topic.name) from=\(author ?? "nil") dataBytes=\(message.data.count)")
                        self.handleIncomingData(message.data, from: author, topicName: topic.name)
                    }
                case .error(let error):
                    Task { @MainActor in
                        self?.log("subscription.error topic=\(topic.name) error=\(error.localizedDescription)")
                        self?.lastError = error.localizedDescription
                    }
                }
                return eventLoop.makeSucceededVoidFuture()
            }

            subscriptions[topic.id] = subscription
            store.setTopicJoined(id: topic.id, joined: true)
            log("subscribe success topic=\(topic.name) id=\(topic.id)")
            announceJoin(on: topic)
            startHeartbeatLoopIfNeeded()
        } catch {
            log("subscribe failure topic=\(topic.name) error=\(error.localizedDescription)")
            lastError = error.localizedDescription
        }
    }

    private func unsubscribe(from topicId: String) {
        log("unsubscribe topicId=\(topicId)")
        subscriptions[topicId]?.unsubscribe()
    }

    private func flushPendingJoins() {
        let names = pendingTopicJoins
        log("flushPendingJoins count=\(names.count) topics=\(names)")
        pendingTopicJoins.removeAll()
        for name in names {
            if let topic = store.topic(named: name) {
                subscribe(to: topic)
            }
        }
    }

    // MARK: - Publishing

    private func publish(envelope: ChatEnvelope, topic: ChatTopic) {
        guard let data = try? JSONEncoder().encode(envelope) else {
            lastError = "Failed to encode message"
            log("publish encode failure envelopeId=\(envelope.id) topic=\(topic.name)")
            store.updateMessageStatus(envelopeId: envelope.id, isLocal: true, status: .failed)
            return
        }

        if let subscription = subscriptions[topic.id] {
            log("publish envelopeId=\(envelope.id) kind=\(envelope.kind) topic=\(topic.name) senderId=\(envelope.senderId) bytes=\(data.count)")
            subscription.publish(data)
            tracker.markSent(messageId: envelope.id)
            store.updateMessageStatus(envelopeId: envelope.id, isLocal: true, status: .sent)
        } else {
            log("publish failure (not subscribed) envelopeId=\(envelope.id) topic=\(topic.name)")
            lastError = "Not subscribed to \(topic.name)"
            store.updateMessageStatus(envelopeId: envelope.id, isLocal: true, status: .failed)
        }
    }

    private func announceJoin(on topic: ChatTopic) {
        let addressList = node.listenAddresses.isEmpty ? "no-known-addresses" : node.listenAddresses.joined(separator: ", ")
        let envelope = ChatEnvelope(
            kind: .join,
            senderId: node.peerIDString,
            senderName: displayName.isEmpty ? node.displayName : displayName,
            topic: topic.name,
            text: "joined \(topic.name) as \(node.peerIDString) on \(addressList)"
        )
        log("announceJoin topic=\(topic.name) envelopeId=\(envelope.id) senderId=\(envelope.senderId) addresses=\(addressList)")
        publish(envelope: envelope, topic: topic)
    }

    private func announceLeave(on topic: ChatTopic) {
        let addressList = node.listenAddresses.isEmpty ? "no-known-addresses" : node.listenAddresses.joined(separator: ", ")
        let envelope = ChatEnvelope(
            kind: .leave,
            senderId: node.peerIDString,
            senderName: displayName.isEmpty ? node.displayName : displayName,
            topic: topic.name,
            text: "left \(topic.name) as \(node.peerIDString) on \(addressList)"
        )
        log("announceLeave topic=\(topic.name) envelopeId=\(envelope.id) senderId=\(envelope.senderId) addresses=\(addressList)")
        publish(envelope: envelope, topic: topic)
    }

    private func sendReceipt(for envelope: ChatEnvelope, topicName: String) {
        guard let topic = store.topic(named: topicName) else { return }
        let addressList = node.listenAddresses.isEmpty ? "no-known-addresses" : node.listenAddresses.joined(separator: ", ")
        let receipt = ChatEnvelope(
            kind: .receipt,
            senderId: node.peerIDString,
            senderName: displayName.isEmpty ? node.displayName : displayName,
            topic: topicName,
            text: "received \(envelope.kind.rawValue) from \(envelope.senderId) as \(node.peerIDString) on \(addressList)",
            originalMessageId: envelope.id
        )
        log("sendReceipt originalEnvelopeId=\(envelope.id) topic=\(topicName) senderId=\(receipt.senderId) addresses=\(addressList)")
        publish(envelope: receipt, topic: topic)
    }

    // MARK: - Incoming data

    private func handleIncomingData(_ data: Data, from: String?, topicName: String) {
        log("handleIncomingData topic=\(topicName) from=\(from ?? "nil") bytes=\(data.count)")
        guard let topic = store.topic(named: topicName) else {
            log("handleIncomingData dropped: unknown topic=\(topicName)")
            return
        }

        let decoder = JSONDecoder()
        if let envelope = try? decoder.decode(ChatEnvelope.self, from: data) {
            log("handleIncomingData decoded envelopeId=\(envelope.id) kind=\(envelope.kind) senderId=\(envelope.senderId) senderName=\(envelope.senderName) topic=\(envelope.topic) textLength=\(envelope.text.count)")

            // Ignore our own reflected messages (gossipsub emitSelf is off, but be defensive).
            guard envelope.senderId != node.peerIDString else {
                log("handleIncomingData ignored own envelope envelopeId=\(envelope.id)")
                return
            }

            ingest(envelope, topic: topic, from: from)
            return
        }

        // Treat undecodable data as a raw message.
        let text = String(data: data, encoding: .utf8) ?? "<binary data>"
        log("handleIncomingData decode fallback topic=\(topicName) from=\(from ?? "nil") textLength=\(text.count)")
        let fallback = ChatEnvelope(
            kind: .raw,
            senderId: from ?? "unknown",
            senderName: from ?? "unknown",
            topic: topicName,
            text: text
        )
        ingest(fallback, topic: topic, from: from)
    }

    private func ingest(_ envelope: ChatEnvelope, topic: ChatTopic, from: String?) {
        guard !tracker.isDuplicateInbound(envelope.id) else {
            log("ingest duplicate ignored envelopeId=\(envelope.id)")
            return
        }
        tracker.markInboundSeen(envelope.id)

        let peerAddresses = store.peer(id: from ?? envelope.senderId)?.addresses ?? []
        let addressSummary = peerAddresses.isEmpty ? "no-known-addresses" : peerAddresses.joined(separator: ", ")

        log("ingest envelopeId=\(envelope.id) kind=\(envelope.kind) topic=\(topic.name) from=\(from ?? "nil") senderId=\(envelope.senderId) addresses=\(addressSummary)")

        if envelope.kind == .receipt, let originalId = envelope.originalMessageId {
            log("ingest receipt originalEnvelopeId=\(originalId) from=\(from ?? "nil") addresses=\(addressSummary)")
            tracker.recordReceipt(originalMessageId: originalId)
            store.updateMessageStatus(envelopeId: originalId, isLocal: true, status: .delivered)
            return
        }

        if envelope.kind == .heartbeat {
            log("ingest heartbeat from=\(from ?? "nil") senderId=\(envelope.senderId) addresses=\(addressSummary) text=\(envelope.text)")
        }

        let message = ChatMessage.fromEnvelope(envelope, topicId: topic.id, isLocal: false, status: .sent)
        log("ingest chatMessage id=\(message.id) envelopeId=\(envelope.id) kind=\(envelope.kind) topic=\(topic.name) senderId=\(envelope.senderId)")
        store.addMessage(message)

        if envelope.kind == .chat || envelope.kind == .raw {
            sendReceipt(for: envelope, topicName: topic.name)
        }

        if let peerID = from {
            store.recordPeer(id: peerID, addresses: [], displayName: envelope.senderName, isConnected: true)
        }
    }

    // MARK: - Heartbeat

    private func startHeartbeatLoopIfNeeded() {
        guard heartbeatTask == nil else { return }
        heartbeatTask = Task.detached(priority: .background) { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 10_000_000_000)
                guard !Task.isCancelled else { return }
                await MainActor.run { [weak self] in
                    guard let self, self.node.isRunning else { return }
                    let hasPeers = !self.store.peers.filter(\.isConnected).isEmpty || !self.node.discoveredPeers.isEmpty
                    guard hasPeers else { return }
                    self.sendHeartbeat()
                }
            }
        }
        log("startHeartbeatLoop")
    }

    private func stopHeartbeatLoop() {
        heartbeatTask?.cancel()
        heartbeatTask = nil
        log("stopHeartbeatLoop")
    }
}

// MARK: - P2PNodeDelegate

extension ChatService: P2PNodeDelegate {
    func p2pNode(_ node: P2PNode, didDiscoverPeer peer: PeerInfo) {
        let peerID = peer.peer.b58String
        let addresses = peer.addresses.map(\.description)
        guard peerID != node.peerIDString else { return }
        log("p2pNode.didDiscoverPeer peerID=\(peerID) addresses=\(addresses)")
        store.recordPeer(
            id: peerID,
            addresses: addresses,
            displayName: nil,
            isConnected: true
        )
    }

    func p2pNode(_ node: P2PNode, didDisconnectPeer peer: PeerID, connection: Connection) {
        let peerID = peer.b58String
        log("p2pNode.didDisconnectPeer peerID=\(peerID) remoteAddr=\(connection.remoteAddr?.description ?? "nil")")
        store.updatePeerConnection(id: peerID, isConnected: false)
    }

    func p2pNode(_ node: P2PNode, didUpdateListenAddresses addresses: [String]) {
        log("p2pNode.didUpdateListenAddresses count=\(addresses.count) addresses=\(addresses)")
    }

    func p2pNodeDidStart(_ node: P2PNode) {
        log("p2pNodeDidStart peerID=\(node.peerIDString)")
        flushPendingJoins()

        // Auto-join the default topic and any previously joined topics.
        for topic in store.topics where topic.isJoined {
            subscribe(to: topic)
        }
        if store.topics.isEmpty || !store.topics.contains(where: { $0.isJoined }) {
            joinTopic(named: defaultTopic)
        }
        if selectedTopicId == nil {
            selectedTopicId = store.topics.first?.id
        }
    }

    func p2pNodeDidStop(_ node: P2PNode) {
        log("p2pNodeDidStop peerID=\(node.peerIDString)")
        subscriptions.removeAll()
        stopHeartbeatLoop()
    }
}

// MARK: - ReceiptTrackerDelegate

extension ChatService: ReceiptTrackerDelegate {
    func receiptTracker(_ tracker: ReceiptTracker, shouldRepublish envelope: ChatEnvelope, onTopic topic: String) {
        log("receiptTracker.shouldRepublish envelopeId=\(envelope.id) topic=\(topic)")
        guard let chatTopic = store.topic(named: topic) else { return }
        publish(envelope: envelope, topic: chatTopic)
    }
}
