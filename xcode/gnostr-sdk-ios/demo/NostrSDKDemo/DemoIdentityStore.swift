//
//  DemoIdentityStore.swift
//  NostrSDKDemo
//
//  Created by Copilot on 6/8/26.
//

import Combine
import GnostrSDK
import Foundation

final class DemoIdentityStore: ObservableObject {
    @Published var privateKeyInput: String = "" {
        didSet {
            refreshIdentity()
        }
    }

    @Published private(set) var publicKeyHex: String?
    @Published private(set) var followedPubkeys: [String] = []

    private var relayPool: RelayPool?
    private var followListSubscriptionId: String?
    private var followListCancellable: AnyCancellable?
    private var trackedFollowListCreatedAt: Int64 = 0
    private var trackedPublicKeyHex: String?

    func attach(relayPool: RelayPool) {
        guard self.relayPool !== relayPool else { return }
        self.relayPool = relayPool
        refreshIdentity()
    }

    private func refreshIdentity() {
        let trimmed = privateKeyInput.trimmingCharacters(in: .whitespacesAndNewlines)
        guard let privateKey = PrivateKey(nsec: trimmed) ?? PrivateKey(hex: trimmed) else {
            stopFollowListTracking()
            publicKeyHex = nil
            trackedPublicKeyHex = nil
            return
        }

        let currentPublicKeyHex = Keypair(privateKey: privateKey)?.publicKey.hex
        publicKeyHex = currentPublicKeyHex

        guard currentPublicKeyHex != trackedPublicKeyHex || followListSubscriptionId == nil else {
            return
        }

        trackedPublicKeyHex = currentPublicKeyHex
        subscribeForFollowList(publicKeyHex: currentPublicKeyHex)
    }

    private func subscribeForFollowList(publicKeyHex: String?) {
        stopFollowListTracking()

        guard let relayPool, let publicKeyHex, let filter = Filter(authors: [publicKeyHex], kinds: [EventKind.followList.rawValue]) else {
            return
        }

        followListCancellable = relayPool.events
            .receive(on: DispatchQueue.main)
            .sink { [weak self] relayEvent in
                guard let self,
                      relayEvent.subscriptionId == self.followListSubscriptionId,
                      let followListEvent = relayEvent.event as? FollowListEvent,
                      followListEvent.pubkey == publicKeyHex else {
                    return
                }

                guard followListEvent.createdAt >= self.trackedFollowListCreatedAt else {
                    return
                }

                self.trackedFollowListCreatedAt = followListEvent.createdAt
                self.followedPubkeys = Self.uniquePubkeys(followListEvent.followedPubkeys)
            }

        followListSubscriptionId = relayPool.subscribe(with: filter)
    }

    private func stopFollowListTracking() {
        if let followListSubscriptionId {
            relayPool?.closeSubscription(with: followListSubscriptionId)
        }
        followListSubscriptionId = nil
        followListCancellable?.cancel()
        followListCancellable = nil
        trackedFollowListCreatedAt = 0
        followedPubkeys = []
    }

    private static func uniquePubkeys(_ pubkeys: [String]) -> [String] {
        var seen = Set<String>()
        return pubkeys.filter { seen.insert($0).inserted }
    }
}
