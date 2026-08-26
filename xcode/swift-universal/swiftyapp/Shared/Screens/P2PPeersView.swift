//
//  P2PPeersView.swift
//  Universal App
//

import Combine
import RustyLib
import SwiftUI

struct P2PPeer: Decodable, Identifiable {
    let peer_id: String
    let source: String
    let addresses: [String]
    let last_seen_secs: UInt64

    var id: String { peer_id }

    var lastSeenText: String {
        let date = Date(timeIntervalSince1970: TimeInterval(last_seen_secs))
        return RelativeDateTimeFormatter().localizedString(for: date, relativeTo: .now)
    }
}

struct P2PPeersView: View {
    @State private var peers: [P2PPeer] = []

    var body: some View {
        List {
            if peers.isEmpty {
                Text("No peers discovered yet.")
                    .foregroundStyle(.secondary)
            } else {
                ForEach(peers) { peer in
                    VStack(alignment: .leading, spacing: 8) {
                        HStack(alignment: .firstTextBaseline) {
                            Text(peer.peer_id)
                                .font(.caption.monospaced())
#if !os(tvOS)
                                .textSelection(.enabled)
#endif
                            Spacer()
                            Text(peer.source)
                                .font(.caption.weight(.semibold))
                                .foregroundStyle(.secondary)
                        }

                        Text("Last seen \(peer.lastSeenText)")
                            .font(.caption)
                            .foregroundStyle(.secondary)

                        ForEach(peer.addresses, id: \.self) { address in
                            Text(address)
                                .font(.caption.monospaced())
#if !os(tvOS)
                                .textSelection(.enabled)
#endif
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
        }
        .navigationTitle("Peers")
        .toolbar {
            Button("Refresh", action: refreshPeers)
        }
        .onAppear {
            refreshPeers()
        }
        .onReceive(Timer.publish(every: 2.0, on: .main, in: .common).autoconnect()) { _ in
            refreshPeers()
        }
    }

    private func refreshPeers() {
        let data = Data(p2pNetworkPeers().utf8)
        peers = (try? JSONDecoder().decode([P2PPeer].self, from: data)) ?? []
    }
}

struct P2PPeersView_Previews: PreviewProvider {
    static var previews: some View {
        P2PPeersView()
    }
}
