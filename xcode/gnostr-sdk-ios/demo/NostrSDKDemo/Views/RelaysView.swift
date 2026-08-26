//
//  RelaysView.swift
//  NostrSDKDemo
//
//  Created by Bryan Montz on 1/1/24.
//

import Foundation
import GnostrSDK
import ContextAwareToolbar
import SwiftUI
import Combine

final class RelayDirectoryStore: ObservableObject {
    @Published var seenRelayURLs: Set<URL> = []
    private var relayPool: RelayPool?
    private var eventsCancellable: AnyCancellable?

    func attach(relayPool: RelayPool) {
        guard self.relayPool !== relayPool else { return }
        self.relayPool = relayPool

        eventsCancellable = relayPool.events
            .receive(on: DispatchQueue.main)
            .sink { [weak self] relayEvent in
                self?.record(seen: relayEvent.event)
            }
    }

    func record(seen relayURLs: [URL]) {
        seenRelayURLs.formUnion(relayURLs)
        deduplicateSeenRelayURLs()
    }

    func record(seen event: NostrEvent) {
        let relayURLs = event.tags
            .filter { $0.name == "relays" || $0.name == "r" }
            .flatMap { relayValues(from: $0) }
            .compactMap { normalizedRelayURL(from: $0) }
        seenRelayURLs.formUnion(relayURLs)
        deduplicateSeenRelayURLs()
    }

    func removeSeen(_ relayURL: URL) {
        seenRelayURLs.remove(relayURL)
    }

    func deduplicateSeenRelayURLs() {
        seenRelayURLs = Set(seenRelayURLs.compactMap { normalizedRelayURL(from: $0.absoluteString) })
    }

    private func normalizedRelayURL(from relayString: String) -> URL? {
        guard let url = URL(string: relayString),
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              components.scheme == "ws" || components.scheme == "wss" else {
            return nil
        }
        return url
    }

    private func relayValues(from tag: Tag) -> [String] {
        let rawValues = [tag.value] + tag.otherParameters
        return rawValues.flatMap { relayValues(fromRawValue: $0) }
    }

    private func relayValues(fromRawValue rawValue: String) -> [String] {
        let trimmedValue = rawValue.trimmingCharacters(in: .whitespacesAndNewlines)
        guard trimmedValue.first == "[",
              let data = trimmedValue.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data),
              let relayStrings = json as? [String] else {
            return [trimmedValue]
        }

        return relayStrings
    }
}

@MainActor
final class RelayInfoLoader: ObservableObject {
    @Published private(set) var relayInfoByURL: [URL: RelayInfo] = [:]
    @Published private(set) var loadingRelayURLs: Set<URL> = []

    private var inFlightTaskURLs: Set<URL> = []

    func refresh(relays: [Relay]) {
        relays.forEach { loadRelayInfo(for: $0.url) }
    }

    func relayInfo(for relayURL: URL) -> RelayInfo? {
        relayInfoByURL[relayURL]
    }

    func isLoading(_ relayURL: URL) -> Bool {
        loadingRelayURLs.contains(relayURL)
    }

    private func loadRelayInfo(for relayURL: URL) {
        guard relayInfoByURL[relayURL] == nil else { return }
        guard let infoURL = relayInfoURL(for: relayURL) else { return }
        guard inFlightTaskURLs.insert(relayURL).inserted else { return }

        loadingRelayURLs.insert(relayURL)

        Task.detached(priority: .utility) { [infoURL, relayURL] in
            defer {
                Task { @MainActor in
                    self.inFlightTaskURLs.remove(relayURL)
                    self.loadingRelayURLs.remove(relayURL)
                }
            }

            do {
                var request = URLRequest(url: infoURL)
                request.setValue("application/nostr+json", forHTTPHeaderField: "Accept")
                let (data, response) = try await URLSession.shared.data(for: request)
                guard let httpResponse = response as? HTTPURLResponse,
                      200..<300 ~= httpResponse.statusCode else {
                    return
                }

                let info = try JSONDecoder().decode(RelayInfo.self, from: data)
                await MainActor.run {
                    self.relayInfoByURL[relayURL] = info
                }
            } catch {
                print("[RelaysView] relay metadata fetch failed url=\(infoURL.absoluteString) error=\(error)")
            }
        }
    }

