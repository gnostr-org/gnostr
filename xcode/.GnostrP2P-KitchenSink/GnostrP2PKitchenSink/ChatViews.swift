//
//  ChatViews.swift
//  GnostrP2P-KitchenSink
//

import SwiftUI

struct ChatAppView: View {
    @StateObject var service: ChatService

    var body: some View {
        NavigationView {
            TopicListView(service: service)
            MessageListView(service: service)
        }
        .navigationViewStyle(DoubleColumnNavigationViewStyle())
    }
}

// MARK: - Network Info (P2P Tab)

struct NetworkInfoView: View {
    @StateObject var service: ChatService

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(title: "P2P Network", subtitle: "Node status, peer ID, listen addresses, and discovered peers.")

                GroupBox {
                    VStack(alignment: .leading, spacing: 10) {
                        HStack {
                            Circle()
                                .fill(statusColor)
                                .frame(width: 12, height: 12)
                            Text(service.node.state.rawValue.capitalized)
                                .font(.headline)
                            Spacer()
                        }

                        statRow(label: "Runtime profile", value: service.node.runtimeProfile)
                        statRow(label: "Listen port", value: "\(service.node.listenPort)")

                        VStack(alignment: .leading, spacing: 4) {
                            Text("Peer ID")
                                .font(.caption.bold())
                            Text(service.node.peerIDString)
                                .font(.callout.monospaced())
                                .textSelection(.enabled)
                        }

                        VStack(alignment: .leading, spacing: 4) {
                            Text("Display name")
                                .font(.caption.bold())
                            Text(service.node.displayName)
                                .font(.callout.monospaced())
                                .textSelection(.enabled)
                        }

                        HStack {
                            Button("Start node") { service.start() }
                                .disabled(service.node.state != .stopped)
                            Button("Stop node") { service.stop() }
                                .disabled(service.node.state != .running)
                            Button("Restart") { service.node.restart() }
                                .disabled(service.node.state != .running)
                        }

                        if let error = service.lastError ?? service.node.lastError {
                            Text("Error: \(error)")
                                .font(.caption)
                                .foregroundStyle(.red)
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Listen addresses")
                            .font(.title3.weight(.semibold))
                        if service.node.listenAddresses.isEmpty {
                            Text("Start the node to populate listen addresses.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(service.node.listenAddresses, id: \.self) { address in
                            Text(address)
                                .font(.callout.monospaced())
                                .textSelection(.enabled)
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Discovered peers")
                            .font(.title3.weight(.semibold))
                        if service.node.discoveredPeers.isEmpty {
                            Text("No peers discovered yet.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(service.node.discoveredPeers) { peer in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(peer.peerID)
                                    .font(.callout.monospaced())
                                    .textSelection(.enabled)
                                ForEach(peer.addresses, id: \.self) { address in
                                    Text(address)
                                        .font(.caption.monospaced())
                                        .textSelection(.enabled)
                                }
                            }
                            .padding(.vertical, 6)
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Message types")
                            .font(.title3.weight(.semibold))
                        VStack(alignment: .leading, spacing: 4) {
                            ForEach(ChatEnvelopeKind.allCases, id: \.self) { kind in
                                HStack {
                                    Text(kind.rawValue)
                                        .font(.caption.bold())
                                        .padding(.horizontal, 8)
                                        .padding(.vertical, 4)
                                        .background(Color.accentColor.opacity(0.15))
                                        .clipShape(Capsule())
                                    Spacer()
                                }
                            }
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        HStack {
                            Text("Activity log")
                                .font(.title3.weight(.semibold))
                            Spacer()
                            Button("Clear") { service.node.clearLogs() }
                        }
                        ForEach(service.node.logs, id: \.self) { line in
                            Text(line)
                                .font(.caption.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .textSelection(.enabled)
                        }
                    }
                }
            }
            .padding(16)
        }
        .textSelection(.enabled)
    }

    private var statusColor: Color {
        switch service.node.state {
        case .running:
            return .green
        case .starting:
            return .yellow
        case .stopping:
            return .orange
        case .stopped:
            return .red
        }
    }

    private func titleBlock(title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.largeTitle.bold())
            Text(subtitle)
                .font(.title3)
                .foregroundStyle(.secondary)
        }
    }

    private func statRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .font(.subheadline.weight(.semibold))
            Spacer()
            Text(value)
                .font(.body)
        }
    }
}

// MARK: - Topic List

