import LibP2P
import LibP2PTesting
import Testing
import FluentSQLiteDriver

@testable import Fluent
@testable import App

@Suite("App Tests", .serialized)
struct AppTests {

    private func configure(_ app: Application) async throws {
        // Set our log level
        app.logger.logLevel = .info

        // Configure the modules to be used
        app.databases.use(.sqlite(.memory), as: .init(string: "test"), isDefault: true)
        app.peerstore.use(.fluent)
        app.peerstore.prepareMigrations()

        try await app.autoMigrate()
        try await app.resetDatabase()

        // Configure our peerstore to use fluent
    }
    
    // This is an example of how you can test various aspects of your libp2p app
    @Test func testPeerStoreMigrations() async throws {
        try await withApp(configure: configure) { app in
            #expect(app.environment == .testing)
        }
    }
    
    @Test func testPeerStoreStoreAndFetch() async throws {
        try await withApp(configure: configure) { app in
            #expect(app.environment == .testing)
            
            let peer = try PeerID(.Ed25519)
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000/").encapsulate(proto: .p2p, address: peer.b58String)
            
            let noPeers = try await app.peers.all().get()
            #expect(noPeers.isEmpty)
            
            try await app.peers.add(peerInfo: PeerInfo(peer: peer, addresses: [ma]))
            
            let peers = try await app.peers.all().get()
            #expect(peers.count == 1)
            #expect(peers.first?.id == peer)
            #expect(peers.first?.addresses.first == ma)
        }
    }
    
    @Test func testPeerStoreNewPeerDiscoveredMetadata() async throws {
        try await withApp(configure: configure) { app in
            #expect(app.environment == .testing)
            
            let peer = try PeerID(.Secp256k1)
            
            let noPeers = try await app.peers.all().get()
            #expect(noPeers.isEmpty)
            
            // Add the peer to our peerstore
            try await app.peers.add(key: peer).get()
            
            let peers1 = try await app.peers.all().get()
            #expect(peers1.count == 1)
            
            let recoveredPeer1 = try #require(peers1.first)
            #expect(recoveredPeer1.id.type == .isPublic)
            #expect(recoveredPeer1.id.keyPair?.keyType == .secp256k1)
            // Expect the `discovered` metadata to be added automatically when a peer is added
            let discoveredMetaData = try #require(recoveredPeer1.metadata[MetadataBook.Keys.Discovered.rawValue])
            let date = Date(timeIntervalSince1970: Double(String(data: Data(discoveredMetaData), encoding: .utf8)!)! )
            // Ensure that the creation data was sometime in the last second
            #expect(date < .now)
            #expect(date > .now.addingTimeInterval(-1))
        }
    }
    
    @Test func testPeerStoreStoreMaxRecords() async throws {
        try await withApp(configure: configure) { app in
            #expect(app.environment == .testing)
            
            let peer = try PeerID(.Secp256k1)
            let mas = [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000/").encapsulate(proto: .p2p, address: peer.b58String),
                try Multiaddr("/ip4/127.0.0.1/tcp/10000/ws/").encapsulate(proto: .p2p, address: peer.b58String),
            ]
            let recordsToTest = 5
            var records:[PeerRecord] = []
            for i in 1...recordsToTest {
                let seqNum = UInt64(Date.now.addingTimeInterval(Double(-i)).timeIntervalSince1970)
                records.append(PeerRecord(peerID: peer, multiaddrs: mas, sequenceNumber: seqNum))
            }
            // Order the records from oldest to newest
            records = records.reversed()
            
            let noPeers = try await app.peers.all().get()
            #expect(noPeers.isEmpty)
            
            // Add the peer to our peerstore
            try await app.peers.add(peerInfo: PeerInfo(peer: peer, addresses: mas)).get()
            
            // Add the records
            for record in records {
                try await app.peers.add(record: record, on: nil).get()
            }
            
            let peers = try await app.peers.all().get()
            #expect(peers.count == 1)
            let recoveredPeer = try #require(peers.first)
            #expect(recoveredPeer.addresses.count == 2)
            #expect(recoveredPeer.records.count == 3)
            #expect(recoveredPeer.records.max(by: { $0.sequenceNumber < $1.sequenceNumber }) == records[recordsToTest-1])
            #expect(recoveredPeer.records.min(by: { $0.sequenceNumber < $1.sequenceNumber }) == records[recordsToTest-3])
            
            let mostRecentRecord = try await app.peers.getMostRecentRecord(forPeer: peer, on: nil).get()
            #expect(mostRecentRecord == records.last)
        }
    }
    
