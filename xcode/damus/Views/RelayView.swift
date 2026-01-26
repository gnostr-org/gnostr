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

    func fetchRelayInfo() {
        let url = relay.descriptor.url
        var request = URLRequest(url: url)
        request.setValue("application/nostr+json", forHTTPHeaderField: "Accept")

        URLSession.shared.dataTask(with: request) { data, response, error in
            guard let data = data, error == nil else { return }

            let decoder = JSONDecoder()
            decoder.keyDecodingStrategy = .convertFromSnakeCase
            
            if let decodedInfo = try? decoder.decode(RelayInformation.self, from: data) {
                DispatchQueue.main.async {
                    self.relayInfo = decodedInfo
                }
            }
        }.resume()
    }
    
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
                } else {
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
            fetchRelayInfo()
        }
        .swipeActions {
            if let privkey = state.keypair.privkey {
                RemoveAction(privkey: privkey)
            }
        }
        .contextMenu {
            CopyAction(relay: relay.descriptor.url.absoluteString)
            
            if let privkey = state.keypair.privkey {
                RemoveAction(privkey: privkey)
            }
        }
    }
    
    func CopyAction(relay: String) -> some View {
        Button {
            UIPasteboard.general.setValue(relay, forPasteboardType: "public.plain-text")
        } label: {
            Label("Copy", systemImage: "doc.on.doc")
        }
    }
    
    func RemoveAction(privkey: String) -> some View {
        Button {
            guard let ev = state.contacts.event else {
                return
            }
            
            let descriptors = state.pool.descriptors
            guard let new_ev = remove_relay(ev: ev, current_relays: descriptors, privkey: privkey, relay: relay.descriptor.url.absoluteString) else {
                return
            }
            
            process_contact_event(pool: state.pool, contacts: state.contacts, pubkey: state.pubkey, ev: new_ev)
            state.pool.send(.event(new_ev))
        } label: {
            Label("Delete", systemImage: "trash")
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
