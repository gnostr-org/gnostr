//
//  P2PListView.swift
//  Universal App
//

import Foundation
import CryptoKit
import RustyLib
import SwiftUI

struct P2PListView: View {
    @EnvironmentObject private var appState: AppState
    @State private var topicDraft: String = ""
    @State private var chatTopics: [String] = []

    var body: some View {
        List {
            Section("IDENTITY") {
                SecureField("Private key", text: $appState.privateKey)
                    .textInputAutocapitalization(.never)
                    .autocorrectionDisabled()
                    .font(.system(.caption, design: .monospaced))
            }

            Section("CHAT") {
                HStack(spacing: 8) {
                    TextField("Topic", text: $topicDraft)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                    Button(action: addTopic) {
                        Image(systemName: "plus.circle.fill")
                            .foregroundStyle(.green)
                    }
                    .accessibilityLabel("Add topic")
                    .disabled(normalizedDraft.isEmpty || chatTopics.contains(normalizedDraft))
                }

                ForEach(chatTopics, id: \.self) { topic in
                    NavigationLink(destination: P2PChatView(topic: topic)) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(topic)
                            if topic == defaultChatTopic {
                                Text("default")
                                    .font(.caption)
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                }
            }

            Section("P2P") {
                NavigationLink(destination: P2PPeersView()) {
                    Label("Peers", systemImage: "person.3.fill")
                }

                NavigationLink(destination: P2PServicesView()) {
                    Label("Services", systemImage: "antenna.radiowaves.left.and.right")
                }
            }
        }
        .navigationTitle("P2P")
        .onAppear {
            loadChatTopics()
        }
        .onChange(of: appState.privateKey) { _ in
            loadChatTopics()
        }
        .onChange(of: appState.chatTopicsRefreshToken) { _ in
            loadChatTopics()
        }
        #if os(macOS)
        .listStyle(SidebarListStyle())
        #endif
    }

    private var normalizedDraft: String {
        topicDraft.trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func addTopic() {
        let topic = normalizedDraft
        guard !topic.isEmpty, !chatTopics.contains(topic) else { return }

        chatTopics.append(topic)
        registerChatTopic(topic)
        guard let privateKey = normalizedPrivateKey else {
            print("Added chat topic in memory; private key required to persist it")
            topicDraft = ""
            return
        }

        do {
            try persistChatTopics(using: privateKey)
            appState.notifyChatTopicsChanged()
        } catch {
            print("Failed to persist chat topics: \(error)")
        }
        topicDraft = ""
    }

    private func loadChatTopics() {
        let defaultTopic = defaultChatTopic

        guard let privateKey = normalizedPrivateKey else {
            if chatTopics.isEmpty {
                chatTopics = [defaultTopic]
            }
            registerChatTopicsInRelay(chatTopics)
            return
        }

        do {
            let result = try loadChatTopicsFromDisk(using: privateKey)
            chatTopics = result.topics.isEmpty ? [defaultTopic] : result.topics
            registerChatTopicsInRelay(chatTopics)

            if result.needsMigration {
                do {
                    try persistChatTopics(using: privateKey)
                    appState.notifyChatTopicsChanged()
                    registerChatTopicsInRelay(chatTopics)
                } catch {
                    print("Failed to migrate legacy chat topics: \(error)")
                }
            }
        } catch {
            print("Failed to load chat topics: \(error)")
            if chatTopics.isEmpty {
                chatTopics = [defaultTopic]
            }
            registerChatTopicsInRelay(chatTopics)
        }
    }

    private func persistChatTopics(using privateKey: String) throws {
        guard let url = chatTopicsFileURL() else { return }

        let payload = try encryptTopics(chatTopics, using: privateKey)
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        let data = try JSONEncoder().encode(payload)
        try data.write(to: url, options: .atomic)
    }

    private func chatTopicsFileURL() -> URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first?
            .appendingPathComponent("gnostr", isDirectory: true)
            .appendingPathComponent("p2p-chat-topics.txt")
    }

    private var normalizedPrivateKey: String? {
        let trimmed = appState.privateKey.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
        return trimmed.isEmpty ? nil : trimmed
    }

    private var defaultChatTopic: String {
        let topic = appState.chatTopic.trimmingCharacters(in: .whitespacesAndNewlines)
        return topic.isEmpty ? "gnostr-dev" : topic
    }

    private func encryptTopics(_ topics: [String], using privateKey: String) throws -> EncryptedChatTopicsPayload {
        let key = SymmetricKey(data: Data(SHA256.hash(data: Data(privateKey.utf8))))
        let plaintext = try JSONEncoder().encode(topics)
        let sealedBox = try ChaChaPoly.seal(plaintext, using: key)
        return EncryptedChatTopicsPayload(
            nonce: Data(sealedBox.nonce).base64EncodedString(),
            ciphertext: sealedBox.ciphertext.base64EncodedString(),
            tag: sealedBox.tag.base64EncodedString()
        )
    }

    private func loadChatTopicsFromDisk(using privateKey: String) throws -> ChatTopicsLoadResult {
        guard let url = chatTopicsFileURL() else {
            throw ChatTopicsStoreError.missingFileURL
        }
        guard FileManager.default.fileExists(atPath: url.path) else {
            return ChatTopicsLoadResult(topics: [], needsMigration: false)
        }

        let data = try Data(contentsOf: url)

        if let payload = try? JSONDecoder().decode(EncryptedChatTopicsPayload.self, from: data) {
            let decryptedTopics = try decryptTopics(payload, using: privateKey)
            return ChatTopicsLoadResult(topics: decryptedTopics, needsMigration: false)
        }

        let legacyTopics = legacyTopics(from: data)
        return ChatTopicsLoadResult(topics: legacyTopics, needsMigration: !legacyTopics.isEmpty)
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

    private func registerChatTopic(_ topic: String) {
        _ = p2pNetworkRegisterChatTopic(topic: topic)
    }

    private func registerChatTopicsInRelay(_ topics: [String]) {
        topics.forEach { registerChatTopic($0) }
    }
}

private struct EncryptedChatTopicsPayload: Codable {
    let nonce: String
    let ciphertext: String
    let tag: String
}

private struct ChatTopicsLoadResult {
    let topics: [String]
    let needsMigration: Bool
}

private enum ChatTopicsStoreError: Error {
    case missingFileURL
    case invalidEncryptedPayload
}

struct P2PListView_Previews: PreviewProvider {
    static var previews: some View {
        P2PListView()
            .environmentObject(AppState())
    }
}
