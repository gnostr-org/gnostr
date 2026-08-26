//
//  Universal_AppApp.swift
//  Shared
//
//  Created by Can Balkaya on 12/10/20.
//

import Darwin
import RustyLib
import SwiftUI

@main
struct Universal_AppApp: App {
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            AppLifecycleView()
                .environmentObject(appState)
        }
    }
}

final class AppState: ObservableObject {
    @Published var privateKey: String = ""
    @Published var discoveryTopic: String = "gnostr/p2p/presence"
    @Published var chatTopic: String = "gnostr-dev"
    @Published var networkProtocol: String = "/ipfs/kad"
    @Published var networkVersion: String = "1.0.0"
    @Published var chatTopicsRefreshToken = UUID()

    func notifyChatTopicsChanged() {
        chatTopicsRefreshToken = UUID()
    }
}

private struct AppLifecycleView: View {
    @EnvironmentObject private var appState: AppState
    @Environment(\.scenePhase) private var scenePhase
    @State private var didStartNetwork = false
    @State private var configRestartTask: Task<Void, Never>?

    var body: some View {
        MainView()
            .onAppear {
                applyPrivateKeyEnvironment()
                applyP2PConfigurationEnvironment()
                startNetworkIfNeeded()
            }
            .onChange(of: appState.privateKey) { _ in
                applyPrivateKeyEnvironment()
                restartNetworkIfNeeded()
            }
            .onChange(of: appState.discoveryTopic) { _ in
                applyP2PConfigurationEnvironment()
                scheduleNetworkRestartIfNeeded()
            }
            .onChange(of: appState.chatTopic) { _ in
                applyP2PConfigurationEnvironment()
                scheduleNetworkRestartIfNeeded()
            }
            .onChange(of: appState.networkProtocol) { _ in
                applyP2PConfigurationEnvironment()
                scheduleNetworkRestartIfNeeded()
            }
            .onChange(of: appState.networkVersion) { _ in
                applyP2PConfigurationEnvironment()
                scheduleNetworkRestartIfNeeded()
            }
            .onChange(of: scenePhase) { phase in
                switch phase {
                case .active:
                    applyPrivateKeyEnvironment()
                    applyP2PConfigurationEnvironment()
                    startNetworkIfNeeded()
                case .background:
                    stopNetwork()
                case .inactive:
                    break
                @unknown default:
                    break
                }
            }
    }

    private func startNetworkIfNeeded() {
        guard !didStartNetwork else { return }
        configRestartTask?.cancel()
        didStartNetwork = true
        applyP2PConfigurationEnvironment()
        _ = p2pNetworkStart()
    }

    private func restartNetworkIfNeeded() {
        configRestartTask?.cancel()
        guard didStartNetwork else { return }
        _ = p2pNetworkStop()
        didStartNetwork = false
        startNetworkIfNeeded()
    }

    private func scheduleNetworkRestartIfNeeded() {
        guard didStartNetwork else { return }
        configRestartTask?.cancel()
        configRestartTask = Task { @MainActor in
            try? await Task.sleep(for: .milliseconds(250))
            guard !Task.isCancelled else { return }
            restartNetworkIfNeeded()
        }
    }

    private func stopNetwork() {
        configRestartTask?.cancel()
        _ = p2pNetworkStop()
        didStartNetwork = false
    }

    private func applyPrivateKeyEnvironment() {
        if appState.privateKey.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            unsetenv("GNOSTR_NSEC")
        } else {
            appState.privateKey.withCString { pointer in
                setenv("GNOSTR_NSEC", pointer, 1)
            }
        }
    }

    private func applyP2PConfigurationEnvironment() {
        setP2PEnvironment("GNOSTR_P2P_DISCOVERY_TOPIC", value: appState.discoveryTopic, fallback: "gnostr/p2p/presence")
        setP2PEnvironment("GNOSTR_P2P_CHAT_TOPIC", value: appState.chatTopic, fallback: "gnostr-dev")
        setP2PEnvironment("GNOSTR_P2P_PROTOCOL", value: appState.networkProtocol, fallback: "/ipfs/kad")
        setP2PEnvironment("GNOSTR_P2P_PROTOCOL_VERSION", value: appState.networkVersion, fallback: "1.0.0")
    }

    private func setP2PEnvironment(_ key: String, value: String, fallback: String) {
        let trimmed = value.trimmingCharacters(in: .whitespacesAndNewlines)
        let resolved = trimmed.isEmpty ? fallback : trimmed
        resolved.withCString { pointer in
            setenv(key, pointer, 1)
        }
    }
}
