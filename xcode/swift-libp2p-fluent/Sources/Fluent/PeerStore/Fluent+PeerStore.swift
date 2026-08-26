//===----------------------------------------------------------------------===//
//
// This source file is part of the swift-libp2p open source project
//
// Copyright (c) 2022-2025 swift-libp2p project authors
// Licensed under MIT
//
// See LICENSE for license information
// See CONTRIBUTORS for the list of swift-libp2p project authors
//
// SPDX-License-Identifier: MIT
//
//===----------------------------------------------------------------------===//
//
//  Created by Vapor
//  Modified by swift-libp2p
//

public import FluentKit
import Foundation
public import LibP2P
import LibP2PCrypto

extension Application.PeerStores {
    public var fluent: any PeerStore {
        self.fluent(nil)
    }

    public func fluent(_ db: DatabaseID?) -> any PeerStore {
        FluentPeerStore(id: db, database: self.application.db(db))
    }

    public func prepareMigrations() {
        self.application.migrations.add(PeerStoreEntry.migration)
        self.application.migrations.add(PeerStoreEntry_Multiaddr.migration)
        self.application.migrations.add(PeerStoreEntry_Protocol.migration)
        self.application.migrations.add(PeerStoreEntry_Record.migration)
        self.application.migrations.add(PeerStoreEntry_Metadata.migration)
    }
}

extension Application.PeerStores.Provider {
    public static var fluent: Self {
        .fluent(nil)
    }

    public static func fluent(_ db: DatabaseID?) -> Self {
        .init {
            $0.peerstore.use { $0.peerstore.fluent(db) }
        }
    }
}

extension FluentPeerStore {
    public enum Error: Swift.Error {
        case notFound
        case noPeerIDWithinMultiaddr
    }
}

private struct FluentPeerStore: PeerStore {
    let id: DatabaseID?
    let database: any Database

    static let maxRecordsToKeep: Int = 3

    init(id: DatabaseID?, database: any Database) {
        self.id = id
        self.database = database
    }

    /// - Warning: This can be very computationally expensive
    func all() -> EventLoopFuture<[ComprehensivePeer]> {
        let promise = database.eventLoop.makePromise(of: [ComprehensivePeer].self)
        promise.completeWithTask {
            var peers: [ComprehensivePeer] = []
            for entry in try await PeerStoreEntry.query(on: database).all() {
                peers.append(try await entry.asComprehensivePeer(on: database))
            }
            return peers
        }
        return promise.futureResult
    }

    func count() -> EventLoopFuture<Int> {
        PeerStoreEntry.query(on: database).count()
    }

    func dump(peer: PeerID) {
        print(peer)
    }

    func dumpAll() {
        print("FluentPeerStore::dumpAll -> Not Yet Implemented")
    }

    private func getPeerEntry(for peer: PeerID) async throws -> PeerStoreEntry {
        guard let pid = try await PeerStoreEntry.query(on: database).filter(\.$peer == peer.b58String).first() else {
            throw Error.notFound
        }
        return pid
    }

    private func getDatabaseID(for peer: PeerID) async throws -> PeerStoreEntry.IDValue {
        try await getPeerEntry(for: peer).requireID()
    }
}

// MARK: Address Book Protocol Conformance

