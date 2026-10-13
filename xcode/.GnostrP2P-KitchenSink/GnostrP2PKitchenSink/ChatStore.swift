//
//  ChatStore.swift
//  GnostrP2P-KitchenSink
//

import Foundation
import SwiftUI

@MainActor
final class ChatStore: ObservableObject {
    @Published private(set) var topics: [ChatTopic] = []
    @Published private(set) var messages: [ChatMessage] = []
    @Published private(set) var peers: [ChatPeer] = []

    private let fileName = "GnostrP2PChatStore.json"
    private var saveTask: Task<Void, Never>?

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        let line = "[\(formatter.string(from: Date()))] ChatStore: \(message)"
        print(line)
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss.SSS"
        return formatter
    }()

    init() {
        load()
    }

    // MARK: - Topics

    func topic(named name: String) -> ChatTopic? {
        topics.first { $0.name == name }
    }

    func topic(id: String) -> ChatTopic? {
        topics.first { $0.id == id }
    }

    @discardableResult
    func addTopic(named name: String) -> ChatTopic {
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        if let existing = topic(named: trimmed) {
            log("addTopic existing=\(existing.name) id=\(existing.id)")
            return existing
        }
        let topic = ChatTopic(name: trimmed)
        topics.append(topic)
        log("addTopic name=\(trimmed) id=\(topic.id)")
        save()
        return topic
    }

    func removeTopic(id: String) {
        log("removeTopic id=\(id)")
        topics.removeAll { $0.id == id }
        messages.removeAll { $0.topicId == id }
        save()
    }

    func touchTopic(id: String) {
        guard let index = topics.firstIndex(where: { $0.id == id }) else { return }
        topics[index].lastActivityAt = Date()
        topics[index].isJoined = true
        save()
    }

    func setTopicJoined(id: String, joined: Bool) {
        guard let index = topics.firstIndex(where: { $0.id == id }) else { return }
        topics[index].isJoined = joined
        save()
    }

    // MARK: - Messages

    func messages(forTopicId topicId: String) -> [ChatMessage] {
        messages
            .filter { $0.topicId == topicId }
            .sorted { $0.timestamp < $1.timestamp }
    }

    @discardableResult
    func addMessage(_ message: ChatMessage) -> Bool {
        guard !messages.contains(where: { $0.envelopeId == message.envelopeId && $0.isLocal == message.isLocal }) else {
            log("addMessage duplicate ignored envelopeId=\(message.envelopeId) isLocal=\(message.isLocal)")
            return false
        }
        log("addMessage id=\(message.id) envelopeId=\(message.envelopeId) topicId=\(message.topicId) senderId=\(message.senderId) isLocal=\(message.isLocal)")
        messages.append(message)
        touchTopic(id: message.topicId)
        save()
        return true
    }

    func message(envelopeId: String, isLocal: Bool) -> ChatMessage? {
        messages.first { $0.envelopeId == envelopeId && $0.isLocal == isLocal }
    }

    func updateMessageStatus(envelopeId: String, isLocal: Bool, status: MessageStatus) {
        guard let index = messages.firstIndex(where: { $0.envelopeId == envelopeId && $0.isLocal == isLocal }) else {
            log("updateMessageStatus not found envelopeId=\(envelopeId) isLocal=\(isLocal)")
            return
        }
        log("updateMessageStatus envelopeId=\(envelopeId) isLocal=\(isLocal) status=\(status.rawValue)")
        messages[index].status = status
        save()
    }

    func clearMessages(forTopicId topicId: String) {
        messages.removeAll { $0.topicId == topicId }
        save()
    }

    // MARK: - Peers

    func peer(id: String) -> ChatPeer? {
        peers.first { $0.id == id }
    }

    func recordPeer(id: String, addresses: [String], displayName: String?, isConnected: Bool) {
        if let index = peers.firstIndex(where: { $0.id == id }) {
            peers[index].addresses = addresses
            peers[index].displayName = displayName ?? peers[index].displayName
            peers[index].lastSeen = Date()
            peers[index].isConnected = isConnected
            log("recordPeer updated id=\(id) addresses=\(addresses) displayName=\(displayName ?? "nil") isConnected=\(isConnected)")
        } else {
            peers.append(ChatPeer(id: id, addresses: addresses, displayName: displayName, isConnected: isConnected))
            log("recordPeer new id=\(id) addresses=\(addresses) displayName=\(displayName ?? "nil") isConnected=\(isConnected)")
        }
        save()
    }

    func updatePeerConnection(id: String, isConnected: Bool) {
        guard let index = peers.firstIndex(where: { $0.id == id }) else {
            log("updatePeerConnection unknown id=\(id) isConnected=\(isConnected)")
            return
        }
        peers[index].isConnected = isConnected
        if isConnected {
            peers[index].lastSeen = Date()
        }
        log("updatePeerConnection id=\(id) isConnected=\(isConnected)")
        save()
    }

    // MARK: - Persistence

    private var storeURL: URL {
        FileManager.default.urls(for: .documentDirectory, in: .userDomainMask)[0]
            .appendingPathComponent(fileName)
    }

    private struct StoredData: Codable {
        var topics: [ChatTopic]
        var messages: [ChatMessage]
        var peers: [ChatPeer]
    }

    private func load() {
        do {
            let data = try Data(contentsOf: storeURL)
            let stored = try JSONDecoder().decode(StoredData.self, from: data)
            topics = stored.topics
            messages = stored.messages
            peers = stored.peers
            log("load success url=\(storeURL.path) topics=\(topics.count) messages=\(messages.count) peers=\(peers.count)")
        } catch {
            topics = []
            messages = []
            peers = []
            log("load failed (first launch or missing file) url=\(storeURL.path) error=\(error.localizedDescription)")
        }
    }

    private func save() {
        saveTask?.cancel()
        saveTask = Task { [weak self] in
            try? await Task.sleep(nanoseconds: 200_000_000)
            guard let self, !Task.isCancelled else { return }
            let stored = StoredData(topics: self.topics, messages: self.messages, peers: self.peers)
            do {
                let data = try JSONEncoder().encode(stored)
                try data.write(to: self.storeURL, options: .atomic)
                log("save success url=\(self.storeURL.path) topics=\(stored.topics.count) messages=\(stored.messages.count) peers=\(stored.peers.count)")
            } catch {
                log("save failure url=\(self.storeURL.path) error=\(error.localizedDescription)")
            }
        }
    }
}