    private func relayInfoURL(for relayURL: URL) -> URL? {
        guard var components = URLComponents(url: relayURL, resolvingAgainstBaseURL: false) else {
            return nil
        }

        switch components.scheme {
        case "ws":
            components.scheme = "http"
        case "wss":
            components.scheme = "https"
        default:
            return nil
        }

        return components.url
    }
}

extension Relay {
    var statusColor: Color {
        switch state {
        case .connected:
            return .green
        case .connecting:
            return .yellow
        case .notConnected, .error:
            return .red
        }
    }

    var statusImage: some View {
        switch state {
        case .connected:
            return Image(systemName: "checkmark.circle")
                .foregroundStyle(statusColor)
        case .connecting:
            return Image(systemName: "arrow.triangle.2.circlepath.circle")
                .foregroundStyle(statusColor)
        case .notConnected, .error:
            return Image(systemName: "xmark.circle")
                .foregroundStyle(statusColor)
        }
    }

    var pingLabel: String? {
        guard let connectionLatency else { return nil }
        return String(format: "%.0f ms", connectionLatency * 1000)
    }
}

struct RelaysView: View {
    
    @EnvironmentObject var pool: RelayPool
    @EnvironmentObject var relayDirectory: RelayDirectoryStore
    @StateObject private var relayInfoLoader = RelayInfoLoader()
    @State private var expandedRelayURLs: Set<URL> = []
    @State private var relaySortOption: RelaySortOption = .urlAscending
    @State private var connectedRelaySortOption: RelaySortOption = .pingDescending
    @State private var activeNIPFilters: Set<Int> = []
    // Relays can resolve from connecting to connected/notConnected/error at any time, so the list needs a hard refresh token.
    @State private var relayStateRefreshToken = 0
    @State private var relayStateCancellable: AnyCancellable?
    @State private var isConnectedRelaysExpanded = true
    @State private var isConnectingRelaysExpanded = false
    @State private var isDisconnectedRelaysExpanded = false
    @State private var isSeenRelaysExpanded = false
    
    var body: some View {
        VStack(spacing: 0) {
            ContextAwareHeaderView(
                title: "Relays",
                subtitle: "Connected, connecting, and disconnected relay URLs.",
                systemImage: "network",
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)

            ContextAwareListToolbar(content: {
                ForEach(activeNIPFilters.sorted(), id: \.self) { nip in
                    Button {
                        activeNIPFilters.remove(nip)
                    } label: {
                        Label {
                            Text("NIP-\(nip)")
                                .font(.caption.weight(.semibold))
                        } icon: {
                            Image(systemName: "xmark.circle")
                        }
                        .padding(.horizontal, 10)
                        .padding(.vertical, 6)
                        .foregroundStyle(Color.accentColor)
                        .background(
                            RoundedRectangle(cornerRadius: 999, style: .continuous)
                                .fill(Color(.tertiarySystemFill))
                        )
                    }
                    .buttonStyle(.plain)
                }
            }, trailing: {
            }, horizontalPadding: 16)

            List {
                relaySection(title: "Connected",
                             relays: connectedRelaysSorted,
                             isExpanded: isConnectedRelaysExpanded,
                             setExpanded: { isConnectedRelaysExpanded = $0 },
                             removeAllAction: removeAllConnectedRelays,
                             sortSelection: $connectedRelaySortOption,
                             sortOption: connectedRelaySortOption)
                relaySection(title: "Connecting",
                             relays: groupedRelays.connecting,
                             isExpanded: isConnectingRelaysExpanded,
                             setExpanded: { isConnectingRelaysExpanded = $0 },
                             removeAllAction: removeAllConnectingRelays)
                relaySection(title: "Disconnected",
                             relays: groupedRelays.disconnected,
                             isExpanded: isDisconnectedRelaysExpanded,
                             setExpanded: { isDisconnectedRelaysExpanded = $0 },
                             removeAllAction: removeAllDisconnectedRelays,
                             showsReconnectButton: true)

                Section {
                    if isSeenRelaysExpanded {
                        if seenRelays.isEmpty {
                            Text("No seen relays yet.")
                                .foregroundColor(.secondary)
                        } else {
                            ForEach(Array(seenRelays), id: \.self) { relayURL in
                                seenRelayRow(for: relayURL)
                            }
                        }
                    }
                } header: {
                    ContextAwareListToolbar(content: {
                        Text("Seen Relays")
                    }, trailing: {
                        ContextAwareActionChipButton(title: isSeenRelaysExpanded ? "Hide" : "Show",
                                                     systemImage: isSeenRelaysExpanded ? "chevron.up" : "chevron.down") {
                            isSeenRelaysExpanded.toggle()
                        }

                        ContextAwareActionChipButton(title: "Add All",
                                                     systemImage: "plus.circle.fill",
                                                     isEnabled: seenRelays.isEmpty == false) {
                            addAllSeenRelays()
                        }

                        EditButton()
                    })
                }
            }
            // Force the filtered sections to rebuild when relay state changes.
            .id(relayStateRefreshToken)
            .onAppear {
                bindRelayStateUpdates()
                relayInfoLoader.refresh(relays: relays)
            }
            .onReceive(pool.$relays) { _ in
                bindRelayStateUpdates()
                relayInfoLoader.refresh(relays: relays)
                relayStateRefreshToken += 1
            }
            .onChange(of: activeNIPFilters) { _ in
                expandedRelayURLs.removeAll()
            }
        }
    }
    
