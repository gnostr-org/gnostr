//
//  ContentView.swift
//  MiniP2PSample
//
//  Created by Lightech on 10/24/2048.
//

import Foundation
import CryptoKit
import LibP2P
import SwiftUI
#if os(iOS)
import UIKit
#endif

@MainActor
final class P2PService: ObservableObject {
    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }

    enum State: String {
        case stopped
        case starting
        case running
        case stopping
    }

    @Published private(set) var listenAddresses: [String] = []
    @Published private(set) var state: State = .stopped
    @Published private(set) var lastError: String?
    @Published private(set) var activityLog: [String] = []

    private var app: Application?
    private var runTask: Task<Void, Never>?

    let peerID: PeerID

    init() {
        peerID = Self.makePeerID(for: Self.runtimeProfile)
    }

    var runtimeProfile: String {
        Self.runtimeProfile.rawValue
    }

    var listenPort: Int {
        Self.listenPort
    }

    var stateLabel: String {
        state.rawValue.capitalized
    }

    var isRunning: Bool {
        state == .running
    }

    var peerIDString: String {
        peerID.b58String
    }

    func clearActivityLog() {
        activityLog.removeAll()
    }

    func start() {
        guard runTask == nil else { return }

        lastError = nil
        state = .starting
        log("Starting node")

        let app = Self.makeApplication(peerID: peerID)
        self.app = app

        app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) { [weak self, weak app] in
            guard let self, let app else { return }
            let addresses = app.listenAddresses.compactMap { address -> String? in
                guard let fullAddress = try? address.encapsulate(proto: .p2p, address: app.peerID.b58String) else {
                    return nil
                }
                return fullAddress.description
            }

            Task { @MainActor in
                self.listenAddresses = addresses
                if !addresses.isEmpty {
                    self.log("Listening on: \(addresses.joined(separator: ", "))")
                }
                if self.state == .starting {
                    self.state = .running
                    self.log("Node is running")
                }
            }
        }

        runTask = Task.detached(priority: .background) { [weak self, app] in
            do {
                await MainActor.run {
                    self?.log("Executing libp2p application")
                }
                try await app.execute()
            } catch {
                await MainActor.run {
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run {
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.log("Node stopped")
            }
        }
    }

    func stop() {
        guard let app else { return }

        state = .stopping
        log("Stopping node")
        self.app = nil
        self.runTask = nil

        Task.detached(priority: .background) { [weak self] in
            do {
                try await app.asyncShutdown()
            } catch {
                await MainActor.run {
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run {
                self?.listenAddresses = []
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.log("Node stopped")
            }
        }
    }

    func restart() {
        stop()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            self.start()
        }
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        app.listen(.tcp(host: "0.0.0.0", port: Self.listenPort))
        return app
    }

    private static var runtimeProfile: RuntimeProfile {
        #if targetEnvironment(macCatalyst)
        return .macCatalyst
        #elseif os(macOS)
        return .macOS
        #elseif os(iOS)
        if ProcessInfo.processInfo.isiOSAppOnMac {
            return .madeForiPad
        }
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return .iPad
        case .mac:
            return .madeForiPad
        default:
            return .iPhone
        }
        #else
        return .iPhone
        #endif
    }

    private static var listenPort: Int {
        if let value = ProcessInfo.processInfo.environment["P2P_LISTEN_PORT"],
           let port = Int(value),
           port > 0 {
            return port
        }

        switch runtimeProfile {
        case .macOS:
            return 10000
        case .iPhone:
            return 10001
        case .iPad:
            return 10002
        case .macCatalyst:
            return 10003
        case .madeForiPad:
            return 10004
        }
    }

    private static func makePeerID(for profile: RuntimeProfile) -> PeerID {
        let seed = Data(SHA256.hash(data: Data("MiniP2PSample.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        activityLog.append("[\(formatter.string(from: Date()))] \(message)")
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

struct ContentView: View {
    @StateObject private var service = P2PService()

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("MiniP2PSample").font(.largeTitle.bold())
            Text("Runtime profile: \(service.runtimeProfile)")
            Text("Peer ID: \(service.peerIDString)")
                .font(.caption.monospaced())
                .textSelection(.enabled)
            Text("Listen port: \(service.listenPort)")
            Text("State: \(service.stateLabel)")

            HStack {
                Button("Start node") {
                    service.start()
                }
                Button("Stop node") {
                    service.stop()
                }
                .disabled(!service.isRunning)
            }

            if let lastError = service.lastError {
                Text("Last error: \(lastError)")
                    .foregroundStyle(.red)
            }

            HStack {
                Text("Network activity").font(.headline)
                Spacer()
                Button("Clear") {
                    service.clearActivityLog()
                }
            }

            if service.activityLog.isEmpty {
                Text("No activity yet")
                    .foregroundStyle(.secondary)
            } else {
                List(service.activityLog.indices, id: \.self) { index in
                    Text(service.activityLog[index])
                        .font(.caption.monospaced())
                }
                .frame(minHeight: 180)
            }

            Divider()

            Text("Listening addresses").font(.headline)
            if service.listenAddresses.isEmpty {
                Text("No addresses yet")
                    .foregroundStyle(.secondary)
            } else {
                List(service.listenAddresses, id: \.self) { address in
                    Text(address)
                        .font(.caption.monospaced())
                }
                .frame(minHeight: 160)
            }
        }
        .padding()
        .onAppear {
            service.start()
        }
        .onDisappear {
            service.stop()
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