extension FluentPeerStore {
    func add(address: Multiaddr, toPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let peer = try await getPeerEntry(for: peer)
            let entry = PeerStoreEntry_Multiaddr()
            entry.address = address.description
            // We keep this optional because we have a unique address contraint on our table
            // and we don't want to fail the entire method if we attempt to add a duplicate ma
            try? await peer.$multiaddrs.create(entry, on: database)
        }
        return promise.futureResult
    }

    func add(addresses: [Multiaddr], toPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        guard !addresses.isEmpty else { return (on ?? database.eventLoop).makeSucceededVoidFuture() }
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let peer = try await getPeerEntry(for: peer)
            for address in addresses {
                let entry = PeerStoreEntry_Multiaddr()
                entry.address = address.description
                // We keep this optional because we have a unique address contraint on our table
                // and we don't want to fail the entire method if we attempt to add a duplicate ma
                try? await peer.$multiaddrs.create(entry, on: database)
            }
        }
        return promise.futureResult
    }

    /// Removes a Multiaddr from an existing PeerID
    func remove(address: Multiaddr, fromPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Multiaddr.query(on: database)
                .filter(\.$peer.$id == pid)
                .filter(\.$address == address.description)
                .delete(force: true)
        }
        return promise.futureResult
    }

    /// Removes all Multiaddrs from an existing PeerID
    func removeAllAddresses(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Multiaddr.query(on: database)
                .filter(\.$peer.$id == pid)
                .delete(force: true)
        }
        return promise.futureResult

    }

    func getAddresses(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<[Multiaddr]> {
        let promise = (on ?? database.eventLoop).makePromise(of: [Multiaddr].self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            return try await PeerStoreEntry_Multiaddr.query(on: database)
                .filter(\.$peer.$id == pid)
                .all()
                .compactMap { try? Multiaddr($0.address) }
        }
        return promise.futureResult
    }

    /// Returns the ID of a peer if one is embedded in the multiaddr, otherwise returns not found
    /// - WARNING: Right now we don't bother trying to match the IP address, should we fall back to searching for the address if the id isn't embedded in the mutliaddr?
    func getPeer(byAddress address: Multiaddr, on: (any EventLoop)? = nil) -> EventLoopFuture<String> {
        if let pid = try? address.getPeerID() {
            (on ?? database.eventLoop).makeSucceededFuture(pid.b58String)
        } else {
            (on ?? database.eventLoop).makeFailedFuture(Error.notFound)
        }
    }

    func getPeerID(byAddress address: Multiaddr, on: (any EventLoop)?) -> EventLoopFuture<PeerID> {
        print("FluentPeerStore::getPeerID(byAddress:)")
        guard let pid = try? address.getPeerID() else {
            return (on ?? database.eventLoop).makeFailedFuture(
                NSError(domain: "No PeerID available for this Multiaddr", code: 0)
            )
        }
        print("FluentPeerStore::getPeerID(byAddress:) -> got pid `\(pid)`")

        return PeerStoreEntry.query(on: database)
            .filter(\.$peer == pid.b58String)
            .first()
            .flatMapThrowing { entry throws -> PeerID in
                print("FluentPeerStore::getPeerID(byAddress:) -> Found Entry `\(entry?.description ?? "nil")`")
                guard let peer = try? entry?.peerID else {
                    throw NSError(
                        domain: "Failed to recover PeerID from PeerStoreEntry \(entry?.description ?? "nil")",
                        code: 0
                    )
                }
                print("FluentPeerStore::getPeerID(byAddress:) -> Found PeerID `\(peer)`")
                return peer
            }
    }

    func getPeerInfo(
        byAddress address: Multiaddr,
        on: (any EventLoop)?
    ) -> EventLoopFuture<PeerInfo> {

        let promise = (on ?? database.eventLoop).makePromise(of: PeerInfo.self)
        promise.completeWithTask {
            guard let pid = try? address.getPeerID() else {
                throw Error.noPeerIDWithinMultiaddr
            }
            guard
                let peer = try await PeerStoreEntry.query(on: database)
                    .filter(\.$peer == pid.b58String)
                    .first()
            else {
                throw Error.notFound
            }
            return try await peer.asPeerInfo(on: database)
        }

        return promise.futureResult
    }
}

// MARK: Key / PeerID Book