    private var groupedRelays: (connected: [Relay], connecting: [Relay], disconnected: [Relay]) {
        let sorted = relays
        return (
            connected: sorted.filter { $0.state == .connected },
            connecting: sorted.filter { $0.state == .connecting },
            disconnected: sorted.filter {
                switch $0.state {
                case .notConnected, .error:
                    return true
                case .connected, .connecting:
                    return false
                }
            }
        )
    }

    private var relays: [Relay] {
        relaySortOption.sort(relays: Array(pool.relays))
    }

    private var connectedRelaysSorted: [Relay] {
        let connected = Array(pool.relays.filter { $0.state == .connected })
        let filtered = activeNIPFilters.isEmpty ? connected : connected.filter { relay in
            let supported = relayInfoLoader.relayInfo(for: relay.url)?.supportedNIPs ?? []
            return activeNIPFilters.isSubset(of: Set(supported))
        }
        return connectedRelaySortOption.sort(relays: filtered)
    }

    private var isNIPDisplayMode: Bool {
        activeNIPFilters.isEmpty == false
    }

    private func bindRelayStateUpdates() {
        // Keep the UI responsive if a relay stalls or flips state after an async connect attempt.
        relayStateCancellable = Publishers.MergeMany(
            pool.relays.map { relay in
                relay.$state.map { _ in () }.eraseToAnyPublisher()
            }
        )
        .receive(on: DispatchQueue.main)
        .sink { _ in
            relayStateRefreshToken += 1
        }
    }

    @ViewBuilder
    private func relaySection(title: String,
                              relays: [Relay],
                              isExpanded: Bool = true,
                              setExpanded: @escaping (Bool) -> Void = { _ in },
                              removeAllAction: (() -> Void)? = nil,
                              sortSelection: Binding<RelaySortOption>? = nil,
                              sortOption: RelaySortOption? = nil,
                              showsReconnectButton: Bool = false) -> some View {
        Section {
            if isExpanded == false {
            } else if relays.isEmpty {
                Text("No \(title.lowercased()) relays.")
                    .foregroundColor(.secondary)
            } else {
                ForEach(Array(relays), id: \.url) { relay in
                    relayCard(for: relay, showsReconnectButton: showsReconnectButton)
                }
            }
        } header: {
            ContextAwareListToolbar {
                Text(title)
            } trailing: {
                if let sortSelection {
                    ContextAwareSortToggleChip(selection: sortSelection,
                                               ascending: .pingAscending,
                                               descending: .pingDescending,
                                               ascendingTitle: "Ping ↑",
                                               descendingTitle: "Ping ↓")
                }

                ContextAwareActionChipButton(title: isExpanded ? "Hide" : "Show",
                                             systemImage: isExpanded ? "chevron.up" : "chevron.down") {
                    setExpanded(isExpanded == false)
                }

                if let removeAllAction {
                    ContextAwareActionChipButton(title: "Remove All",
                                                 systemImage: "minus.circle.fill",
                                                 role: .destructive,
                                                 isEnabled: relays.isEmpty == false) {
                        removeAllAction()
                    }
                }

                EditButton()
            }
        }
    }