struct TopicListView: View {
    @StateObject var service: ChatService
    @State private var isAddingTopic = false
    @State private var newTopicName = ""
    @State private var topicToDelete: ChatTopic?

    var body: some View {
        List(selection: Binding(
            get: { service.selectedTopicId },
            set: { newValue in
                if let newValue { service.selectTopic(id: newValue) }
            }
        )) {
            Section {
                NodeStatusRow(service: service)
            }

            Section(header: Text("Topics")) {
                ForEach(service.store.topics.sorted { $0.lastActivityAt > $1.lastActivityAt }) { topic in
                    TopicRow(topic: topic, isSelected: service.selectedTopicId == topic.id)
                        .tag(topic.id)
                }
                .onDelete { indexSet in
                    let sorted = service.store.topics.sorted { $0.lastActivityAt > $1.lastActivityAt }
                    for index in indexSet {
                        topicToDelete = sorted[index]
                    }
                }
            }
        }
        .listStyle(SidebarListStyle())
        .navigationTitle("GnostrP2P Chat")
        .toolbar {
            ToolbarItem(placement: .navigationBarTrailing) {
                Button {
                    isAddingTopic = true
                } label: {
                    Image(systemName: "plus")
                }
            }
        }
        .alert("Join Topic", isPresented: $isAddingTopic) {
            TextField("Topic name", text: $newTopicName)
            Button("Cancel", role: .cancel) {
                newTopicName = ""
            }
            Button("Join") {
                let name = newTopicName
                newTopicName = ""
                service.joinTopic(named: name)
            }
        } message: {
            Text("Enter a topic name to join.")
        }
        .alert("Leave and delete topic?", isPresented: Binding(
            get: { topicToDelete != nil },
            set: { if !$0 { topicToDelete = nil } }
        )) {
            Button("Cancel", role: .cancel) {
                topicToDelete = nil
            }
            Button("Delete", role: .destructive) {
                if let topic = topicToDelete {
                    service.deleteTopic(id: topic.id)
                }
                topicToDelete = nil
            }
        } message: {
            if let topic = topicToDelete {
                Text("Delete '\(topic.name)'? This cannot be undone.")
            } else {
                Text("")
            }
        }
    }
}

struct TopicRow: View {
    let topic: ChatTopic
    let isSelected: Bool

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text(topic.name)
                    .font(.headline)
                Text(topic.lastActivityAt, style: .time)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            if topic.isJoined {
                Circle()
                    .fill(Color.green)
                    .frame(width: 8, height: 8)
            }
        }
        .padding(.vertical, 4)
        .background(isSelected ? Color.accentColor.opacity(0.15) : Color.clear)
        .cornerRadius(8)
    }
}

struct NodeStatusRow: View {
    @StateObject var service: ChatService

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack {
                Circle()
                    .fill(statusColor)
                    .frame(width: 10, height: 10)
                Text(service.node.state.rawValue.capitalized)
                    .font(.subheadline.weight(.semibold))
                Spacer()
            }
            Text("Peer: \(service.node.peerIDString)")
                .font(.caption.monospaced())
                .foregroundStyle(.secondary)
                .lineLimit(nil)
                .textSelection(.enabled)
            Text("Port: \(service.node.listenPort)")
                .font(.caption)
                .foregroundStyle(.secondary)
            if let error = service.lastError ?? service.node.lastError {
                Text(error)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .lineLimit(2)
            }
        }
        .padding(.vertical, 4)
    }

    private var statusColor: Color {
        switch service.node.state {
        case .running:
            return .green
        case .starting:
            return .yellow
        case .stopping:
            return .orange
        case .stopped:
            return .red
        }
    }
}

// MARK: - Message List

struct MessageListView: View {
    @StateObject var service: ChatService

    var body: some View {
        Group {
            if let topic = service.store.topic(id: service.selectedTopicId ?? "") {
                MessageListContent(service: service, topic: topic)
                    .navigationTitle(topic.name)
            } else {
                EmptyTopicView()
            }
        }
    }
}

struct MessageListContent: View {
    @StateObject var service: ChatService
    let topic: ChatTopic

    @Namespace private var bottomID

