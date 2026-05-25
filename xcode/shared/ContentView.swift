//
//  ContentView.swift
//  swiftyapp
//
//  Created by Jonathan McKenzie on 7/9/24.
//

import Foundation
import Combine
import SwiftUI
import RustyLib

struct ContentView: View {
    @State private var networkStatus = p2pNetworkStatus()
    @State private var networkLogs = p2pNetworkLogs()
    @State private var chatTopic = chatCurrentTopic()
    @State private var chatStatus = chatStatus()
    @State private var chatLogs = chatLogs()
    @State private var chatDraft = ""
    @State private var showingNetworkPanel = false
    @State private var didAutoStartServices = false
    @State private var logFontSize: CGFloat = 12

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            Color.clear
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .ignoresSafeArea()
        }
        .onAppear {
            if !didAutoStartServices {
                didAutoStartServices = true
                startServices()
            }
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            refreshSnapshots()
        }
    }

    private var headerBar: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("P2P + Chat")
                    .font(.headline)
                Text(networkStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                Text(chatStatus)
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Button {
                showingNetworkPanel = true
            } label: {
                Image(systemName: "gearshape.fill")
                    .font(.title3.weight(.semibold))
                    .padding(10)
                    .background(.thinMaterial, in: Circle())
                    .shadow(radius: 2)
            }
            .accessibilityLabel("P2P and chat settings")
            .fullScreenCover(isPresented: $showingNetworkPanel) {
                networkPanel
            }
        }
        .padding(.horizontal)
        .padding(.vertical, 12)
        .background(.regularMaterial)
    }

    private var networkPanel: some View {
        VStack(spacing: 0) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text("P2P + Chat")
                        .font(.headline)
                    Text(networkStatus)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.leading)
                    Text(chatStatus)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .multilineTextAlignment(.leading)
                }

                Spacer()

                Button("Done") {
                    showingNetworkPanel = false
                }
            }
            .padding()
            .background(.regularMaterial)

            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    p2pSection
                    chatSection
                    Divider()
                    Text(rustHello())
                    Text(String(rustAdd(a: 10, b: 32)))
                }
                .padding()
                .frame(maxWidth: .infinity, alignment: .topLeading)
            }
            .background(.background)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(.background)
    }

    private var p2pSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("P2P Network")
                    .font(.headline)
                Spacer()
                Button("Start") { startNetwork() }
                Button("Stop") { stopNetwork() }
                Button("Refresh") { refreshSnapshots() }
            }

            HStack {
                Spacer()
                Button {
                    adjustLogFontSize(by: -1)
                } label: {
                    Image(systemName: "minus")
                }
                .accessibilityLabel("Smaller logger text")
                Text("\(Int(logFontSize))")
                    .font(.caption.monospacedDigit())
                    .foregroundStyle(.secondary)
                    .frame(minWidth: 28)
                Button {
                    adjustLogFontSize(by: 1)
                } label: {
                    Image(systemName: "plus")
                }
                .accessibilityLabel("Larger logger text")
            }

            Text(networkStatus)
                .font(.caption)
                .foregroundStyle(.secondary)

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 4) {
                        if networkLogLines.isEmpty {
                            Text("No P2P logs yet.")
                                .font(logFont)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            ForEach(Array(networkLogLines.enumerated()), id: \.offset) { index, line in
                                Text(line)
                                    .font(logFont)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .id(index)
                            }
                        }
                    }
                    .textSelection(.enabled)
                }
                .frame(minHeight: 220, maxHeight: 280)
                .onAppear {
                    scrollToLatestLog(using: proxy, lines: networkLogLines)
                }
                .onChange(of: networkLogs) { _ in
                    scrollToLatestLog(using: proxy, lines: networkLogLines)
                }
            }
        }
    }

    private var chatSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Chat")
                .font(.headline)

            TextField("Topic", text: $chatTopic)
                .textFieldStyle(.roundedBorder)
                .textInputAutocapitalization(.never)
                .autocorrectionDisabled()

            HStack {
                Button("Start") { startChat() }
                Button("Stop") { stopChat() }
                Button("Refresh") { refreshSnapshots() }
                Spacer()
            }

            Text(chatStatus)
                .font(.caption)
                .foregroundStyle(.secondary)

            HStack(alignment: .top, spacing: 12) {
                TextField("Send a message", text: $chatDraft, axis: .vertical)
                    .textFieldStyle(.roundedBorder)
                    .lineLimit(1...4)
                Button("Send") { sendChatMessage() }
                    .buttonStyle(.borderedProminent)
            }

            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 4) {
                        if chatLogLines.isEmpty {
                            Text("No chat activity yet.")
                                .font(logFont)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        } else {
                            ForEach(Array(chatLogLines.enumerated()), id: \.offset) { index, line in
                                Text(line)
                                    .font(logFont)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .id(index)
                            }
                        }
                    }
                    .textSelection(.enabled)
                }
                .frame(minHeight: 220, maxHeight: 280)
                .onAppear {
                    scrollToLatestLog(using: proxy, lines: chatLogLines)
                }
                .onChange(of: chatLogs) { _ in
                    scrollToLatestLog(using: proxy, lines: chatLogLines)
                }
            }
        }
    }

    private var networkLogLines: [String] {
        networkLogs.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
    }

    private var chatLogLines: [String] {
        chatLogs.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
    }

    private var logFont: Font {
        .system(size: logFontSize, weight: .regular, design: .monospaced)
    }

    private func startServices() {
        startNetwork()
        startChat()
    }

    private func startNetwork() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = p2pNetworkStart()
            let logs = p2pNetworkLogs()
            DispatchQueue.main.async {
                networkStatus = status
                networkLogs = logs
            }
        }
    }

    private func stopNetwork() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = p2pNetworkStop()
            let logs = p2pNetworkLogs()
            DispatchQueue.main.async {
                networkStatus = status
                networkLogs = logs
            }
        }
    }

    private func startChat() {
        let topic = chatTopic
        DispatchQueue.global(qos: .userInitiated).async {
            let status = chatStart(topic: topic)
            let resolvedTopic = chatCurrentTopic()
            let logs = chatLogs()
            DispatchQueue.main.async {
                chatStatus = status
                chatTopic = resolvedTopic
                chatLogs = logs
            }
        }
    }

    private func stopChat() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = chatStop()
            let logs = chatLogs()
            DispatchQueue.main.async {
                chatStatus = status
                chatLogs = logs
            }
        }
    }

    private func sendChatMessage() {
        let message = chatDraft
        guard !message.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else { return }
        DispatchQueue.global(qos: .userInitiated).async {
            let status = chatSend(text: message)
            let logs = chatLogs()
            DispatchQueue.main.async {
                chatStatus = status
                chatLogs = logs
                chatDraft = ""
            }
        }
    }

    private func refreshSnapshots() {
        DispatchQueue.global(qos: .userInitiated).async {
            let networkStatus = p2pNetworkStatus()
            let networkLogs = p2pNetworkLogs()
            let chatStatus = chatStatus()
            let chatTopic = chatCurrentTopic()
            let chatLogs = chatLogs()
            DispatchQueue.main.async {
                self.networkStatus = networkStatus
                self.networkLogs = networkLogs
                self.chatStatus = chatStatus
                self.chatTopic = chatTopic
                self.chatLogs = chatLogs
            }
        }
    }

    private func scrollToLatestLog(using proxy: ScrollViewProxy, lines: [String]) {
        guard let lastIndex = lines.indices.last else { return }
        DispatchQueue.main.async {
            proxy.scrollTo(lastIndex, anchor: .bottom)
        }
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
    }
}

#Preview {
    ContentView()
}
