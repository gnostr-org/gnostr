//
//  RelayPool.swift
//  damus
//
//  Created by William Casarin on 2022-04-11.
//

import Foundation
import Combine

struct SubscriptionId: Identifiable, CustomStringConvertible {
    let id: String

    var description: String {
        id
    }
}

struct RelayId: Identifiable, CustomStringConvertible {
    let id: String

    var description: String {
        id
    }
}

struct RelayHandler {
    let sub_id: String
    let callback: (String, NostrConnectionEvent) -> ()
}

class RelayPool: ObservableObject {
    @Published var relays: [Relay] = []
    @Published private(set) var suggested_relays: [RelayDescriptor] = []
    @Published private(set) var ignored_relays: Set<String>
    @Published var auto_subscribe_nip34: Bool {
        didSet {
            defaults?.set(auto_subscribe_nip34, forKey: "auto_subscribe_nip34")
            if auto_subscribe_nip34 {
                for (url, information) in information_cache {
                    subscribe_to_nip34_relay(url, information: information)
                }
            }
        }
    }
    private let defaults: UserDefaults?
    private var removed_relays: Set<String>
    private var discovery_started = false
    private var information_cache: [URL: RelayInformation] = [:]
    private var information_tasks: [URL: Task<RelayInformation, Error>] = [:]
    private let information_fetcher: (URL) async throws -> RelayInformation
    private let relay_connector: (Relay) -> Void
    var handlers: [RelayHandler] = []

    init(defaults: UserDefaults? = .standard,
         information_fetcher: @escaping (URL) async throws -> RelayInformation = fetch_relay_information,
         relay_connector: @escaping (Relay) -> Void = { $0.connection.connect() }) {
        self.defaults = defaults
        self.information_fetcher = information_fetcher
        self.relay_connector = relay_connector
        self.ignored_relays = Set((defaults?.stringArray(forKey: "ignored_relays") ?? [])
            .compactMap { normalized_relay_url($0)?.absoluteString })
        self.removed_relays = Set((defaults?.stringArray(forKey: "removed_relays") ?? [])
            .compactMap { normalized_relay_url($0)?.absoluteString })
        self.auto_subscribe_nip34 = defaults?.object(forKey: "auto_subscribe_nip34") as? Bool ?? true
    }

    var recommended_relays: [RelayDescriptor] {
        return available_recommendations(show_ignored: false)
    }

    func available_recommendations(show_ignored: Bool) -> [RelayDescriptor] {
        let configured = Set(descriptors.compactMap { normalized_relay_url($0.url.absoluteString) })
        var seen = configured
        let bootstrap = BOOTSTRAP_RELAYS.compactMap { value -> RelayDescriptor? in
            guard let url = normalized_relay_url(value) else { return nil }
            return RelayDescriptor(url: url, info: .rw)
        }
        let ignored = ignored_relays.sorted().compactMap { value -> RelayDescriptor? in
            guard let url = normalized_relay_url(value) else { return nil }
            return RelayDescriptor(url: url, info: .rw)
        }
        return (suggested_relays + bootstrap + ignored).filter {
            (show_ignored || !is_relay_ignored($0.url)) && seen.insert($0.url).inserted
        }
    }

    func is_relay_ignored(_ url: URL) -> Bool {
        guard let normalized = normalized_relay_url(url.absoluteString) else { return false }
        return ignored_relays.contains(normalized.absoluteString)
    }

    func set_relay_ignored(_ relay: RelayDescriptor, ignored: Bool) {
        guard let url = normalized_relay_url(relay.url.absoluteString) else { return }
        remember_relay(relay)
        if ignored {
            ignored_relays.insert(url.absoluteString)
            if let configured = relays.first(where: {
                normalized_relay_url($0.descriptor.url.absoluteString) == url
            }) {
                remove_relay(configured.id)
            }
        } else {
            ignored_relays.remove(url.absoluteString)
            allow_relay(url)
            discover_relay(relay)
        }
        defaults?.set(ignored_relays.sorted(), forKey: "ignored_relays")
    }

    func is_relay_removed(_ url: URL) -> Bool {
        guard let normalized = normalized_relay_url(url.absoluteString) else { return false }
        return removed_relays.contains(normalized.absoluteString)
    }

    func allow_relay(_ url: URL) {
        guard let normalized = normalized_relay_url(url.absoluteString) else { return }
        removed_relays.remove(normalized.absoluteString)
        ignored_relays.remove(normalized.absoluteString)
        defaults?.set(removed_relays.sorted(), forKey: "removed_relays")
        defaults?.set(ignored_relays.sorted(), forKey: "ignored_relays")
    }

