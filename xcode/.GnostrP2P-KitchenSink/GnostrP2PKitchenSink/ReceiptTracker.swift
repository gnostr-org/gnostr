//
//  ReceiptTracker.swift
//  GnostrP2P-KitchenSink
//

import Foundation

@MainActor
protocol ReceiptTrackerDelegate: AnyObject {
    func receiptTracker(_ tracker: ReceiptTracker, shouldRepublish envelope: ChatEnvelope, onTopic topic: String)
}

@MainActor
final class ReceiptTracker: ObservableObject {
    struct PendingMessage: Identifiable, Equatable {
        let id: String
        let envelope: ChatEnvelope
        let topic: String
        var sendAttempts: Int
        var lastSentAt: Date
        var status: MessageStatus

        init(envelope: ChatEnvelope, topic: String) {
            self.id = envelope.id
            self.envelope = envelope
            self.topic = topic
            self.sendAttempts = 1
            self.lastSentAt = Date()
            self.status = .sending
        }
    }

    @Published private(set) var pending: [PendingMessage] = []

    weak var delegate: ReceiptTrackerDelegate?

    private var seenInboundMessageIds = Set<String>()
    private var retryTask: Task<Void, Never>?

    private let retryInterval: TimeInterval = 5
    private let maxRetryAttempts = 3

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        let line = "[\(formatter.string(from: Date()))] ReceiptTracker: \(message)"
        print(line)
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss.SSS"
        return formatter
    }()

    init() {
        log("init")
        startRetryLoop()
    }

    deinit {
        retryTask?.cancel()
    }

    // MARK: - Outbound tracking

    @discardableResult
    func track(_ envelope: ChatEnvelope, topic: String) -> PendingMessage {
        let pendingMessage = PendingMessage(envelope: envelope, topic: topic)
        pending.append(pendingMessage)
        log("track envelopeId=\(envelope.id) topic=\(topic) attempts=\(pendingMessage.sendAttempts)")
        return pendingMessage
    }

    func markSent(messageId: String) {
        guard let index = pending.firstIndex(where: { $0.id == messageId }) else {
            log("markSent not found messageId=\(messageId)")
            return
        }
        if pending[index].status == .sending {
            pending[index].status = .sent
            log("markSent messageId=\(messageId) status=sent")
        }
    }

    func recordReceipt(originalMessageId: String) {
        guard let index = pending.firstIndex(where: { $0.envelope.id == originalMessageId }) else {
            log("recordReceipt not found originalMessageId=\(originalMessageId)")
            return
        }
        pending[index].status = .delivered
        log("recordReceipt originalMessageId=\(originalMessageId) status=delivered")
    }

    func fail(messageId: String) {
        guard let index = pending.firstIndex(where: { $0.id == messageId }) else { return }
        pending[index].status = .failed
        log("fail messageId=\(messageId)")
    }

    func remove(messageId: String) {
        log("remove messageId=\(messageId)")
        pending.removeAll { $0.id == messageId }
    }

    func clear() {
        log("clear")
        pending.removeAll()
    }

    // MARK: - Inbound deduplication

    func markInboundSeen(_ messageId: String) {
        log("markInboundSeen messageId=\(messageId)")
        seenInboundMessageIds.insert(messageId)
    }

    func isDuplicateInbound(_ messageId: String) -> Bool {
        let dup = seenInboundMessageIds.contains(messageId)
        if dup {
            log("isDuplicateInbound true messageId=\(messageId)")
        }
        return dup
    }

    // MARK: - Retry loop

    private func startRetryLoop() {
        retryTask?.cancel()
        retryTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: UInt64(TimeInterval(5) * 1_000_000_000))
                guard let self, !Task.isCancelled else { return }
                self.retryPending()
            }
        }
    }

    private func retryPending() {
        let now = Date()
        for index in pending.indices where pending[index].status != .delivered && pending[index].status != .failed {
            guard now.timeIntervalSince(pending[index].lastSentAt) >= retryInterval else { continue }
            guard pending[index].sendAttempts < maxRetryAttempts else {
                pending[index].status = .failed
                log("retryPending max attempts reached messageId=\(pending[index].id)")
                continue
            }
            pending[index].sendAttempts += 1
            pending[index].lastSentAt = now
            log("retryPending republish messageId=\(pending[index].id) attempt=\(pending[index].sendAttempts) topic=\(pending[index].topic)")
            delegate?.receiptTracker(self, shouldRepublish: pending[index].envelope, onTopic: pending[index].topic)
        }
    }
}
