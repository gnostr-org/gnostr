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
    @State private var showingNetworkPanel = false
    @State private var showingCrawlerPanel = false
    @State private var showingSniperPanel = false
    @State private var didAutoStartNetwork = false
    @State private var didAutoStartCrawler = false
    @State private var didAutoStartSniper = false
    @State private var logFontSize: CGFloat = 12
    @StateObject private var crawlerStore = CrawlerNetworkStore()
    @StateObject private var sniperStore = SniperServiceStore()

    var body: some View {
        VStack(spacing: 0) {
            headerBar

            Color.clear
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .ignoresSafeArea()
        }
        .onAppear {
            if !didAutoStartNetwork {
                didAutoStartNetwork = true
                startNetwork()
            }
            if !didAutoStartCrawler {
                didAutoStartCrawler = true
                Task.detached(priority: .userInitiated) {
                    await crawlerStore.startCrawlerServe()
                }
            }
            if !didAutoStartSniper {
                didAutoStartSniper = true
                Task.detached(priority: .userInitiated) {
                    await sniperStore.startSniperService()
                }
            }
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            refreshNetworkSnapshot()
            Task {
                await crawlerStore.refresh()
                await sniperStore.refresh()
            }
        }
    }

    private var headerBar: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("64:P2P Network")
                    .font(.headline)
                Text(networkStatus)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Button {
                showingCrawlerPanel = true
            } label: {
                Image(systemName: "network")
                    .font(.title3.weight(.semibold))
                    .padding(10)
                    .background(.thinMaterial, in: Circle())
                    .foregroundStyle(crawlerStore.iconColor)
                    .shadow(radius: 2)
            }
            .accessibilityLabel("Crawler network")
            .fullScreenCover(isPresented: $showingCrawlerPanel) {
                CrawlerNetworkDashboard(store: crawlerStore)
            }

            Button {
                showingSniperPanel = true
            } label: {
                Image(systemName: "scope")
                    .font(.title3.weight(.semibold))
                    .padding(10)
                    .background(.thinMaterial, in: Circle())
                    .foregroundStyle(sniperStore.iconColor)
                    .shadow(radius: 2)
            }
            .accessibilityLabel("Sniper service")
            .fullScreenCover(isPresented: $showingSniperPanel) {
                SniperServiceDashboard(store: sniperStore)
            }

            Button {
                showingNetworkPanel = true
            } label: {
                Image(systemName: "gearshape.fill")
                    .font(.title3.weight(.semibold))
                    .padding(10)
                    .background(.thinMaterial, in: Circle())
                    .foregroundStyle(serviceIconColor(for: networkStatus))
                    .shadow(radius: 2)
            }
            .accessibilityLabel("P2P settings")
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
                    Text("124:P2P Network")
                        .font(.headline)
                    Text(networkStatus)
                        .font(.caption)
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

            VStack(alignment: .leading, spacing: 16) {
                HStack {
                    Button("Start") { startNetwork() }
                    Button("Stop") { stopNetwork() }
                    Button("Refresh Logs") { refreshNetworkSnapshot() }
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
                    .onAppear {
                        scrollToLatestLog(using: proxy)
                    }
                    .onChange(of: networkLogs) { _ in
                        scrollToLatestLog(using: proxy)
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                Divider()
                Text(rustHello())
                Text(String(rustAdd(a: 10, b: 32)))
            }
            .padding()
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            .textSelection(.enabled)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(.background)
    }

    private var networkLogLines: [String] {
        networkLogs.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
    }

    private var logFont: Font {
        .system(size: logFontSize, weight: .regular, design: .monospaced)
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

    private func refreshNetworkSnapshot() {
        DispatchQueue.global(qos: .userInitiated).async {
            let status = p2pNetworkStatus()
            let logs = p2pNetworkLogs()
            DispatchQueue.main.async {
                networkStatus = status
                networkLogs = logs
            }
        }
    }

    private func scrollToLatestLog(using proxy: ScrollViewProxy) {
        guard let lastIndex = networkLogLines.indices.last else { return }
        DispatchQueue.main.async {
            proxy.scrollTo(lastIndex, anchor: .bottom)
        }
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
    }

    private func serviceIconColor(for status: String) -> Color {
        let lowercased = status.lowercased()
        if lowercased.contains("running") {
            return .green
        }
        if lowercased.contains("starting") || lowercased.contains("stopping") || lowercased.contains("requested") {
            return .yellow
        }
        return .red
    }
}

#Preview {
    ContentView()
}