    @MainActor
    func load_relay_information(_ relayURL: URL) async throws -> RelayInformation {
        guard let url = normalized_relay_url(relayURL.absoluteString) else {
            throw URLError(.badURL)
        }
        if let information = information_cache[url] {
            subscribe_to_nip34_relay(url, information: information)
            return information
        }
        if let task = information_tasks[url] {
            return try await task.value
        }
        let fetcher = information_fetcher
        let task = Task { try await fetcher(url) }
        information_tasks[url] = task
        do {
            let information = try await task.value
            information_tasks[url] = nil
            information_cache[url] = information
            subscribe_to_nip34_relay(url, information: information)
            return information
        } catch {
            information_tasks[url] = nil
            throw error
        }
    }

    private func discover_relay(_ relay: RelayDescriptor) {
        guard discovery_started, !is_relay_ignored(relay.url) else { return }
        Task { @MainActor [weak self] in
            do {
                _ = try await self?.load_relay_information(relay.url)
            } catch {
                print("NIP-11 fetch failed for \(relay.url): \(error)")
            }
        }
    }

    private func subscribe_to_nip34_relay(_ url: URL, information: RelayInformation) {
        guard auto_subscribe_nip34, information.supportsNIP34,
              !is_relay_ignored(url), !is_relay_removed(url),
              !relays.contains(where: { normalized_relay_url($0.descriptor.url.absoluteString) == url }) else {
            return
        }
        let info = suggested_relays.first(where: { $0.url == url })?.info ?? .rw
        do {
            try add_relay(url, info: info)
            if let relay = get_relay(url.absoluteString) {
                relay_connector(relay)
            }
            notify(.relays_changed, ())
        } catch {
            print("Failed to auto-subscribe to NIP-34 relay \(url): \(error)")
        }
    }

    private func remember_relay(_ relay: RelayDescriptor) {
        guard let url = normalized_relay_url(relay.url.absoluteString) else { return }
        if let index = suggested_relays.firstIndex(where: { $0.url == url }) {
            let previous = suggested_relays[index]
            let info = RelayInfo(read: previous.info.read || relay.info.read,
                                 write: previous.info.write || relay.info.write)
            if info.read != previous.info.read || info.write != previous.info.write {
                suggested_relays[index] = RelayDescriptor(url: url, info: info)
            }
        } else {
            suggested_relays.append(RelayDescriptor(url: url, info: relay.info))
        }
    }

    func collect_relay_recommendations(_ event: NostrEvent) {
        let recommendations = relay_recommendations(event)
        guard !recommendations.isEmpty else { return }
        guard event.pubkey.count == 64, event.sig.count == 128,
              validate_event(ev: event) == .ok else {
            print("Ignoring relay recommendations from invalid event \(event.id)")
            return
        }
        for recommendation in recommendations {
            remember_relay(recommendation)
            discover_relay(recommendation)
        }
    }

    var descriptors: [RelayDescriptor] {
        relays.map { $0.descriptor }
    }
    
    var num_connecting: Int {
        return relays.reduce(0) { n, r in n + (r.connection.isConnecting ? 1 : 0) }
    }

    func remove_handler(sub_id: String) {
        self.handlers = handlers.filter { $0.sub_id != sub_id }
        print("removing \(sub_id) handler, current: \(handlers.count)")
    }

    func register_handler(sub_id: String, handler: @escaping (String, NostrConnectionEvent) -> ()) {
        for handler in handlers {
            // don't add duplicate handlers
            if handler.sub_id == sub_id {
                return
            }
        }
        self.handlers.append(RelayHandler(sub_id: sub_id, callback: handler))
        print("registering \(sub_id) handler, current: \(self.handlers.count)")
    }
    
    func remove_relay(_ relay_id: String) {
        var i: Int = 0
        
        self.disconnect(to: [relay_id])
        
        for relay in relays {
            if relay.id == relay_id {
                remember_relay(relay.descriptor)
                if let url = normalized_relay_url(relay_id) {
                    removed_relays.insert(url.absoluteString)
                    defaults?.set(removed_relays.sorted(), forKey: "removed_relays")
                }
                relays.remove(at: i)
                notify(.relays_changed, ())
                break
            }
            
            i += 1
        }
    }
    
