//
//  ChatModels.swift
//  GnostrP2P-KitchenSink
//

import Foundation

enum MessageStatus: String, Codable, CaseIterable, Equatable {
    case sending = "sending"
    case sent = "sent"
    case delivered = "delivered"
    case failed = "failed"
}

enum ChatEnvelopeKind: String, Codable, Equatable, CaseIterable {
    case chat = "Chat"
    case join = "Join"
    case leave = "Leave"
    case heartbeat = "Heartbeat"
    case receipt = "Received"
    case peerEvent = "Peer"
    case raw = "Raw"
}

struct ChatEnvelope: Codable, Identifiable, Equatable {
    var id: String
    var kind: ChatEnvelopeKind
    var senderId: String
    var senderName: String
    var topic: String
    var text: String
    var timestamp: Date
    var originalMessageId: String?

    init(
        id: String = UUID().uuidString,
        kind: ChatEnvelopeKind,
        senderId: String,
        senderName: String,
        topic: String,
        text: String,
        timestamp: Date = Date(),
        originalMessageId: String? = nil
    ) {
        self.id = id
        self.kind = kind
        self.senderId = senderId
        self.senderName = senderName
        self.topic = topic
        self.text = text
        self.timestamp = timestamp
        self.originalMessageId = originalMessageId
    }

    private enum CodingKeys: String, CodingKey {
        case id = "message_id"
        case kind
        case senderId = "from"
        case senderName
        case topic
        case text
        case timestamp
        case originalMessageId
        case content
        case commitId = "commit_id"
        case nostrEvent = "nostr_event"
        case sequenceNum = "sequence_num"
        case totalChunks = "total_chunks"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)

        // Prefer new field names, fall back to legacy RustChatMessage shape.
        self.id = (try? container.decode(String.self, forKey: .id))
            ?? (try? container.decode(String.self, forKey: .originalMessageId))
            ?? UUID().uuidString

        if let kindString = try? container.decode(String.self, forKey: .kind),
           let kind = ChatEnvelopeKind(rawValue: kindString) {
            self.kind = kind
        } else {
            self.kind = .chat
        }

        self.senderId = (try? container.decode(String.self, forKey: .senderId))
            ?? "unknown"
        self.senderName = (try? container.decode(String.self, forKey: .senderName))
            ?? self.senderId
        self.topic = (try? container.decode(String.self, forKey: .topic)) ?? ""

        if let text = try? container.decode(String.self, forKey: .text) {
            self.text = text
        } else if let content = try? container.decode([String].self, forKey: .content),
                  let first = content.first {
            self.text = first
        } else {
            self.text = ""
        }

        self.timestamp = (try? container.decode(Date.self, forKey: .timestamp)) ?? Date()
        self.originalMessageId = try? container.decode(String.self, forKey: .originalMessageId)
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(id, forKey: .id)
        try container.encode(kind.rawValue, forKey: .kind)
        try container.encode(senderId, forKey: .senderId)
        try container.encode(senderName, forKey: .senderName)
        try container.encode(topic, forKey: .topic)
        try container.encode(text, forKey: .text)
        try container.encode(timestamp, forKey: .timestamp)
        try container.encodeIfPresent(originalMessageId, forKey: .originalMessageId)

        // Legacy compatibility fields for older nodes.
        try container.encode([text], forKey: .content)
        try container.encode(String(repeating: "0", count: 40), forKey: .commitId)
        try container.encode([String: String](), forKey: .nostrEvent)
        try container.encode(1, forKey: .sequenceNum)
        try container.encode(1, forKey: .totalChunks)
    }
}

struct ChatMessage: Codable, Identifiable, Equatable {
    var id: String
    var topicId: String
    var envelopeId: String
    var kind: ChatEnvelopeKind
    var senderId: String
    var senderName: String
    var text: String
    var timestamp: Date
    var status: MessageStatus
    var isLocal: Bool

    init(
        id: String = UUID().uuidString,
        topicId: String,
        envelopeId: String,
        kind: ChatEnvelopeKind,
        senderId: String,
        senderName: String,
        text: String,
        timestamp: Date = Date(),
        status: MessageStatus = .sent,
        isLocal: Bool
    ) {
        self.id = id
        self.topicId = topicId
        self.envelopeId = envelopeId
        self.kind = kind
        self.senderId = senderId
        self.senderName = senderName
        self.text = text
        self.timestamp = timestamp
        self.status = status
        self.isLocal = isLocal
    }

    static func fromEnvelope(_ envelope: ChatEnvelope, topicId: String, isLocal: Bool, status: MessageStatus = .sent) -> ChatMessage {
        ChatMessage(
            id: UUID().uuidString,
            topicId: topicId,
            envelopeId: envelope.id,
            kind: envelope.kind,
            senderId: envelope.senderId,
            senderName: envelope.senderName,
            text: envelope.text,
            timestamp: envelope.timestamp,
            status: status,
            isLocal: isLocal
        )
    }
}

struct ChatTopic: Codable, Identifiable, Equatable {
    var id: String
    var name: String
    var joinedAt: Date
    var lastActivityAt: Date
    var isJoined: Bool

    init(
        id: String = UUID().uuidString,
        name: String,
        joinedAt: Date = Date(),
        lastActivityAt: Date = Date(),
        isJoined: Bool = true
    ) {
        self.id = id
        self.name = name
        self.joinedAt = joinedAt
        self.lastActivityAt = lastActivityAt
        self.isJoined = isJoined
    }
}

struct ChatPeer: Codable, Identifiable, Equatable {
    var id: String
    var addresses: [String]
    var displayName: String?
    var lastSeen: Date
    var isConnected: Bool

    init(
        id: String,
        addresses: [String] = [],
        displayName: String? = nil,
        lastSeen: Date = Date(),
        isConnected: Bool = false
    ) {
        self.id = id
        self.addresses = addresses
        self.displayName = displayName
        self.lastSeen = lastSeen
        self.isConnected = isConnected
    }
}