    @Test func testPeerStoreUpdatePeerID() async throws {
        try await withApp(configure: configure) { app in
            #expect(app.environment == .testing)
            
            let peer = try PeerID(.Secp256k1)
            let mas = [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000/").encapsulate(proto: .p2p, address: peer.b58String),
                try Multiaddr("/ip4/127.0.0.1/tcp/10000/ws/").encapsulate(proto: .p2p, address: peer.b58String),
            ]
            
            let noPeers = try await app.peers.all().get()
            #expect(noPeers.isEmpty)
            
            // Extract an ID only PeerID (no public key information)
            let peerIDOnly = try #require( try mas.first?.getPeerID() )
            // Add the peer to our peerstore
            try await app.peers.add(key: peerIDOnly).get()
            
            let peers1 = try await app.peers.all().get()
            #expect(peers1.count == 1)
            
            let recoveredPeer1 = try #require(peers1.first)
            #expect(recoveredPeer1.id.type == .idOnly)
            #expect(recoveredPeer1.id.keyPair == nil)
            // Expect the `discovered` metadata to be added automatically when a peer is added
            let discoveredMetaData = try #require(recoveredPeer1.metadata[MetadataBook.Keys.Discovered.rawValue])
            
            // Update the peer with it's public key
            // This should update the peer with the additional public key info
            try await app.peers.add(key: peer).get()
            
            let peers2 = try await app.peers.all().get()
            #expect(peers2.count == 1)
            
            let recoveredPeer2 = try #require(peers2.first)
            #expect(recoveredPeer2.id.type == .isPublic)
            #expect(recoveredPeer2.id.keyPair?.keyType == .secp256k1)
            #expect(recoveredPeer1.id.b58String == recoveredPeer2.id.b58String)
            #expect(recoveredPeer1.id == recoveredPeer2.id)
            // Ensure the update to the peer didn't adjust the `discovered` metadata
            #expect(recoveredPeer2.metadata[MetadataBook.Keys.Discovered.rawValue] == discoveredMetaData)
        }
    }
    
