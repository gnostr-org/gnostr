//
//  Timeline.swift
//  damus
//
//  Created by William Casarin on 2022-05-09.
//

import Foundation


class SearchModel: ObservableObject {
    @Published var events: [NostrEvent] = []
    @Published var loading: Bool = false
    @Published var channel_name: String? = nil
    
    let pool: RelayPool
    var search: NostrFilter
    private(set) var sub_id = UUID().description
    let limit: UInt32 = 500
    private let requested_kinds: [Int]?
    private var pending_events: [NostrEvent] = []
    private var pending_offset = 0
    private var seen_ids: Set<String> = []
    private var batch_task: Task<Void, Never>?
    private var generation = 0
    private var subscribed = false
    private var received_eose = false
    
    init(pool: RelayPool, search: NostrFilter) {
        self.pool = pool
        self.search = search
        self.requested_kinds = search.kinds
    }
    
    func subscribe(nip34_only: Bool = false, selected_kinds: Set<Int>? = nil) {
        if subscribed {
            unsubscribe()
        } else {
            cancel_pending_events()
        }
        sub_id = UUID().description
        search.limit = self.limit
        search.kinds = NostrKind.search_kinds(nip34_only: nip34_only, selected: selected_kinds,
                                             requested: requested_kinds)
        events.removeAll()
        guard search.kinds?.isEmpty != true else {
            unsubscribe()
            return
        }

        //likes_filter.ids = ref_events.referenced_ids!

        print("subscribing to search '\(search)' with sub_id \(sub_id)")
        pool.register_handler(sub_id: sub_id, handler: handle_event)
        subscribed = true
        loading = true
        pool.send(.subscribe(.init(filters: [search], sub_id: sub_id)))
    }
    
    func unsubscribe() {
        subscribed = false
        cancel_pending_events()
        self.pool.unsubscribe(sub_id: sub_id)
        loading = false
        print("unsubscribing from search '\(search)' with sub_id \(sub_id)")
    }
    
    func add_event(_ ev: NostrEvent) {
        guard subscribed, seen_ids.insert(ev.id).inserted else { return }
        pending_events.append(ev)
        schedule_batch()
    }

    private func cancel_pending_events() {
        generation += 1
        batch_task?.cancel()
        batch_task = nil
        pending_events.removeAll()
        pending_offset = 0
        seen_ids.removeAll()
        received_eose = false
    }

    private func schedule_batch() {
        guard batch_task == nil, !pending_events.isEmpty else { return }
        let generation = self.generation
        batch_task = Task { @MainActor [weak self] in
            do {
                try await Task.sleep(nanoseconds: 30_000_000)
            } catch is CancellationError {
                return
            } catch {
                assertionFailure("Search batch delay failed: \(error)")
                return
            }
            guard let self = self, self.generation == generation else { return }
            let end = min(self.pending_offset + 64, self.pending_events.count)
            let batch = Array(self.pending_events[self.pending_offset..<end])
            self.pending_offset = end
            if end == self.pending_events.count {
                self.pending_events.removeAll(keepingCapacity: true)
                self.pending_offset = 0
            }
            let current = self.events
            let order = current.enumerated().map {
                SearchEventOrder(index: $0.offset, created_at: $0.element.created_at)
            }
            // Only value snapshots cross threads; NostrEvent has mutable lazy caches.
            let snapshots = batch.map(SearchEventSnapshot.init)
            let filter = self.search
            let processing = Task.detached(priority: .userInitiated) {
                var merged = order
                for (index, event) in snapshots.enumerated() {
                    guard !Task.isCancelled else { return [Int]() }
                    if event.content.count <= 32000 &&
                        event_matches_filter(kind: event.kind, content: event.content,
                                             tags: event.tags, filter: filter) {
                        merged.append(SearchEventOrder(index: order.count + index,
                                                       created_at: event.created_at))
                    }
                }
                return merged.sorted {
                    $0.created_at == $1.created_at ? $0.index < $1.index : $0.created_at > $1.created_at
                }.map { $0.index }
            }
            let indices = await withTaskCancellationHandler(operation: {
                await processing.value
            }, onCancel: {
                processing.cancel()
            })
            guard !Task.isCancelled, self.generation == generation else { return }
            if indices.count != current.count {
                let combined = current + batch
                self.events = indices.map { combined[$0] }
            }
            self.batch_task = nil
            if self.pending_events.isEmpty && self.received_eose {
                self.loading = false
            }
            self.schedule_batch()
        }
    }
    
    func handle_channel_create(_ ev: NostrEvent) {
        self.channel_name = ev.content
        return
    }
    
    func handle_channel_meta(_ ev: NostrEvent) {
        self.channel_name = ev.content
        return
    }
    
    func handle_event(relay_id: String, ev: NostrConnectionEvent) {
        guard subscribed else { return }
        let (sid, done) = handle_subid_event(pool: pool, relay_id: relay_id, ev: ev) { sub_id, ev in
            guard sub_id == self.sub_id else { return }
            if ev.is_textlike || NostrKind.nip34_kinds.contains(ev.kind) {
                self.add_event(ev)
            } else if ev.known_kind == .channel_create {
                handle_channel_create(ev)
            } else if ev.known_kind == .channel_meta {
                handle_channel_meta(ev)
            }
        }
        
        if done && sid == sub_id {
            received_eose = true
            if batch_task == nil && pending_events.isEmpty {
                loading = false
            }
        }
    }
}

private struct SearchEventOrder: Sendable {
    let index: Int
    let created_at: Int64
}

private struct SearchEventSnapshot: Sendable {
    let created_at: Int64
    let kind: Int
    let content: String
    let tags: [[String]]

    init(_ event: NostrEvent) {
        created_at = event.created_at
        kind = event.kind
        content = event.content
        tags = event.tags
    }
}

func event_matches_hashtag(_ ev: NostrEvent, hashtags: [String]) -> Bool {
    return event_matches_hashtag(tags: ev.tags, hashtags: hashtags)
}

private func event_matches_hashtag(tags: [[String]], hashtags: [String]) -> Bool {
    for tag in tags {
        if tag_is_hashtag(tag) && hashtags.contains(tag[1]) {
            return true
        }
    }
    return false
}

func event_matches_filter(_ ev: NostrEvent, filter: NostrFilter) -> Bool {
    return event_matches_filter(kind: ev.kind, content: ev.content, tags: ev.tags, filter: filter)
}

private func event_matches_filter(kind: Int, content: String, tags: [[String]], filter: NostrFilter) -> Bool {
    if let kinds = filter.kinds, !kinds.contains(kind) {
        return false
    }
    if let search = filter.search, !content.localizedCaseInsensitiveContains(search) {
        return false
    }
    if let hashtags = filter.hashtag {
        return event_matches_hashtag(tags: tags, hashtags: hashtags)
    }
    return true
}

func handle_subid_event(pool: RelayPool, relay_id: String, ev: NostrConnectionEvent, handle: (String, NostrEvent) -> ()) -> (String?, Bool) {
    switch ev {
    case .ws_event:
        return (nil, false)
        
    case .nostr_event(let res):
        switch res {
        case .event(let ev_subid, let ev):
            handle(ev_subid, ev)
            return (ev_subid, false)

        case .notice(let note):
            if note.contains("Too many subscription filters") {
                // TODO: resend filters?
                pool.reconnect(to: [relay_id])
            }
            return (nil, false)
            
        case .eose(let subid):
            return (subid, true)
        }
    }
}