extension FluentPeerStore {
    /// Adds a Key (PeerID) to our KeyBook
    func add(key: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            if let existingPeer = try? await getPeerEntry(for: key) {
                // Check to see if the new key contains more info then the previous entry before overwriting it
                if let epid = try? existingPeer.peerID {
                    if epid.type == .idOnly && key.type != .idOnly {
                        existingPeer.keypair = try Data(key.marshalPublicKey())
                    }
                } else {
                    existingPeer.keypair = try Data(key.marshalPublicKey())
                }
                try await existingPeer.update(on: database)
            } else {
                // Create a new peer entry
                let newPeer = PeerStoreEntry(peerID: key)
                try await newPeer.create(on: database)
                let metadata = PeerStoreEntry_Metadata()
                metadata.key = MetadataBook.Keys.Discovered.rawValue
                metadata.value = "\(Date().timeIntervalSince1970)"
                try? await newPeer.$metadata.create(metadata, on: database)

                // TODO: Trim database if neccessary
            }
        }
        return promise.futureResult
    }

    /// Removes a Key (PeerID) from our PeerStore
    func remove(key: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            guard let pid = try? await getDatabaseID(for: key) else { return }
            // If we're running mongodb, we have to manually delete our child references
            if database.isMongoDB {
                try? await PeerStoreEntry_Record.query(on: database)
                    .filter(\.$peer.$id == pid)
                    .delete(force: true)
                try? await PeerStoreEntry_Multiaddr.query(on: database)
                    .filter(\.$peer.$id == pid)
                    .delete(force: true)
                try? await PeerStoreEntry_Protocol.query(on: database)
                    .filter(\.$peer.$id == pid)
                    .delete(force: true)
                try? await PeerStoreEntry_Metadata.query(on: database)
                    .filter(\.$peer.$id == pid)
                    .delete(force: true)
            }
            try await PeerStoreEntry.query(on: database)
                .filter(\.$peer == key.b58String)
                .delete(force: true)
        }
        return promise.futureResult
    }

    /// Removes all Keys (PeerIDs) from our PeerStore
    func removeAllKeys(on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            try await PeerStoreEntry.query(on: database)
                .delete(force: true)
            // If we're running mongodb, we have to manually delete our child references
            if database.isMongoDB {
                try? await PeerStoreEntry_Record.query(on: database)
                    .delete(force: true)
                try? await PeerStoreEntry_Multiaddr.query(on: database)
                    .delete(force: true)
                try? await PeerStoreEntry_Protocol.query(on: database)
                    .delete(force: true)
                try? await PeerStoreEntry_Metadata.query(on: database)
                    .delete(force: true)
            }
        }
        return promise.futureResult
    }

    func getKey(forPeer id: String, on: (any EventLoop)? = nil) -> EventLoopFuture<PeerID> {
        let promise = (on ?? database.eventLoop).makePromise(of: PeerID.self)
        promise.completeWithTask {
            guard
                let peer = try await PeerStoreEntry.query(on: database)
                    .filter(\.$peer == id)
                    .first()
            else {
                throw Error.notFound
            }
            return try peer.peerID
        }
        return promise.futureResult
    }
}

// MARK: Protocol Book

