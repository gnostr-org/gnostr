//
//  RelayView.swift
//  damus
//
//  Created by William Casarin on 2022-10-16.
//

import SwiftUI

struct RelayView: View {
    let state: DamusState
    @ObservedObject var relay: Relay
    
    let timer = Timer.publish(every: 2, on: .main, in: .common).autoconnect()
    @State var conn_color: Color = .gray
    @State private var isSelected: Bool = false
    @State private var relayInfo: RelayInformation?
    @State private var relayInfoError: String?
    
    func update_connection_color() {
        let c = relay.connection
        if c.isConnected {
            conn_color = .green
        } else if c.isConnecting || c.isReconnecting {
            conn_color = .yellow
        } else {
            conn_color = .red
        }
    }
    
    var body: some View {
        VStack(alignment: .leading, spacing: 5) {
            HStack {
                Circle()
                    .frame(width: 8.0, height: 8.0)
                    .foregroundColor(conn_color)
                Text(relayInfo?.name ?? relay.descriptor.url.absoluteString)
                    .font(isSelected ? .title2 : .headline)
                Spacer()
                if let pingTime = relay.connection.pingTime {
                    Text(String(format: "%.0f ms", pingTime * 1000))
                        .font(.caption)
                        .foregroundColor(.gray)
                }
            }
            Text(relay.descriptor.url.absoluteString)
                .font(.caption)
                .foregroundColor(.gray)
            if let info = relayInfo {
                Text(info.supportsNIP34 ? "NIP-34 advertised" : "NIP-34 not advertised")
                    .font(.caption)
                    .foregroundColor(info.supportsNIP34 ? .green : .secondary)
            } else if let error = relayInfoError {
                Text("NIP-34 unknown: \(error)")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            
            if isSelected {
                if let info = relayInfo {
                    if let description = info.description, !description.isEmpty {
                        Text(description)
                            .font(.body)
                            .padding(.top, 4)
                    }
                    if let supportedNips = info.supportedNips {
                        Text("NIPs: \(supportedNips.map(String.init).joined(separator: ", "))")
                            .font(.subheadline)
                            .foregroundColor(.gray)
                            .padding(.top, 2)
                    }
                    if let contact = info.contact, !contact.isEmpty {
                        Text("Contact: \(contact)")
                            .font(.footnote)
                            .foregroundColor(.gray)
                    }
                    if let software = info.software, !software.isEmpty {
                        Text("Software: \(software)")
                            .font(.footnote)
                            .foregroundColor(.gray)
                    }
                    if let version = info.version, !version.isEmpty {
                        Text("Version: \(version)")
                            .font(.footnote)
                            .foregroundColor(.gray)
                    }
                } else if relayInfoError == nil {
                    ProgressView() // Show a loading indicator while fetching
                }
            } else {
                if let description = relayInfo?.description, !description.isEmpty {
                    Text(description)
                        .font(.footnote)
                        .lineLimit(1)
                }
            }
        }
        .padding(.vertical, 5)
        .contentShape(Rectangle())
        .onTapGesture {
            withAnimation {
                self.isSelected.toggle()
            }
        }
        .onReceive(timer) { _ in
            update_connection_color()
        }
        .onAppear() {
            update_connection_color()
        }
        .task(id: relay.descriptor.url) {
            do {
                relayInfo = try await state.pool.load_relay_information(relay.descriptor.url)
                relayInfoError = nil
            } catch {
                guard !Task.isCancelled else { return }
                relayInfoError = error.localizedDescription
                print("NIP-11 fetch failed for \(relay.descriptor.url): \(error)")
            }
        }
        .swipeActions {
            RemoveAction()
        }
        .contextMenu {
            CopyAction(relay: relay.descriptor.url.absoluteString)
            
            RemoveAction()
        }
    }
    
    func CopyAction(relay: String) -> some View {
        Button {
            UIPasteboard.general.setValue(relay, forPasteboardType: "public.plain-text")
        } label: {
            Label("Copy", systemImage: "doc.on.doc")
        }
    }
    
    func RemoveAction() -> some View {
        Button {
            let descriptors = state.pool.descriptors
            state.pool.remove_relay(relay.id)
            if let ev = state.contacts.event, let privkey = state.keypair.privkey,
               let new_ev = remove_relay(ev: ev, current_relays: descriptors, privkey: privkey,
                                        relay: relay.descriptor.url.absoluteString) {
                process_contact_event(pool: state.pool, contacts: state.contacts, pubkey: state.pubkey, ev: new_ev)
                state.pool.send(.event(new_ev))
            }
        } label: {
            Label("Remove", systemImage: "minus.circle")
        }
        .tint(.red)
    }
}

struct RelayView_Previews: PreviewProvider {
    static var previews: some View {
        let url = URL(string: "wss://relay.damus.io")!
        let desc = RelayDescriptor(url: url, info: .rw)
        let conn = RelayConnection(url: url) { event in }
        let relay = Relay(descriptor: desc, connection: conn)
        RelayView(state: test_damus_state(), relay: relay, conn_color: .red)
    }
}
