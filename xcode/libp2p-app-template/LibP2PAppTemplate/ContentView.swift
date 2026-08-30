//
//  ContentView.swift
//  libp2p-app-template
//

import Foundation
import SwiftUI

enum AppTab: String, CaseIterable, Hashable {
    case status = "Status"
    case p2p = "P2P"
    case chat = "Chat"
    case activity = "Activity"
}

struct ContentView: View {
    @StateObject private var p2p = P2PService()
    @State private var selectedTab = AppTab.chat

    var body: some View {
        TabView(selection: $selectedTab) {
            statusTab
                .tabItem { Label("Status", systemImage: "network") }
                .tag(AppTab.status)

            p2pTab
                .tabItem { Label("P2P", systemImage: "dot.radiowaves.left.and.right") }
                .tag(AppTab.p2p)

            chatTab
                .tabItem { Label("Chat", systemImage: "message.fill") }
                .tag(AppTab.chat)

            activityTab
                .tabItem { Label("Activity", systemImage: "text.bubble") }
                .tag(AppTab.activity)
        }
        .task {
            p2p.start()
        }
    }

    private var statusTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "LibP2P App Template",
                    subtitle: "A cross-platform SwiftUI app demonstrating swift-libp2p."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 10) {
                        statRow(label: "Runtime profile", value: p2p.runtimeProfile)
                        statRow(label: "Peer ID", value: p2p.peerIDString)
                            .textSelection(.enabled)
                        statRow(label: "Listen port", value: "\(p2p.listenPort)")
                        statRow(label: "State", value: p2pStateLabel)
                        statRow(label: "AutoNAT", value: p2p.autonatStatus)

                        HStack {
                            Button("Start node") { p2p.start() }
                            Button("Stop node") { p2p.stop() }
                                .disabled(!p2p.isRunning)
                            Button("Restart") { p2p.restart() }
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Listen addresses")
                            .font(.title3.weight(.semibold))
                        if p2p.listenAddresses.isEmpty {
                            Text("Start the node to populate listen addresses.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(p2p.listenAddresses, id: \.self) { address in
                            Text(address)
                                .font(.callout.monospaced())
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Discovered peers")
                            .font(.title3.weight(.semibold))
                        if p2p.discoveredPeers.isEmpty {
                            Text("No peers discovered yet.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(p2p.discoveredPeers) { peer in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(peer.peerID)
                                    .font(.callout.monospaced())
                                ForEach(peer.addresses, id: \.self) { address in
                                    Text(address)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .padding(.vertical, 4)
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private var p2pTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "P2P Node",
                    subtitle: "Control the libp2p node and inspect network state."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 10) {
                        statRow(label: "State", value: p2pStateLabel)
                        statRow(label: "Peer ID", value: p2p.peerIDString)
                            .textSelection(.enabled)
                        statRow(label: "Listen port", value: "\(p2p.listenPort)")

                        HStack {
                            Button("Start node") { p2p.start() }
                            Button("Stop node") { p2p.stop() }
                                .disabled(!p2p.isRunning)
                            Button("Restart") { p2p.restart() }
                        }

                        HStack {
                            TextField("Ping message", text: $p2p.draftMessage)
                            Button("Queue ping") { p2p.sendLocalPing() }
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Listen addresses")
                            .font(.title3.weight(.semibold))
                        if p2p.listenAddresses.isEmpty {
                            Text("No addresses yet.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(p2p.listenAddresses, id: \.self) { address in
                            Text(address)
                                .font(.callout.monospaced())
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Discovered peers")
                            .font(.title3.weight(.semibold))
                        if p2p.discoveredPeers.isEmpty {
                            Text("No peers discovered yet.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(p2p.discoveredPeers) { peer in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(peer.peerID)
                                    .font(.callout.monospaced())
                                ForEach(peer.addresses, id: \.self) { address in
                                    Text(address)
                                        .font(.caption)
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .padding(.vertical, 4)
                        }
                    }
                }

            }
            .padding(16)
        }
    }

    private var chatTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "PubSub Chat",
                    subtitle: "Join a Gossipsub topic, send messages, and watch live updates."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        statRow(label: "Node state", value: p2pStateLabel)
                        statRow(label: "Current topic", value: p2p.chatTopic)
                        statRow(label: "Display name", value: p2p.chatDisplayName)

                        HStack {
                            TextField("Topic", text: $p2p.chatTopic)
                                .textFieldStyle(.roundedBorder)
                                .onSubmit {
                                    p2p.joinChatTopic()
                                }
                            Button("Join topic") {
                                p2p.joinChatTopic()
                            }
                        }

                        HStack {
                            TextField("Display name", text: $p2p.chatDisplayName)
                                .textFieldStyle(.roundedBorder)
                        }

                        HStack {
                            TextField("Message", text: $p2p.chatDraftMessage)
                                .textFieldStyle(.roundedBorder)
                            Button("Send") {
                                p2p.sendChatMessage()
                            }
                            .disabled(p2p.chatDraftMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Messages")
                            .font(.title3.weight(.semibold))
                        if p2p.chatMessages.isEmpty {
                            Text("No chat messages yet. Join \(p2p.chatTopic) and send one.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        LazyVStack(alignment: .leading, spacing: 10) {
                            ForEach(p2p.chatMessages) { entry in
                                VStack(alignment: .leading, spacing: 8) {
                                    HStack {
                                        Text(entry.topic)
                                            .font(.headline)
                                        Text(entry.kind)
                                            .font(.caption2.bold())
                                            .padding(.horizontal, 6)
                                            .padding(.vertical, 2)
                                            .background(.blue.opacity(0.18))
                                            .clipShape(Capsule())
                                        if entry.isLocal {
                                            Text("local")
                                                .font(.caption2.bold())
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(.green.opacity(0.2))
                                                .clipShape(Capsule())
                                        }
                                        Spacer()
                                        Text(entry.timestamp, style: .time)
                                            .font(.callout)
                                            .foregroundStyle(.secondary)
                                    }
                                    Text(entry.author)
                                        .font(.subheadline.monospaced())
                                        .foregroundStyle(.secondary)
                                    if entry.kind == "Ping", let sentMs = Int64(entry.text) {
                                        HStack(spacing: 6) {
                                            Image(systemName: "arrow.left.arrow.right")
                                                .font(.caption)
                                                .foregroundStyle(.secondary)
                                            if let delta = entry.pingDeltaMs {
                                                Text("Ping | \(delta)ms")
                                                    .font(.body.monospaced())
                                            } else if entry.isLocal {
                                                let date = Date(timeIntervalSince1970: Double(sentMs) / 1000.0)
                                                Text("Ping sent at \(date, style: .time)")
                                                    .font(.body.monospaced())
                                            } else {
                                                Text("Ping")
                                                    .font(.body.monospaced())
                                            }
                                        }
                                    } else {
                                        Text(entry.text)
                                            .font(.body)
                                    }
                                }
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(
                                    RoundedRectangle(cornerRadius: 14, style: .continuous)
                                        .fill(Color.secondary.opacity(0.08))
                                )
                            }
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private var activityTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "Activity Log",
                    subtitle: "Recent P2P events and state changes."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Button("Clear log") {
                            p2p.clearActivityLog()
                        }
                        if p2p.activityLog.isEmpty {
                            Text("No activity yet.")
                                .font(.callout)
                                .foregroundStyle(.secondary)
                        }
                        ForEach(p2p.activityLog, id: \.self) { entry in
                            Text(entry)
                                .font(.callout.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }
            .padding(16)
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

    private var p2pStateLabel: String {
        p2p.state.rawValue.capitalized
    }
}

#Preview {
    ContentView()
}