    @ViewBuilder
    private func relayCard(for relay: Relay, showsReconnectButton: Bool) -> some View {
        // Keep the relay row compact; metadata is expanded inline so the user never loses the current list context.
        let info = relayInfoLoader.relayInfo(for: relay.url)
        Group {
        if isNIPDisplayMode {
            relayCardBody(for: relay, info: info, showsReconnectButton: showsReconnectButton)
        } else {
            DisclosureGroup(isExpanded: binding(for: relay.url)) {
                relayMetadataDetails(for: relay)
                    .padding(.top, 10)
            } label: {
                relayCardBody(for: relay, info: info, showsReconnectButton: showsReconnectButton)
            }
        }
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 8)
        .background(
        RoundedRectangle(cornerRadius: 12, style: .continuous)
            .fill(Color(.secondarySystemBackground))
        )
        .onAppear {
            relayInfoLoader.refresh(relays: [relay])
        }
    }

    @ViewBuilder
    private func relayCardBody(for relay: Relay,
                               info: RelayInfo?,
                               showsReconnectButton: Bool) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .center, spacing: 12) {
                VStack(alignment: .leading, spacing: 6) {
                    Text(relay.url.absoluteString)
                        .font(.subheadline)
                        .textSelection(.enabled)
                    if let pingLabel = relay.pingLabel {
                        Text("Ping \(pingLabel)")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                Spacer(minLength: 12)

                relay.statusImage.hidden()

                if showsReconnectButton {
                    ContextAwareActionChipButton(title: "Reconnect",
                                                 systemImage: "plus.circle.fill") {
                        relay.connect()
                    }
                }

                ContextAwareActionChipButton(title: "Remove",
                                             systemImage: "minus.circle.fill",
                                             role: .destructive) {
                    disconnect(relay)
                }
            }

            if isNIPDisplayMode, let supportedNIPs = info?.supportedNIPs?.sorted(), supportedNIPs.isEmpty == false {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 6) {
                        ForEach(supportedNIPs, id: \.self) { nip in
                            ContextAwareActionChipButton(title: "NIP-\(nip)",
                                                         systemImage: "network") {
                                toggleNIPFilter(nip)
                            }
                        }
                    }
                }
            }
        }
        .contentShape(Rectangle())
    }

    @ViewBuilder
    private func relayMetadataDetails(for relay: Relay) -> some View {
        let info = relayInfoLoader.relayInfo(for: relay.url)

        VStack(alignment: .leading, spacing: 10) {
            if relayInfoLoader.isLoading(relay.url) && info == nil {
                ProgressView("Loading relay metadata...")
                    .font(.caption)
            } else if let info {
                RelayMetadataDetailView(info: info,
                                        relayLabel: relay.url.absoluteString,
                                        connectedRelays: connectedRelays,
                                        relayInfoLoader: relayInfoLoader) { nip in
                    toggleNIPFilter(nip)
                }
            } else {
                Text("No relay metadata available.")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var seenRelays: [URL] {
        ContextAwareSortOrder.urlAscending.sort(urls: Array(relayDirectory.seenRelayURLs.filter { contains($0) == false }))
    }

    private var connectedRelays: [Relay] {
        relays.filter { $0.state == .connected }
    }

    private func contains(_ relayURL: URL) -> Bool {
        relays.contains(where: { $0.url == relayURL })
    }

    private func add(_ relayURL: URL) {
        guard contains(relayURL) == false, let relay = try? Relay(url: relayURL) else {
            return
        }
        pool.add(relay: relay)
        relayDirectory.removeSeen(relayURL)
        relayDirectory.deduplicateSeenRelayURLs()
    }

    private func addAllSeenRelays() {
        seenRelays.forEach { add($0) }
    }

    @ViewBuilder
    private func seenRelayRow(for relayURL: URL) -> some View {
        // Seen relays are a persistent registry; these actions only decide whether the relay is active in the pool.
        HStack {
            Text(relayURL.absoluteString)
            Spacer()
            if contains(relayURL) {
                ContextAwareActionChipButton(title: "Remove",
                                             systemImage: "minus.circle.fill",
                                             role: .destructive) {
                    disconnect(relayURL)
                }
            } else {
                ContextAwareActionChipButton(title: "Add",
                                             systemImage: "plus.circle.fill") {
                    add(relayURL)
                }
            }
        }
        .swipeActions(edge: .trailing) {
            if contains(relayURL) == false {
                Button(role: .destructive) {
                    relayDirectory.removeSeen(relayURL)
                } label: {
                    Label("Remove", systemImage: "trash")
                }
            } else {
                Button(role: .destructive) {
                    disconnect(relayURL)
                } label: {
                    Label("Remove", systemImage: "trash")
                }
            }
        }
    }

    private func removeAllConnectedRelays() {
        connectedRelays.forEach { disconnect($0) }
    }

    private func removeAllConnectingRelays() {
        groupedRelays.connecting.forEach { disconnect($0) }
    }

    private func removeAllDisconnectedRelays() {
        groupedRelays.disconnected.forEach { disconnect($0) }
    }

    private func disconnect(_ relay: Relay) {
        relayDirectory.record(seen: [relay.url])
        pool.remove(relay: relay)
        relayDirectory.deduplicateSeenRelayURLs()
    }

    private func disconnect(_ relayURL: URL) {
        relayDirectory.record(seen: [relayURL])
        pool.removeRelay(withURL: relayURL)
        relayDirectory.deduplicateSeenRelayURLs()
    }
    
    private func remove(relays: [Relay], at offsets: IndexSet) {
        offsets.map { relays[$0] }.forEach { disconnect($0) }
    }

    private func binding(for relayURL: URL) -> Binding<Bool> {
        Binding(
            get: { expandedRelayURLs.contains(relayURL) },
            set: { isExpanded in
                if isExpanded {
                    expandedRelayURLs.insert(relayURL)
                    relayInfoLoader.refresh(relays: relays.filter { $0.url == relayURL })
                } else {
                    expandedRelayURLs.remove(relayURL)
                }
            }
        )
    }

    private func toggleNIPFilter(_ nip: Int) {
        if activeNIPFilters.contains(nip) {
            activeNIPFilters.remove(nip)
        } else {
            activeNIPFilters.insert(nip)
        }
    }
}

private struct RelayMetadataDetailView: View {
    let info: RelayInfo
    let relayLabel: String
    let connectedRelays: [Relay]
    @ObservedObject var relayInfoLoader: RelayInfoLoader
    let onNIPSelected: (Int) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            if let name = info.name, name.isEmpty == false {
                detailRow(title: "Name", value: name)
            }
            if let description = info.description, description.isEmpty == false {
                detailRow(title: "Description", value: description)
            }
            if let software = info.software, software.isEmpty == false {
                detailRow(title: "Software", value: software)
            }
            if let version = info.version, version.isEmpty == false {
                detailRow(title: "Version", value: version)
            }
            if let contact = info.contactPubkey, contact.isEmpty == false {
                detailRow(title: "Contact Pubkey", value: contact)
            } else if let alternativeContact = info.alternativeContact, alternativeContact.isEmpty == false {
                detailRow(title: "Contact", value: alternativeContact)
            }
            if let supportedNIPs = info.supportedNIPs?.sorted(), supportedNIPs.isEmpty == false {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Supported NIPs")
                        .font(.caption2.weight(.semibold))
                        .foregroundColor(.secondary)

                    LazyVGrid(columns: [GridItem(.adaptive(minimum: 88), alignment: .leading)], alignment: .leading, spacing: 8) {
                        ForEach(supportedNIPs, id: \.self) { nip in
                            Button {
                                onNIPSelected(nip)
                            } label: {
                                Text("NIP-\(nip)")
                                    .font(.caption.weight(.semibold))
                                    .foregroundColor(.primary)
                                    .padding(.horizontal, 10)
                                    .padding(.vertical, 6)
                                    .frame(maxWidth: .infinity)
                                    .background(
                                        RoundedRectangle(cornerRadius: 999, style: .continuous)
                                            .fill(Color(.tertiarySystemFill))
                                    )
                            }
                            .buttonStyle(.plain)
                        }
                    }
                }
            }
            if let relayCountries = info.relayCountries, relayCountries.isEmpty == false {
                detailRow(title: "Countries", value: relayCountries.joined(separator: ", "))
            }
            if let languageTags = info.languageTags, languageTags.isEmpty == false {
                detailRow(title: "Languages", value: languageTags.joined(separator: ", "))
            }
            if let tags = info.tags, tags.isEmpty == false {
                detailRow(title: "Tags", value: tags.joined(separator: ", "))
            }

            if let limits = info.limitations {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Limitations")
                        .font(.caption.weight(.semibold))
                    if let maxSubscriptions = limits.maxSubscriptions {
                        detailRow(title: "Max subscriptions", value: String(maxSubscriptions))
                    }
                    if let maxFilters = limits.maxFilters {
                        detailRow(title: "Max filters", value: String(maxFilters))
                    }
                    if let maxLimit = limits.maxLimit {
                        detailRow(title: "Max limit", value: String(maxLimit))
                    }
                    if let maxMessageLength = limits.maxMessageLength {
                        detailRow(title: "Max message length", value: String(maxMessageLength))
                    }
                    if let maxEventTags = limits.maxEventTags {
                        detailRow(title: "Max event tags", value: String(maxEventTags))
                    }
                    if let authRequired = limits.isAuthenticationRequired {
                        detailRow(title: "Auth required", value: authRequired ? "Yes" : "No")
                    }
                    if let restrictedWrites = limits.isWriteRestricted {
                        detailRow(title: "Write restricted", value: restrictedWrites ? "Yes" : "No")
                    }
                }
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private func detailRow(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.caption2.weight(.semibold))
                .foregroundColor(.secondary)
            Text(value)
                .font(.caption)
                .foregroundColor(.primary)
                .textSelection(.enabled)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}

private struct RelayNIPConnectedRelaysView: View {
    let nip: Int
    let connectedRelays: [Relay]
    @ObservedObject var relayInfoLoader: RelayInfoLoader
    let onBack: () -> Void

    private var relaysSupportingNIP: [Relay] {
        connectedRelays.filter { relay in
            relayInfoLoader.relayInfo(for: relay.url)?.supportedNIPs?.contains(nip) == true
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ContextAwareListToolbar(content: {
                Text("NIP-\(nip)")
                    .font(.caption.weight(.semibold))
            }, trailing: {
                Button {
                    onBack()
                } label: {
                    Label("Clear", systemImage: "xmark.circle")
                }
                .buttonStyle(.plain)
            }, horizontalPadding: 0)

            List {
                if relaysSupportingNIP.isEmpty {
                    Text("No connected relays support NIP-\(nip).")
                        .foregroundColor(.secondary)
                } else {
                    ForEach(relaysSupportingNIP, id: \.url) { relay in
                        VStack(alignment: .leading, spacing: 8) {
                            HStack(alignment: .center, spacing: 12) {
                                VStack(alignment: .leading, spacing: 4) {
                                    Text(relay.url.absoluteString)
                                        .font(.subheadline)
                                        .textSelection(.enabled)
                                    Text("Connected")
                                        .font(.caption)
                                        .foregroundColor(.green)
                                }

                                Spacer(minLength: 12)

                                relay.statusImage
                            }

                            if let info = relayInfoLoader.relayInfo(for: relay.url) {
                                RelayMetadataCompactView(info: info)
                            }
                        }
                        .padding(.vertical, 6)
                    }
                }
            }
        }
    }
}

private struct RelayMetadataCompactView: View {
    let info: RelayInfo

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            if let name = info.name, name.isEmpty == false {
                Text(name)
                    .font(.caption.weight(.semibold))
            }
            if let description = info.description, description.isEmpty == false {
                Text(description)
                    .font(.caption)
                    .foregroundColor(.secondary)
                    .lineLimit(3)
            }
        }
    }
}