extension FluentPeerStore {
    /// Adds a Protocol to an existing PeerID
    func add(protocol proto: SemVerProtocol, toPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getPeerEntry(for: peer)
            let newProtocol = PeerStoreEntry_Protocol()
            newProtocol.protocol = proto.stringValue
            // Duplicate values will throw a unique constraint violation here, do we want to fail?
            try? await pid.$protocols.create(newProtocol, on: database)
        }
        return promise.futureResult
    }

    func add(
        protocols protos: [SemVerProtocol],
        toPeer peer: PeerID,
        on: (any EventLoop)? = nil
    ) -> EventLoopFuture<Void> {
        guard !protos.isEmpty else { return (on ?? database.eventLoop).makeSucceededVoidFuture() }
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getPeerEntry(for: peer)
            for proto in protos {
                let newProtocol = PeerStoreEntry_Protocol()
                newProtocol.protocol = proto.stringValue
                // Duplicate values will throw a unique constraint violation here, do we want to fail?
                try? await pid.$protocols.create(newProtocol, on: database)
            }
        }
        return promise.futureResult
    }

    /// Removes a Protocol from an existing PeerID
    func remove(
        protocol proto: SemVerProtocol,
        fromPeer peer: PeerID,
        on: (any EventLoop)? = nil
    ) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Protocol.query(on: database)
                .filter(\.$peer.$id == pid)
                .filter(\.$protocol == proto.stringValue)
                .delete(force: true)
        }
        return promise.futureResult
    }

    func remove(
        protocols: [SemVerProtocol],
        fromPeer peer: PeerID,
        on: (any EventLoop)?
    ) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            let protos = protocols.map { $0.stringValue }
            let matches = try await PeerStoreEntry_Protocol.query(on: database)
                .filter(\.$peer.$id == pid)
                .filter(\.$protocol ~~ protos)
                .all()

            if matches.count > 0 {
                print("Deleting \(matches.count) protocols from \(peer)")
                try await matches.delete(on: database)
            }

            return
        }
        return promise.futureResult
    }

    func removeAllProtocols(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Protocol.query(on: database)
                .filter(\.$peer.$id == pid)
                .delete(force: true)
        }
        return promise.futureResult
    }

    func getProtocols(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<[SemVerProtocol]> {
        let promise = (on ?? database.eventLoop).makePromise(of: [SemVerProtocol].self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            return try await PeerStoreEntry_Protocol.query(on: database)
                .filter(\.$peer.$id == pid)
                .all()
                .compactMap {
                    SemVerProtocol($0.protocol)
                }
        }
        return promise.futureResult
    }

    func getPeers(supportingProtocol proto: SemVerProtocol, on: (any EventLoop)? = nil) -> EventLoopFuture<[String]> {
        let promise = (on ?? database.eventLoop).makePromise(of: [String].self)
        promise.completeWithTask {
            try await PeerStoreEntry_Protocol.query(on: database)
                .filter(\.$protocol == proto.stringValue)
                .with(\.$peer)
                .all()
                .map { $0.peer.peer }
        }
        return promise.futureResult
    }

    func getPeerIDs(supportingProtocol proto: SemVerProtocol, on: (any EventLoop)? = nil) -> EventLoopFuture<[PeerID]> {
        let promise = (on ?? database.eventLoop).makePromise(of: [PeerID].self)
        promise.completeWithTask {
            try await PeerStoreEntry_Protocol.query(on: database)
                .filter(\.$protocol == proto.stringValue)
                .with(\.$peer)
                .all()
                .compactMap { try? $0.peer.peerID }
        }
        return promise.futureResult
    }
}

// MARK: Record Book

