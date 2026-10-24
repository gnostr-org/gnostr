//
//  RecommendedRelayView.swift
//  damus
//
//  Created by William Casarin on 2022-12-29.
//

import SwiftUI

struct RecommendedRelayView: View {
    let damus: DamusState
    let relay: String
    let add_button: Bool
    let info: RelayInfo
    @State private var relayInfo: RelayInformation?
    @State private var relayInfoError: String?
    
    init(damus: DamusState, relay: String, info: RelayInfo = .rw, add_button: Bool = true) {
        self.damus = damus
        self.relay = relay
        self.add_button = add_button
        self.info = info
    }
    
    var body: some View {
        HStack {
            VStack(alignment: .leading) {
                Text(relay)
                if !info.write {
                    Text("Read only").font(.caption).foregroundColor(.secondary)
                } else if !info.read {
                    Text("Write only").font(.caption).foregroundColor(.secondary)
                }
                if let information = relayInfo {
                    Text(information.supportsNIP34 ? "NIP-34 advertised" : "NIP-34 not advertised")
                        .font(.caption)
                        .foregroundColor(information.supportsNIP34 ? .green : .secondary)
                } else if let error = relayInfoError {
                    Text("NIP-34 unknown: \(error)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    Text("Checking NIP-34 support").font(.caption).foregroundColor(.secondary)
                }
            }
            .task(id: relay) {
                do {
                    guard let url = URL(string: relay) else { throw URLError(.badURL) }
                    relayInfo = try await fetch_relay_information(url)
                    relayInfoError = nil
                } catch {
                    guard !Task.isCancelled else { return }
                    relayInfoError = error.localizedDescription
                    print("NIP-11 fetch failed for \(relay): \(error)")
                }
            }
            Spacer()
            if let ev = damus.contacts.event, add_button {
                if let privkey = damus.keypair.privkey {
                    Button("Add") {
                        guard let ev = add_relay(ev: ev, privkey: privkey, current_relays: damus.pool.descriptors, relay: relay, info: info) else {
                            return
                        }
                        process_contact_event(pool: damus.pool, contacts: damus.contacts, pubkey: damus.pubkey, ev: ev)
                        damus.pool.send(.event(ev))
                    }
                    .buttonStyle(.borderless)
                }
            }
        }
    }
}

struct RecommendedRelayView_Previews: PreviewProvider {
    static var previews: some View {
        RecommendedRelayView(damus: test_damus_state(), relay: "wss://relay.damus.io")
    }
}
