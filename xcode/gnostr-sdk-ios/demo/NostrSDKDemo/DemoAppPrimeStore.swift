//
//  DemoAppPrimeStore.swift
//  NostrSDKDemo
//
//  Created by Copilot on 6/9/26.
//

import Combine
import Foundation
import GnostrSDK

final class DemoAppPrimeStore: ObservableObject {
    @Published private(set) var repositoryEventByRepoIDAndKind: [String: [Int: NostrEvent]] = [:]
    @Published private(set) var seenRepositoryURLs: Set<URL> = []

    private var relayPool: RelayPool?
    private var subscriptionId: String?
    private var eventsCancellable: AnyCancellable?

    private let primeKinds = [30617, 30618, 1617, 1621, 1630, 1631, 1632, 1633]

    func attach(relayPool: RelayPool) {
        guard self.relayPool !== relayPool else { return }
        self.relayPool = relayPool
        // Prime repository events off the main actor so relay bootstrap does not block the UI.
        Task.detached(priority: .background) { [weak self] in
            await self?.primeRepositoryEvents()
        }
    }

    func record(event: NostrEvent) {
        recordSeenRepositories(from: event)

        guard let repoID = repoID(for: event) else { return }

        var eventsByKind = repositoryEventByRepoIDAndKind[repoID] ?? [:]
        if eventsByKind[event.kind.rawValue]?.createdAt ?? 0 <= event.createdAt {
            eventsByKind[event.kind.rawValue] = event
            repositoryEventByRepoIDAndKind[repoID] = eventsByKind
        }
    }

    private func primeRepositoryEvents() async {
        if let subscriptionId {
            relayPool?.closeSubscription(with: subscriptionId)
        }

        guard let relayPool, let filter = Filter(kinds: primeKinds) else {
            subscriptionId = nil
            return
        }

        eventsCancellable = relayPool.events
            .receive(on: DispatchQueue.main)
            .sink { [weak self] relayEvent in
                guard let self, relayEvent.subscriptionId == self.subscriptionId else { return }
                self.record(event: relayEvent.event)
            }

        subscriptionId = relayPool.subscribe(with: filter)
    }

    private func repoID(for event: NostrEvent) -> String? {
        event.tags.first(where: { $0.name == "d" })?.value
    }

    private func recordSeenRepositories(from event: NostrEvent) {
        seenRepositoryURLs.formUnion(event.repositoryCloneURLs)
    }
}