extension FluentPeerStore {
    func add(record: PeerRecord, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            var peer: PeerStoreEntry! = nil
            var shouldTrim: Bool = false
            if let existingPeer = try? await getPeerEntry(for: record.peerID) {
                peer = existingPeer
                shouldTrim = true
            } else {
                //Create a new peer entry
                let newPeer = PeerStoreEntry(peerID: record.peerID)
                try await newPeer.create(on: database)
                peer = newPeer
            }
            let rec = PeerStoreEntry_Record()
            rec.record = try record.marshal().asString(base: .base64Pad, withMultibasePrefix: false)
            rec.sequence = Int64(bitPattern: record.sequenceNumber)
            try await peer.$records.create(rec, on: database)

            if shouldTrim {
                try await trimRecords(forPeer: record.peerID, on: on).get()
            }
        }
        return promise.futureResult
    }

    func getRecords(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<[PeerRecord]> {
        let promise = (on ?? database.eventLoop).makePromise(of: [PeerRecord].self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            let matches = try await PeerStoreEntry_Record.query(on: database)
                .filter(\.$peer.$id == pid)
                .all()

            return try matches.compactMap {
                guard let asData = Data(base64Encoded: $0.record) else { return nil }
                return try PeerRecord(marshaledData: asData)
            }
        }
        return promise.futureResult
    }

    /// Returns the most recent (highest‑sequence) record for the given peer, if any.
    func getMostRecentRecord(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<PeerRecord?> {
        let promise = (on ?? database.eventLoop).makePromise(of: PeerRecord?.self)
        promise.completeWithTask {
            let peerStoreID = try await getDatabaseID(for: peer)
            if let recordRow = try await PeerStoreEntry_Record.query(on: database)
                .filter(\.$peer.$id == peerStoreID)
                .sort(\.$sequence, .descending)
                .first()
            {
                guard let asData = Data(base64Encoded: recordRow.record) else { return nil }
                return try PeerRecord(marshaledData: asData)
            } else {
                return nil
            }
        }
        return promise.futureResult
    }

    func trimRecords(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await trimRecords(forPeerID: pid)
        }
        return promise.futureResult
    }

    private func trimRecords(forPeerID pid: PeerStoreEntry.IDValue, on: (any EventLoop)? = nil) async throws {
        let matches = try await PeerStoreEntry_Record.query(on: database)
            .filter(\.$peer.$id == pid)
            .sort(\.$sequence, .descending)
            .all()
        if matches.count <= FluentPeerStore.maxRecordsToKeep { return }
        try await matches.dropFirst(FluentPeerStore.maxRecordsToKeep).delete(on: database)
    }

    func trimAllRecords() -> EventLoopFuture<Void> {
        let promise = database.eventLoop.makePromise(of: Void.self)
        promise.completeWithTask {
            for peer in try await PeerStoreEntry.query(on: database).all() {
                try? await trimRecords(forPeerID: peer.requireID())
            }
        }
        return promise.futureResult
    }

    func removeRecords(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Record.query(on: database)
                .filter(\.$peer.$id == pid)
                .delete(force: true)
        }
        return promise.futureResult
    }
}

// MARK: Metadata Book

extension FluentPeerStore {
    func add(
        metaKey key: String,
        data: [UInt8],
        toPeer peer: PeerID,
        on: (any EventLoop)? = nil
    ) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let peer = try await getPeerEntry(for: peer)

            let meta = PeerStoreEntry_Metadata()
            meta.key = key
            meta.value = String(data: Data(data), encoding: .utf8) ?? ""

            try await peer.$metadata.create(meta, on: database)
        }
        return promise.futureResult
    }

    func add(
        metaKey key: MetadataBook.Keys,
        data: [UInt8],
        toPeer peer: PeerID,
        on: (any EventLoop)? = nil
    ) -> EventLoopFuture<Void> {
        self.add(metaKey: key.rawValue, data: data, toPeer: peer, on: on)
    }

    func remove(metaKey key: String, fromPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Metadata.query(on: database)
                .filter(\.$peer.$id == pid)
                .filter(\.$key == key)
                .delete(force: true)
        }
        return promise.futureResult
    }

    func removeAllMetadata(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Void> {
        let promise = (on ?? database.eventLoop).makePromise(of: Void.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            try await PeerStoreEntry_Metadata.query(on: database)
                .filter(\.$peer.$id == pid)
                .delete(force: true)
        }
        return promise.futureResult
    }

    func getMetadata(forPeer peer: PeerID, on: (any EventLoop)? = nil) -> EventLoopFuture<Metadata> {
        let promise = (on ?? database.eventLoop).makePromise(of: Metadata.self)
        promise.completeWithTask {
            let pid = try await getDatabaseID(for: peer)
            let metas = try await PeerStoreEntry_Metadata.query(on: database)
                .filter(\.$peer.$id == pid)
                .all()
            var metadata: [String: [UInt8]] = [:]
            for meta in metas {
                metadata[meta.key] = [UInt8](Data(meta.value.utf8))
            }
            return metadata
        }
        return promise.futureResult
    }
}

extension Database {
    fileprivate var isMongoDB: Bool {
        "\(self.configuration)".hasPrefix("FluentMongoConfiguration")
    }
}
