//
//  P2PChatView.swift
//  Universal App
//

import Combine
import Foundation
import CryptoKit
import RustyLib
import SwiftUI

struct P2PChatView: View {
    @EnvironmentObject private var appState: AppState
    private let chatTopic: String
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()
    @State private var chatEntries: [P2PChatMessage] = []
    @State private var seenChatLogLines: Set<String> = []
    @State private var logFontSize: CGFloat = 12
    @State private var messageDraft: String = ""

    init(topic: String = "gnostr-dev") {
        self.chatTopic = topic.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? "gnostr-dev" : topic
    }

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 16) {
                        chatSummary
                        messagePanel
                        composerPanel
                    }
                    .padding()
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .onAppear {
                    scrollToLatestLog(using: proxy)
                }
                .onChange(of: chatEntries.count) { _ in
                    scrollToLatestLog(using: proxy)
                }
            }
        }
        .navigationTitle("Chat")
        .onAppear {
            registerChatTopic()
            refreshNetworkSnapshot()
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            refreshNetworkSnapshot()
        }
    }

    private var headerBar: some View {
        HStack(alignment: .top, spacing: 12) {
            VStack(alignment: .leading, spacing: 4) {
                Text("P2P Chat")
                    .font(.headline)
                Text(networkStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            VStack(alignment: .trailing, spacing: 8) {
                HStack(spacing: 8) {
                    Button("Refresh", action: refreshNetworkSnapshot)
                }
                .buttonStyle(.borderedProminent)

                HStack(spacing: 8) {
                    Button {
                        adjustLogFontSize(by: -1)
                    } label: {
                        Image(systemName: "minus")
                    }
                    .accessibilityLabel("Smaller chat log text")

                    Text("\(Int(logFontSize))")
                        .font(.caption.monospacedDigit())
                        .foregroundStyle(.secondary)
                        .frame(minWidth: 28)

                    Button {
                        adjustLogFontSize(by: 1)
                    } label: {
                        Image(systemName: "plus")
                    }
                    .accessibilityLabel("Larger chat log text")
                }
            }
        }
        .padding()
        .background(.regularMaterial)
    }

    private var chatSummary: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Chat messages")
                .font(.subheadline.weight(.semibold))

            VStack(alignment: .leading, spacing: 8) {
                topicRow(label: "Topic", value: effectiveChatTopic)
                topicRow(label: "Filter", value: "kind == Chat")
            }
            .padding()
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
    }

    private var messagePanel: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Chat stream")
                .font(.subheadline.weight(.semibold))

            ScrollView {
                LazyVStack(alignment: .leading, spacing: 4) {
                    if visibleChatEntries.isEmpty {
                        Text("No chat messages for this topic yet.")
                            .font(logFont)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    } else {
                        ForEach(Array(visibleChatEntries.enumerated()), id: \.element.id) { index, message in
                            chatBubble(message)
                                .id(index)
                        }
                    }
                }
#if !os(tvOS)
                .textSelection(.enabled)
#endif
                .padding()
            }
            .frame(minHeight: 340)
            .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        }
    }

    private var composerPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Send message")
                .font(.subheadline.weight(.semibold))

            HStack(spacing: 8) {
                TextField("Type a message", text: $messageDraft, axis: .vertical)
#if !os(tvOS)
                    .textFieldStyle(.roundedBorder)
#else
                    .textFieldStyle(.plain)
#endif
                    .lineLimit(1...4)
                    .onSubmit(sendCurrentMessage)

                Button("Send", action: sendCurrentMessage)
                    .buttonStyle(.borderedProminent)
                    .disabled(normalizedMessageDraft.isEmpty)
            }
        }
        .padding()
        .background(.thinMaterial, in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var logFont: Font {
        .system(size: logFontSize, weight: .regular, design: .monospaced)
    }

    private var effectiveChatTopic: String {
        let topic = chatTopic.trimmingCharacters(in: .whitespacesAndNewlines)
        if !topic.isEmpty {
            return topic
        }
        let fallback = appState.chatTopic.trimmingCharacters(in: .whitespacesAndNewlines)
        return fallback.isEmpty ? "gnostr-dev" : fallback
    }

    private var visibleChatEntries: [P2PChatMessage] {
        chatEntries.filter { $0.topic == effectiveChatTopic }
    }

    private var normalizedMessageDraft: String {
        messageDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func refreshNetworkSnapshot() {
        networkStatus = p2pNetworkStatus()
        networkLogs = p2pNetworkLogs()
        ingestChatLogs(networkLogs)
    }

    private func registerChatTopic() {
        do {
            let topic = effectiveChatTopic
            registerChatTopicInRelay(topic)

            guard let privateKey = normalizedPrivateKey else {
                print("Private key required to persist chat topics")
                return
            }

            guard let topicsURL = chatTopicsFileURL() else {
                throw ChatTopicsStoreError.missingFileURL
            }

            let existingTopics = try existingChatTopics(at: topicsURL, using: privateKey)
            var topics = existingTopics
            if !topics.contains(topic) {
                topics.append(topic)
            }

            try persistChatTopics(topics, using: privateKey, to: topicsURL)
            appState.notifyChatTopicsChanged()
        } catch {
            print("Failed to register chat topic: \(error)")
        }
    }

    private func sendCurrentMessage() {
        let message = normalizedMessageDraft
        guard !message.isEmpty else { return }

        let topic = effectiveChatTopic
        let result = p2pNetworkSendChatMessage(topic: topic, message: message)
        guard !result.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }

        appendLocalChatMessage(topic: topic, message: message)
        messageDraft = ""
    }

    private var normalizedPrivateKey: String? {
        let trimmed = appState.privateKey.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return trimmed.isEmpty ? nil : trimmed
    }

    private func chatTopicsFileURL() -> URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first?
            .appendingPathComponent("gnostr", isDirectory: true)
            .appendingPathComponent("p2p-chat-topics.txt")
    }

    private func existingChatTopics(at url: URL, using privateKey: String) throws -> [String] {
        guard FileManager.default.fileExists(atPath: url.path) else {
            return []
        }

        let data = try Data(contentsOf: url)
        if let payload = try? JSONDecoder().decode(EncryptedChatTopicsPayload.self, from: data) {
            return try decryptTopics(payload, using: privateKey)
        }

        return legacyTopics(from: data)
    }

    private func persistChatTopics(_ topics: [String], using privateKey: String, to url: URL) throws {
        let payload = try encryptTopics(topics, using: privateKey)
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        let data = try JSONEncoder().encode(payload)
        try data.write(to: url, options: .atomic)
    }

    private func encryptTopics(_ topics: [String], using privateKey: String) throws -> EncryptedChatTopicsPayload {
        let key = SymmetricKey(data: Data(SHA256.hash(data: Data(privateKey.utf8))))
        let plaintext = try JSONEncoder().encode(normalizedTopics(topics))
        let sealedBox = try ChaChaPoly.seal(plaintext, using: key)
        return EncryptedChatTopicsPayload(
            nonce: Data(sealedBox.nonce).base64EncodedString(),
            ciphertext: sealedBox.ciphertext.base64EncodedString(),
            tag: sealedBox.tag.base64EncodedString()
        )
    }

    private func decryptTopics(_ payload: EncryptedChatTopicsPayload, using privateKey: String) throws -> [String] {
        guard
            let nonceData = Data(base64Encoded: payload.nonce),
            let ciphertext = Data(base64Encoded: payload.ciphertext),
            let tag = Data(base64Encoded: payload.tag)
        else {
            throw ChatTopicsStoreError.invalidEncryptedPayload
        }

        let key = SymmetricKey(data: Data(SHA256.hash(data: Data(privateKey.utf8))))
        let sealedBox = try ChaChaPoly.SealedBox(
            nonce: try ChaChaPoly.Nonce(data: nonceData),
            ciphertext: ciphertext,
            tag: tag
        )
        let plaintext = try ChaChaPoly.open(sealedBox, using: key)
        return normalizedTopics(try JSONDecoder().decode([String].self, from: plaintext))
    }

    private func legacyTopics(from data: Data) -> [String] {
        let contents = String(decoding: data, as: UTF8.self)
        return normalizedTopics(
            contents
                .split(whereSeparator: { $0 == "\n" || $0 == "," })
                .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter { !$0.isEmpty }
        )
    }

    private func normalizedTopics(_ topics: [String]) -> [String] {
        var seen = Set<String>()
        var result: [String] = []

        for topic in topics {
            let trimmed = topic.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !trimmed.isEmpty, seen.insert(trimmed).inserted else { continue }
            result.append(trimmed)
        }

        return result
    }

    private func registerChatTopicInRelay(_ topic: String) {
        _ = p2pNetworkRegisterChatTopic(topic: topic)
    }

    private func scrollToLatestLog(using proxy: ScrollViewProxy) {
        guard let lastIndex = visibleChatEntries.indices.last else { return }
        DispatchQueue.main.async {
           proxy.scrollTo(lastIndex, anchor: .bottom)
        }
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
    }

    private func topicRow(label: String, value: String) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 12) {
            Text(label)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
                .frame(width: 86, alignment: .leading)
            Text(value)
                .font(.system(.caption, design: .monospaced))
#if !os(tvOS)
                .textSelection(.enabled)
#endif
            Spacer()
        }
    }

    private func appendLocalChatMessage(topic: String, message: String) {
        let localMessage = P2PChatMessage(
            from: "me",
            topic: topic,
            text: message,
            raw: message,
            isLocal: true
        )
        chatEntries.append(localMessage)
    }

    private func chatBubble(_ message: P2PChatMessage) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(alignment: .firstTextBaseline) {
                Text(message.from)
                    .font(.caption.weight(.semibold))
                Spacer()
                Text(message.topic)
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text(message.text)
                .font(.system(size: logFontSize, weight: .regular, design: .monospaced))
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .background(message.isLocal ? Color.blue.opacity(0.14) : Color.primary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func ingestChatLogs(_ logs: String) {
        for line in logs.split(separator: "\n", omittingEmptySubsequences: false).map(String.init) {
            guard !seenChatLogLines.contains(line),
                  let message = P2PChatMessage.parse(from: line) else {
                continue
            }
            seenChatLogLines.insert(line)
            chatEntries.append(message)
        }
    }
}

private struct EncryptedChatTopicsPayload: Codable {
    let nonce: String
    let ciphertext: String
    let tag: String
}

private enum ChatTopicsStoreError: Error {
    case missingFileURL
    case invalidEncryptedPayload
}

private struct P2PChatMessage: Identifiable {
    let id = UUID()
    let from: String
    let topic: String
    let text: String
    let raw: String
    let isLocal: Bool

    private struct Payload: Decodable {
        let from: String?
        let content: [String]?
        let kind: String?
    }

    static func parse(from line: String) -> P2PChatMessage? {
        guard line.contains("kind\":\"Chat") || line.contains("kind: \"Chat\"") || line.contains("kind':'Chat'") || line.contains("\"kind\":\"Chat\"") else {
            return nil
        }

        let receivedMarkers = [
            "Received message: '",
            "received message '"
        ]

        guard let payloadStart = receivedMarkers.compactMap({ line.range(of: $0) }).first,
              let topicRange = line.range(of: "' on topic '", range: payloadStart.upperBound..<line.endIndex),
              let peerRange = line.range(of: "' from peer", range: topicRange.upperBound..<line.endIndex)
        else {
            return nil
        }

        let payloadString = String(line[payloadStart.upperBound..<topicRange.lowerBound])
        let topicString = String(line[topicRange.upperBound..<peerRange.lowerBound])
        let peerString = String(line[peerRange.upperBound...]).trimmingCharacters(in: .whitespaces)
        let rawText = payloadString.trimmingCharacters(in: CharacterSet(charactersIn: "'"))

        if let data = rawText.data(using: .utf8),
           let decoded = try? JSONDecoder().decode(Payload.self, from: data) {
            let text = decoded.content?.first ?? rawText
            let from = decoded.from ?? peerString
            return P2PChatMessage(
                from: from,
                topic: topicString,
                text: text,
                raw: line,
                isLocal: from.contains("79be667e")
            )
        }

        return P2PChatMessage(
            from: peerString,
            topic: topicString,
            text: rawText,
            raw: line,
            isLocal: false
        )
    }
}

struct P2PChatView_Previews: PreviewProvider {
    static var previews: some View {
        P2PChatView()
            .environmentObject(AppState())
    }
}