    func add_relay(_ url: URL, info: RelayInfo) throws {
        let relay_id = get_relay_id(url)
        if get_relay(relay_id) != nil {
            throw RelayError.RelayAlreadyExists
        }
        let conn = RelayConnection(url: url) { event in
            self.handle_event(relay_id: relay_id, event: event)
        }
        let descriptor = RelayDescriptor(url: url, info: info)
        let relay = Relay(descriptor: descriptor, connection: conn)
        self.relays.append(relay)
        discover_relay(descriptor)
    }
    
    /// This is used to retry dead connections
    func connect_to_disconnected() {
        for relay in relays {
            if is_relay_ignored(relay.descriptor.url) || is_relay_removed(relay.descriptor.url) {
                continue
            }
            let c = relay.connection
            
            let is_connecting = c.isReconnecting || c.isConnecting
            
            if is_connecting && (Date.now.timeIntervalSince1970 - c.last_connection_attempt) > 5 {
                print("stale connection detected (\(relay.descriptor.url.absoluteString)). retrying...")
                relay.connection.connect(force: true)
            } else if relay.is_broken || is_connecting || c.isConnected {
                continue
            } else {
                relay.connection.reconnect()
            }
            
        }
    }
    
    func reconnect(to: [String]? = nil) {
        let relays = to.map{ get_relays($0) } ?? self.relays
        for relay in relays {
            if is_relay_ignored(relay.descriptor.url) || is_relay_removed(relay.descriptor.url) {
                continue
            }
            // don't try to reconnect to broken relays
            relay.connection.reconnect()
        }
    }
    
    func mark_broken(_ relay_id: String) {
        for relay in relays {
            relay.mark_broken()
        }
    }

    func connect(to: [String]? = nil) {
        if !discovery_started {
            discovery_started = true
            for descriptor in descriptors + recommended_relays {
                discover_relay(descriptor)
            }
        }
        let relays = to.map{ get_relays($0) } ?? self.relays
        for relay in relays {
            if !is_relay_ignored(relay.descriptor.url) && !is_relay_removed(relay.descriptor.url) {
                relay_connector(relay)
            }
        }
    }

    func disconnect(to: [String]? = nil) {
        let relays = to.map{ get_relays($0) } ?? self.relays
        for relay in relays {
            relay.connection.disconnect()
        }
    }
    
    func unsubscribe(sub_id: String, to: [String]? = nil) {
        if to == nil {
            self.remove_handler(sub_id: sub_id)
        }
        self.send(.unsubscribe(sub_id), to: to)
    }
    
    func subscribe(sub_id: String, filters: [NostrFilter], handler: @escaping (String, NostrConnectionEvent) -> ()) {
        register_handler(sub_id: sub_id, handler: handler)
        send(.subscribe(.init(filters: filters, sub_id: sub_id)))
    }
    
    func subscribe_to(sub_id: String, filters: [NostrFilter], to: [String]?, handler: @escaping (String, NostrConnectionEvent) -> ()) {
        register_handler(sub_id: sub_id, handler: handler)
        send(.subscribe(.init(filters: filters, sub_id: sub_id)), to: to)
    }
    
    func send(_ req: NostrRequest, to: [String]? = nil) {
        let relays = to.map{ get_relays($0) } ?? self.relays

        for relay in relays {
            if relay.connection.isConnected {
                relay.connection.send(req)
            }
        }
    }

    func get_relays(_ ids: [String]) -> [Relay] {
        var relays: [Relay] = []

        for id in ids {
            if let relay = get_relay(id) {
                relays.append(relay)
            }
        }

        return relays
    }

    func get_relay(_ id: String) -> Relay? {
        for relay in relays {
            if relay.id == id {
                return relay
            }
        }

        return nil
    }
    
    func record_last_pong(relay_id: String, event: NostrConnectionEvent) {
        if case .ws_event(let ws_event) = event {
            if case .pong = ws_event {
                for relay in relays {
                    if relay.id == relay_id {
                        relay.last_pong = UInt32(Date.now.timeIntervalSince1970)
                        return
                    }
                }
            }
        }
    }
    
    func handle_event(relay_id: String, event: NostrConnectionEvent) {
        record_last_pong(relay_id: relay_id, event: event)
        if case .nostr_event(.event(_, let ev)) = event {
            collect_relay_recommendations(ev)
        }
        
        // handle reconnect logic, etc?
        for handler in handlers {
            handler.callback(relay_id, event)
        }
    }
}

func add_rw_relay(_ pool: RelayPool, _ url: String) {
    let url_ = URL(string: url)!
    guard !pool.is_relay_ignored(url_), !pool.is_relay_removed(url_) else { return }
    try? pool.add_relay(url_, info: RelayInfo.rw)
}