    var body: some View {
        VStack(spacing: 0) {
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 12) {
                        ForEach(service.store.messages(forTopicId: topic.id)) { message in
                            MessageBubbleView(message: message)
                                .id(message.id)
                        }
                        Color.clear
                            .frame(height: 1)
                            .id(bottomID)
                    }
                    .padding()
                }
                .textSelection(.enabled)
                .onChange(of: service.store.messages(forTopicId: topic.id).count) { _ in
                    scrollToBottom(proxy: proxy)
                }
                .onAppear {
                    scrollToBottom(proxy: proxy)
                }
            }

            Divider()
            ComposerView(service: service, topic: topic)
                .padding()
        }
    }

    private func scrollToBottom(proxy: ScrollViewProxy) {
        DispatchQueue.main.async {
            withAnimation {
                proxy.scrollTo(bottomID, anchor: .bottom)
            }
        }
    }
}

struct EmptyTopicView: View {
    var body: some View {
        VStack(spacing: 12) {
            Image(systemName: "bubble.left.and.bubble.right")
                .font(.system(size: 56))
                .foregroundStyle(.secondary)
            Text("Select or join a topic to start chatting.")
                .font(.title3)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
        }
        .padding()
        .navigationTitle("Chat")
    }
}

// MARK: - Message Bubble

struct MessageBubbleView: View {
    let message: ChatMessage

    var body: some View {
        HStack {
            if message.isLocal { Spacer(minLength: 32) }

            VStack(alignment: message.isLocal ? .trailing : .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(message.senderName)
                        .font(.caption.bold())
                    Text(message.timestamp, style: .time)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                    Text(message.kind.rawValue)
                        .font(.caption2.bold())
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(kindColor.opacity(0.18))
                        .clipShape(Capsule())
                    if message.isLocal {
                        StatusIndicator(status: message.status)
                    }
                }

                Text(message.text)
                    .font(.body)
                    .padding(12)
                    .background(
                        RoundedRectangle(cornerRadius: 16, style: .continuous)
                            .fill(message.isLocal ? Color.accentColor.opacity(0.2) : Color.secondary.opacity(0.12))
                    )
                    .foregroundStyle(.primary)
            }
            .textSelection(.enabled)

            if !message.isLocal { Spacer(minLength: 32) }
        }
    }

    private var kindColor: Color {
        switch message.kind {
        case .chat:
            return .blue
        case .join:
            return .green
        case .leave:
            return .red
        case .heartbeat:
            return .orange
        case .receipt:
            return .purple
        case .peerEvent:
            return .cyan
        case .raw:
            return .gray
        }
    }
}

struct StatusIndicator: View {
    let status: MessageStatus

    var body: some View {
        Image(systemName: iconName)
            .font(.caption2)
            .foregroundStyle(color)
    }

    private var iconName: String {
        switch status {
        case .sending:
            return "clock"
        case .sent:
            return "checkmark"
        case .delivered:
            return "checkmark.circle"
        case .failed:
            return "exclamationmark.triangle"
        }
    }

    private var color: Color {
        switch status {
        case .sending:
            return .secondary
        case .sent:
            return .secondary
        case .delivered:
            return .green
        case .failed:
            return .red
        }
    }
}

// MARK: - Composer

struct ComposerView: View {
    @StateObject var service: ChatService
    let topic: ChatTopic

    @FocusState private var isFocused: Bool

    var body: some View {
        VStack(spacing: 8) {
            HStack(alignment: .bottom, spacing: 12) {
                TextEditor(text: $service.composerText)
                    .focused($isFocused)
                    .font(.body)
                    .frame(minHeight: 36, maxHeight: 120)
                    .padding(8)
                    .background(
                        RoundedRectangle(cornerRadius: 12, style: .continuous)
                            .fill(Color.secondary.opacity(0.1))
                    )
                    .onSubmit {
                        sendChat()
                    }

                Button {
                    sendChat()
                } label: {
                    Image(systemName: "arrow.up.circle.fill")
                        .font(.system(size: 32))
                }
                .disabled(service.composerText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || !service.node.isRunning)
            }

            HStack(spacing: 12) {
                Button {
                    sendRaw()
                } label: {
                    Label("Send raw", systemImage: "doc.plaintext")
                }
                .disabled(service.composerText.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || !service.node.isRunning)

                Button {
                    service.sendHeartbeat()
                } label: {
                    Label("Heartbeat", systemImage: "waveform")
                }
                .disabled(!service.node.isRunning)

                Spacer()
            }
            .font(.caption)
        }
    }

    private func sendChat() {
        let text = service.composerText
        service.sendMessage(text)
        isFocused = true
    }

    private func sendRaw() {
        let text = service.composerText
        service.sendRawMessage(text)
        isFocused = true
    }
}