    @Test func testPeerStoreStoreAndFetchCompPeer() async throws {
        try await withApp(configure: configure) { app in
            #expect(app.environment == .testing)
            
            let peer = try PeerID(.Secp256k1)
            let mas = [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000/").encapsulate(proto: .p2p, address: peer.b58String),
                try Multiaddr("/ip4/127.0.0.1/tcp/10000/ws/").encapsulate(proto: .p2p, address: peer.b58String),
            ]
            let record = PeerRecord.init(peerID: peer, multiaddrs: mas)
            let compPeer = ComprehensivePeer(
                id: peer,
                addresses: Set(mas),
                protocols: Set([
                    .init("/echo/1.0.0")!,
                    .init("/echo/2.0.0")!,
                    .init("/echo/2.0.1")!,
                    .init("/ipfs/ping/1.0.0")!
                ]),
                metadata: [
                    MetadataBook.Keys.Discovered.rawValue: Array<UInt8>("\(Date.now.timeIntervalSince1970)".utf8),
                    MetadataBook.Keys.Prunable.rawValue: Array<UInt8>(arrayLiteral: MetadataBook.PrunableMetadata.Prunable.necessary.rawValue)
                ],
                records: Set([
                    record
                ])
            )
            
            let noPeers = try await app.peers.all().get()
            #expect(noPeers.isEmpty)
            
            // Add the peer to our peerstore
            try await app.peers.add(key: peer).get()
            // Add the Multiaddresses
            try await app.peers.add(addresses: mas, toPeer: peer).get()
            // Add the protocols
            try await app.peers.add(protocols: Array(compPeer.protocols), toPeer: peer).get()
            // Add the PeerRecord
            try await app.peers.add(record: record, on: nil).get()
            // Add the Metadata
            try await app.peers.add(
                metaKey: MetadataBook.Keys.Prunable.rawValue,
                data: Array<UInt8>(arrayLiteral: MetadataBook.PrunableMetadata.Prunable.necessary.rawValue),
                toPeer: peer
            ).get()
            
            // Fetch the peer
            #expect(try await app.peers.count().get() == 1)
            let recoveredPeer = try #require(try await app.peers.all().get().first)
            #expect(recoveredPeer == compPeer)
            
            // Ensure we can query the peer by their id
            let recoveredPeer2 = try await app.peers.getPeerInfo(byID: peer.b58String).get()
            #expect(recoveredPeer2.peer == recoveredPeer.id)
            #expect(recoveredPeer2.addresses.isEquivalent(to: recoveredPeer.addresses))
            
            // Ensure we can query the peer via their mutliaddr
            let recoveredPeer3 = try await app.peers.getPeerInfo(byAddress: mas.randomElement()!, on: nil).get()
            #expect(recoveredPeer3.peer == recoveredPeer.id)
            #expect(recoveredPeer3.addresses.isEquivalent(to: recoveredPeer.addresses))
            
            // Ensure we can query the peer via their supported protocols
            let protocolPeers = try await app.peers.getPeers(supportingProtocol: compPeer.protocols.randomElement()!, on: nil).get()
            #expect(protocolPeers.count == 1)
            let recoveredPeer4 = try #require(protocolPeers.first)
            #expect(recoveredPeer4 == recoveredPeer.id.b58String)
            
            // Delete the peer
            do {
                try await app.peers.remove(key: peer).get()
            } catch {
                print(String(reflecting: error))
            }
            // Ensure that our foreign key onDelete cascades work
            #expect(try await app.peers.count().get() == 0)
            #expect(try await PeerStoreEntry.query(on: app.db).count().get() == 0)
            #expect(try await PeerStoreEntry_Multiaddr.query(on: app.db).count().get() == 0)
            #expect(try await PeerStoreEntry_Protocol.query(on: app.db).count().get() == 0)
            #expect(try await PeerStoreEntry_Record.query(on: app.db).count().get() == 0)
            #expect(try await PeerStoreEntry_Metadata.query(on: app.db).count().get() == 0)
        }
    }
}

extension Application {
    func resetDatabase() async throws {
        try await PeerStoreEntry_Multiaddr.query(on: self.db).delete(force: true).get()
        try await PeerStoreEntry_Protocol.query(on: self.db).delete(force: true).get()
        try await PeerStoreEntry_Record.query(on: self.db).delete(force: true).get()
        try await PeerStoreEntry_Metadata.query(on: self.db).delete(force: true).get()
        try await PeerStoreEntry.query(on: self.db).delete(force: true).get()
    }
}

extension ComprehensivePeer: @retroactive Equatable {
    public static func == (lhs: LibP2PCore.ComprehensivePeer, rhs: LibP2PCore.ComprehensivePeer) -> Bool {
        lhs.id == rhs.id
        && lhs.addresses == rhs.addresses
        && lhs.protocols == rhs.protocols
        && lhs.records == rhs.records
        && lhs.metadata.keys == rhs.metadata.keys
    }
}

extension Array where Element == Multiaddr {
    /// Compares two arrays, ensuring they both contain the same elements, but not necessarily in the same order
    func isEquivalent(to: Array<Multiaddr>) -> Bool {
        var copy = to
        for item in self {
            guard let match = copy.firstIndex(of: item) else {
                return false
            }
            copy.remove(at: match)
        }
        guard copy.isEmpty else { return false }
        return true
    }
    
    /// Compares two arrays, ensuring they both contain the same elements, but not necessarily in the same order
    func isEquivalent(to: Set<Multiaddr>) -> Bool {
        self.isEquivalent(to: Array(to))
    }
}
