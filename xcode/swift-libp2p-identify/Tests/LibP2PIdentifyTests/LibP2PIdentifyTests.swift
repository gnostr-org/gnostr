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

import LibP2PNoise
import LibP2PYAMUX
import Testing

@testable import LibP2P
@testable import LibP2PIdentify

@Suite("Libp2p Identify Tests", .serialized, .timeLimit(.minutes(5)))
struct LibP2PIdentifyTests {

    @Test func testIPFSIdentifyPayload() throws {
        let payloadString =
            "0aab04080012a60430820222300d06092a864886f70d01010105000382020f003082020a0282020100a1f5c0e7c0d5e556afc0e84566f8c565773adb548ddc219ca9688613a0096c2dfd069804c84968545b9c9df19dd131cc8408b7781df7ddfaf208a42a821523ce03955164a62dcab6bd10dd26f8507517567ca128f00a056d8636b9549ddb59ca727628775c90bd91d6251adbdfd36bf68a09c3bfe69e1b1587e8f31a4b55afc8095e7b6f6683165f9c0ef0ad1b22d8b73749ee02aa46566cd5f7a9ff6eb1099fe36b363abd4e1293108a6d473a349e77aca15e49b20ffe61b4222eb3a634e8481d71a7fdceea88a2044fa5cedde1dee314e27880bc713ca578814684e85e0d21cff40e23c341f13ee1a06452f284664999862973e51d692b578cd9b7de89d786ad6baebcf8dfc343db8eda434a15929591917c52bf16741359149d0e7092bc919928f1d5b25cb48b0f90a7a05b0eb29adca993f893c6fb137a53a5c470a8a309b574bb4fd80879bde7dcc237eaf2ce9a17b9193032df99c8bf551987561ee264a09730f9029610571625e0d0e1e2a7f90469a6a480ed08cf9b4c3af0567bfe9abf470079d8cc7d7f22efc83598f86c9e0678caf79e2299a99c47c8d057e7f3b8af40185c8dd499a1c167c358d7ab83af6581944ce0b8b6bd2cfe4bf80c8c9e7f61fe94816df79e12ae5e82c588f894b86fd599da5912f8754de2a23f2d1529845a5570a72d8d8537325b95dd3c69d9ca30b8186c20170d10955b7da216822c73020301000112080468838352060fa1120b046883835291020fa1cc03120b046883835291020fa1cd031257046883835291020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e1257046883835291020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e1a0d2f697066732f626974737761701a132f697066732f626974737761702f312e302e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e322e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a0f2f697066732f6b61642f312e302e301a132f697066732f6c616e2f6b61642f312e302e301a102f697066732f70696e672f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a1f2f6c69627032702f636972637569742f72656c61792f302e322e302f686f701a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a0d2f6c69627032702f64637574721a032f782f2208044caa3c8506e78a2a0a697066732f302e312e3032196b75626f2f302e32312e302d7263332f39663231636635333842cb0b0aab04080012a60430820222300d06092a864886f70d01010105000382020f003082020a0282020100a1f5c0e7c0d5e556afc0e84566f8c565773adb548ddc219ca9688613a0096c2dfd069804c84968545b9c9df19dd131cc8408b7781df7ddfaf208a42a821523ce03955164a62dcab6bd10dd26f8507517567ca128f00a056d8636b9549ddb59ca727628775c90bd91d6251adbdfd36bf68a09c3bfe69e1b1587e8f31a4b55afc8095e7b6f6683165f9c0ef0ad1b22d8b73749ee02aa46566cd5f7a9ff6eb1099fe36b363abd4e1293108a6d473a349e77aca15e49b20ffe61b4222eb3a634e8481d71a7fdceea88a2044fa5cedde1dee314e27880bc713ca578814684e85e0d21cff40e23c341f13ee1a06452f284664999862973e51d692b578cd9b7de89d786ad6baebcf8dfc343db8eda434a15929591917c52bf16741359149d0e7092bc919928f1d5b25cb48b0f90a7a05b0eb29adca993f893c6fb137a53a5c470a8a309b574bb4fd80879bde7dcc237eaf2ce9a17b9193032df99c8bf551987561ee264a09730f9029610571625e0d0e1e2a7f90469a6a480ed08cf9b4c3af0567bfe9abf470079d8cc7d7f22efc83598f86c9e0678caf79e2299a99c47c8d057e7f3b8af40185c8dd499a1c167c358d7ab83af6581944ce0b8b6bd2cfe4bf80c8c9e7f61fe94816df79e12ae5e82c588f894b86fd599da5912f8754de2a23f2d1529845a5570a72d8d8537325b95dd3c69d9ca30b8186c20170d10955b7da216822c730203010001120203011a93030a221220b04a57d40eca138809f139a76b12044333c3740391c9bf1ce9d8e21a79210bfd10b38ba6afbab1bab9171a0a0a080468838352060fa11a0d0a0b046883835291020fa1cc031a0d0a0b046883835291020fa1cd031a590a57046883835291020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e1a590a57046883835291020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e1a0a0a08047f000001060fa11a0d0a0b047f00000191020fa1cc031a0d0a0b047f00000191020fa1cd031a590a57047f00000191020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e2a8004089547ae16ad9517e74d09c8280fcb8e305875cd5614964e95629660de26415a41a20ce232d448d5cbb2c245c6646c03e8ec5082277fcb612dc44d11efff4a3443b2690fd8abe011534e325bac89c9aeb5b832011c0ace99e29307bf37e790201625e33d11697c81d33e7e703d613d787cef34634b1870a4209b50b0bdf7e93bee1d954ab86e767373d524e59971da4ed63a1b330ca95ebaa0f4ba3667407e8f1e48f28008d10d510a03f79bece87b8a9b540ecb00a7cca06165ecd669d7ca51a3aa47436c77f53933e573fecee192519275638dcb068f4271cfc821684e2050969b3603473404cdb510b9cba00dfa7d74ec0bbd60e2898cd0efdd167ca137468ebeccb81d3e18725ea63c747c04de5a49cf974d36c4380bf16643b3b232e5f204d48494c18d82fb891d02416344b9f82d9e02909b7658bac1451dd685b2baf89e11e2ad4629f9b6ed0f791ae483a11f312b0370d73c008ed919addc22247e2935679868b4f80414c268b2756e6a65eb14e3de43b4213acdd676b1ba16ad168ead8500ce8c21074d5288672a3c3e600db5e1bb834f818e719752f09a2219fcaf8db2df74903429590cfdd59ccfed4e08baf47812261ae789437d876a7e8b252862aaed6dabf8484b25bc318b0ac8261833fa4599abb5b0fbc1c0755badaec2adbe09ae3444a02cbc09f2c6a4f8f3ea977af8c1ec0c1c015faaaf63bf7e7b14ff"

        let payload = [UInt8](hex: payloadString)

        let identifyProto = try LibP2PIdentify.IdentifyMessage(serializedBytes: payload)

        for address in identifyProto.listenAddrs {
            #expect(throws: Never.self, "Failed to decode multiaddr '\(address.toHexString())'") {
                try Multiaddr(address)
            }
        }
        #expect(identifyProto.listenAddrs.count == 5)

        for proto in identifyProto.protocols {
            #expect(SemVerProtocol(proto) != nil, "Failed to decode protocol '\(proto)'")
        }
        #expect(identifyProto.protocols.count == 14)
    }

    @Test func testIPFSIdentifyPushPayloadJSClient() throws {
        /// An example /ipfs/id/push/1.0.0 payload from a JS Client
        let payloadString =
            "a7061208047f00000106c223120804c0a8011706c223120804c0a8011606c2231208044caa3c850688fc1208044caa3c850686601a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e30429e050aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a02820101009a3520ce8cfcfa4fc1d9b1fecb0e9c6241188dd8e8dec881d44b4e69f1058eaf710550216c0b7d51e26a22844e4737e17135a954cb215953fff28dfd6976794c26aad507225231afb2db2e31d85b9ca680803bded3c7e896cf0959d945c451733563cd6684f6de597cbec0fdb11254e02044744ec9ffb61a00d120f6bbdc09b95bccedd07b701707626a95e891fe29609e7514ee9ba3b506cb2a3ffe0b6e6dbeae4adb678fa8551a14d8344ba0584aab0a8bb7296b6ee8f85ce2375f290c5d5e7eb905f3d49cee6cd381f65b1ce8af9e442fb0f218610ee14c833919e2aa260a7c77ba13baeca68809df32aaa05ae8f27ff9f04ce57938863c91e346f071fe350203010001120203011a670a221220add8ac59b1fe19f20dfaa2228c239e3c131d73e0a7848ac869d5eb959a27ec6c10a5d9c6f28b301a0a0a08047f00000106c2231a0a0a0804c0a8011706c2231a0a0a0804c0a8011606c2231a0a0a08044caa3c850688fc1a0a0a08044caa3c850686602a800228a3216af2fd7701d4cc0014d5fc410a8423a21170b2784516d7740405c7bf903cd7ed837725e857680c0cb0dbaf6fc4f95d7fe692872f16417e8fcfaad3cdd3b529e8fb224ba3a0cabdc36bee1f77ab013aa35beae0e46c4bf4e438ffad4a2fa558b43323a2526547bb47935ea1f3ea9fa67dc8bd51160f4a9875717ec146f0ef293cd4892e0395b042e6e7f0fd0af9babb06e1527ee0f11b50671d7a190877d354abe11e32c607c7492598b37a29c35925c43b89f5a4e92e97b1973ecaa1027714e2439ffe27b390e31d4763d6b5fe637c95848e8832f1bb7ec4de20ee9b767286661a8d0aeeedf5cdb97e639fa530b237a2914459e173fb2aad3363d8bb08"

        let payload = [UInt8](hex: payloadString)

        let lengthPrefix = uVarInt(payload)

        let identifyProto = try? LibP2PIdentify.IdentifyMessage(
            serializedBytes: payload.dropFirst(lengthPrefix.bytesRead)
        )

        #expect(identifyProto != nil)

        for addy in identifyProto!.listenAddrs {
            #expect(throws: Never.self) { try Multiaddr(addy) }
        }
        #expect(identifyProto?.listenAddrs.count == 5)

        for proto in identifyProto!.protocols {
            #expect(SemVerProtocol(proto) != nil)
        }
        #expect(identifyProto?.protocols.count == 4)

        // Try and decode the Sealed Envelope containing the Signed PeerRecord
        // We should use the public key that we have on file to verify the signedEnvelope
        guard
            let signedEnvelope = try? SealedEnvelope(
                marshaledEnvelope: identifyProto!.signedPeerRecord.byteArray,
                verifiedWithPublicKey: nil
            )
        else {
            Issue.record("Identify::Failed to decode Envelope containing PeerRecord")
            return
        }
        //print("\n\(signedEnvelope)")

        // Try and decode the PeerRecord (passing in the publicKey data so we get a pubkey PeerID, instead of an idOnly PeerID)
        guard let peerRecord = try? PeerRecord(marshaledData: Data(signedEnvelope.rawPayload)) else {
            Issue.record("Identify::Failed to decode Peer Record")
            return
        }

        print(peerRecord)

        #expect(peerRecord.peerID.b58String == "Qma3GsJmB47xYuyahPZPSadh1avvxfyYQwk8R3UnFrQ6aP")
    }

    @Test func testIDPushRecordDecoding() throws {

        let application = Application(.testing)
        application.environment.arguments = ["libp2p"]

        try application.start()

        defer { application.shutdown() }

        /// Ensure our identifiedPeer Events are being fired correctly
        var peerUpdateEvents: Int = 0
        application.events.on(
            application,
            event: .remotePeerProtocolChange({ _ in
                peerUpdateEvents += 1
            })
        )

        for hexRecord in Fixtures.PushRecords {
            let payload = [UInt8](hex: hexRecord)

            let identifyProto = try LibP2PIdentify.IdentifyMessage(serializedBytes: payload)
            let signedEnvelope = try SealedEnvelope(
                marshaledEnvelope: identifyProto.signedPeerRecord.byteArray,
                verifiedWithPublicKey: identifyProto.publicKey.byteArray
            )
            let peerRecord = try PeerRecord(
                marshaledData: Data(signedEnvelope.rawPayload),
                withPublicKey: identifyProto.publicKey
            )

            //print(identifyProto)
            //print(signedEnvelope)
            //print(peerRecord)
            //print((try? Multiaddr(identifyProto.observedAddr)) ?? "NIL")

            updateIdentifiedPeerInPeerStore(peerRecord, identifyMessage: identifyProto)
        }

        func updateIdentifiedPeerInPeerStore(_ peerRecord: PeerRecord, identifyMessage: LibP2PIdentify.IdentifyMessage)
        {
            let eventLoop = application.eventLoopGroup.next()
            let identifiedPeer = peerRecord.peerID
            guard identifiedPeer != application.peerID else { return }

            var tasks: [EventLoopFuture<Void>] = []

            // This call to add key will only update/upgrade the PeerID in the PeerStore, it wont 'downgrade' an existing PeerID
            tasks.append(application.peers.add(key: identifiedPeer, on: eventLoop))

            // Update our peers listening addresses
            let listeningAddresses = identifyMessage.listenAddrs.compactMap { multiaddrData -> Multiaddr? in
                if let ma = try? Multiaddr(multiaddrData) {
                    if !ma.protocols().contains(.p2p) {
                        return try? ma.encapsulate(proto: .p2p, address: identifiedPeer.b58String)
                    } else {
                        return ma
                    }
                }
                return nil
            }
            tasks.append(application.peers.add(addresses: listeningAddresses, toPeer: identifiedPeer, on: eventLoop))

            // Update our peers known protocols
            let protocols = identifyMessage.protocols.compactMap { SemVerProtocol($0) }
            tasks.append(application.peers.add(protocols: protocols, toPeer: identifiedPeer, on: eventLoop))

            // Add the PeerRecord to our Records list
            tasks.append(application.peers.add(record: peerRecord, on: eventLoop))

            // Update our peers metadata (agent version, protocol version, etc.. maybe include a verified attribute (the signed peer record))
            //connection.logger.trace("Identify::Adding Metadata to peer \(identifiedPeer.b58String)")
            //connection.logger.trace("Identify::AgentVersion: \(identifyMessage.agentVersion)")
            if identifyMessage.hasAgentVersion, let agentVersion = identifyMessage.agentVersion.data(using: .utf8) {
                tasks.append(
                    application.peers.add(
                        metaKey: .AgentVersion,
                        data: agentVersion.byteArray,
                        toPeer: identifiedPeer,
                        on: eventLoop
                    )
                )
            }
            //connection.logger.trace("Identify::ProtocolVersion: \(identifyMessage.protocolVersion)")
            if identifyMessage.hasProtocolVersion,
                let protocolVersion = identifyMessage.protocolVersion.data(using: .utf8)
            {
                tasks.append(
                    application.peers.add(
                        metaKey: .ProtocolVersion,
                        data: protocolVersion.byteArray,
                        toPeer: identifiedPeer,
                        on: eventLoop
                    )
                )
            }
            //connection.logger.trace("Identify::ObservedAddress: \((try? Multiaddr(identifyMessage.observedAddr).description) ?? "NIL")")
            if identifyMessage.hasObservedAddr,
                let ma = try? Multiaddr(identifyMessage.observedAddr).description.data(using: .utf8)
            {
                tasks.append(
                    application.peers.add(
                        metaKey: .ObservedAddress,
                        data: ma.byteArray,
                        toPeer: identifiedPeer,
                        on: eventLoop
                    )
                )
            }

            // -TODO: Our Connection should do this when we complete our security handshake, also we should remove this here...
            tasks.append(
                application.peers.add(
                    metaKey: .LastHandshake,
                    data: String(Date().timeIntervalSince1970).bytes,
                    toPeer: identifiedPeer,
                    on: eventLoop
                )
            )

            // Wait for the metadata to be updated then alert the application of the changes...
            tasks.flatten(on: eventLoop).whenComplete { _ in
                //print("Identify::Done Adding Metadata to PeerStore. Alerting Application to Remote Peer Protocol Change.")
                //print(peerRecord)
                application.events.post(
                    .remotePeerProtocolChange(
                        RemotePeerProtocolChange(
                            peer: identifiedPeer,
                            protocols: protocols,
                            connection: DummyConnection()
                        )
                    )
                )
            }
        }

        sleep(1)

        print("Added \(Fixtures.PushRecords.count) Push ID Records to our PeerStore")
        print("Peer Count: \((try? application.peers.count().wait()) ?? -1)")
        #expect(Fixtures.PushRecords.count == 16)
        #expect(try! application.peers.count().wait() == 7)

        /// Ensure we received the correct number of updates
        #expect(peerUpdateEvents == 16)

        /// Instantiate a peer from the test fixtures
        let aRemotePeer = try PeerID(cid: "QmRYiTAVhmPUuE6dnLa2vQGH6pQvUatKJtGFtDRc9bAkeQ")
        /// Fetch the Records for the remote peer
        let remotePeerRecords = try application.peers.getRecords(forPeer: aRemotePeer, on: nil).wait()
        #expect(remotePeerRecords.count > 0)
        let latestPeerRecord = try application.peers.getMostRecentRecord(forPeer: aRemotePeer, on: nil).wait()
        #expect(latestPeerRecord?.sequenceNumber == 1_663_285_097_713_605_627)

        /// Trim the Records for the remote peer
        try application.peers.trimRecords(forPeer: aRemotePeer, on: nil).wait()
        /// And ensure that the latest Record was kept
        let remotePeerRecordsAfter = try application.peers.getRecords(forPeer: aRemotePeer, on: nil).wait()
        #expect(remotePeerRecordsAfter.count == 1)
        let latestPeerRecordAfter = try application.peers.getMostRecentRecord(forPeer: aRemotePeer, on: nil).wait()
        #expect(latestPeerRecordAfter?.sequenceNumber == 1_663_285_097_713_605_627)

        //application.peers.dumpAll()

        sleep(1)
    }

    @Test func testLibP2PInternalPingMultiaddr() throws {
        let app1 = try makeClient(port: 10_000)
        let app2 = try makeClient(port: 10_001)

        defer {
            app1.shutdown()
            app2.shutdown()
        }

        try app1.start()
        try app2.start()

        let ma = try Multiaddr(app2.listenAddresses.first!.description + "/p2p/" + app2.peerID.b58String)

        let latency = try app1.identify.ping(addr: ma).wait()

        print("Latency: \(latency.nanoseconds) ns")
        #expect(latency.nanoseconds >= 0)
    }

    @Test func testLibP2PInternalPingMultiaddr_Async() async throws {
        let app1 = try makeClient(port: 10_000)
        let app2 = try makeClient(port: 10_001)

        try await app1.startup()
        try await app2.startup()

        let ma = try app2.listenAddresses.first!.encapsulate(proto: .p2p, address: app2.peerID.b58String)

        let ping = try await app1.identify.ping(addr: ma)
        print("Latency: \(ping.nanoseconds) ns")
        #expect(ping.nanoseconds >= 0)

        try await app1.asyncShutdown()
        try await app2.asyncShutdown()
    }

    @Test func testLibP2PInternalPingPeer() throws {
        let app1 = try makeClient(port: 10_000)
        let app2 = try makeClient(port: 10_001)

        defer {
            app1.shutdown()
            app2.shutdown()
        }

        try app1.start()
        try app2.start()

        try app1.peers.add(peerInfo: app2.peerInfo).wait()

        let ping = try app1.identify.ping(peer: app2.peerID).wait()
        print("Latency: \(ping.nanoseconds) ns")
        #expect(ping.nanoseconds >= 0)
    }

    @Test func testLibP2PInternalPingPeer_Async() async throws {
        let app1 = try makeClient(port: 10_000)
        let app2 = try makeClient(port: 10_001)

        try await app1.startup()
        try await app2.startup()

        do {
            try await app1.peers.add(peerInfo: app2.peerInfo)

            let ping = try await app1.identify.ping(peer: app2.peerID)
            print("Latency: \(ping.nanoseconds) ns")
            #expect(ping.nanoseconds >= 0)
        } catch {
            Issue.record(error)
        }

        try await app1.asyncShutdown()
        try await app2.asyncShutdown()
    }

    @Test func testLibP2PInternalPingPeerCascadeMultipleInflightPings() async throws {
        let app1 = try makeClient(port: 10_000)
        let app2 = try makeClient(port: 10_001)

        try await app1.startup()
        try await app2.startup()

        do {
            try await app1.peers.add(peerInfo: app2.peerInfo)

            // Note: These pings happen concurrently
            async let latency1 = app1.identify.ping(peer: app2.peerID)
            async let latency2 = app1.identify.ping(peer: app2.peerID)
            async let latency3 = app1.identify.ping(peer: app2.peerID)

            // Even when we await them like this here...
            let ping1 = try await latency1
            let ping2 = try await latency2
            let ping3 = try await latency3

            let connectionCount = try await app1.connectionManager.getTotalConnectionCount().get()
            let streamCount = try await app1.connectionManager.getTotalStreamCount().get()

            print("Connection Count: \(connectionCount)")
            print("Stream Count: \(streamCount)")

            #expect(connectionCount == 1)
            // 2 streams for ID protocol and 1 stream for all three pings
            #expect(streamCount == 3)

            // All of the pings should be indentical because all three calls where cascaded into one
            #expect(ping1 == ping2)
            #expect(ping2 == ping3)
        } catch {
            Issue.record(error)
        }

        try await app1.asyncShutdown()
        try await app2.asyncShutdown()
    }

    @Test func testLibP2PInternalPingPeerSequentialPingsUseSameConnection() async throws {
        let app1 = try makeClient(port: 10_000)
        let app2 = try makeClient(port: 10_001)

        try await app1.startup()
        try await app2.startup()

        do {
            try await app1.peers.add(peerInfo: app2.peerInfo)

            // Note: These pings happen sequentially
            let ping1 = try await app1.identify.ping(peer: app2.peerID)
            let ping2 = try await app1.identify.ping(peer: app2.peerID)
            let ping3 = try await app1.identify.ping(peer: app2.peerID)

            let connectionCount = try await app1.connectionManager.getTotalConnectionCount().get()
            let streamCount = try await app1.connectionManager.getTotalStreamCount().get()

            print("Connection Count: \(connectionCount)")
            print("Stream Count: \(streamCount)")

            #expect(connectionCount == 1)
            // 2 streams for ID protocol and 1 stream for each of the 3 pings
            #expect(streamCount == 5)

            // All three pings should be different because they happened sequentially
            #expect(ping1 != ping2)
            #expect(ping2 != ping3)
        } catch {
            Issue.record(error)
        }

        try await app1.asyncShutdown()
        try await app2.asyncShutdown()
    }

    @Test() func testInternalInterop() async throws {
        let host = try makeEchoHost(port: 10000)
        let client = try makeClient(port: 10001)

        try await host.startup()
        try await client.startup()

        do {
            let message: Data = "Hello Swift LibP2P".data(using: .utf8)!

            /// Fire off an echo request
            let response = try await client.newRequest(
                to: host.listenAddresses.first!.encapsulate(proto: .p2p, address: host.peerID.b58String),
                forProtocol: "/echo/1.0.0",
                withRequest: message,
                withHandlers: .handlers([.newLineDelimited])
            ).get()

            #expect(response == message)

            try await Task.sleep(for: .milliseconds(10))
        } catch {
            Issue.record(error)
        }

        try await host.asyncShutdown()
        try await client.asyncShutdown()
    }

    @Test(.timeLimit(.minutes(2)), arguments: [3, 5, 10])
    func testInternalInteropMultipleRequests_Sequentially(_ numberOfRequests: Int) async throws {
        let host = try makeEchoHost(port: 10000)
        let client = try makeClient(port: 10001)

        try await host.startup()
        try await client.startup()

        do {
            // Yamux handles 10_000 requests in ~40 seconds
            for _ in 0..<numberOfRequests {
                /// Fire off an echo request
                let response = try await client.newRequest(
                    to: host.listenAddresses.first!.encapsulate(proto: .p2p, address: host.peerID.b58String),
                    forProtocol: "/echo/1.0.0",
                    withRequest: "Hello Swift LibP2P".data(using: .utf8)!,
                    withHandlers: .handlers([.newLineDelimited])
                ).get()

                #expect(response == "Hello Swift LibP2P".data(using: .utf8)!)
            }

            try await Task.sleep(for: .milliseconds(10))

            let connections = try await host.connectionManager.getTotalConnectionCount().get()
            let streams = try await host.connectionManager.getTotalStreamCount().get()

            #expect(connections == 1)
            #expect(streams == numberOfRequests + 2)
        } catch {
            Issue.record(error)
        }

        try await host.asyncShutdown()
        try await client.asyncShutdown()
    }
}

extension LibP2PIdentifyTests {
    fileprivate func makeEchoHost(
        port: Int,
        peerID: PeerID? = nil,
        logLevel: Logger.Level = .notice
    ) throws -> Application {
        let lib = try Application(.testing, peerID: peerID ?? PeerID(.Ed25519))
        lib.environment.arguments = ["libp2p"]
        lib.security.use(.noise)
        lib.muxers.use(.yamux)
        lib.servers.use(.tcp(host: "127.0.0.1", port: port))

        lib.logger.logLevel = logLevel

        lib.routes.group("echo", handlers: [.newLineDelimited]) { echo in
            echo.on("1.0.0") { req -> Response<ByteBuffer> in
                switch req.event {
                case .ready: return .stayOpen
                case .data(let data): return .respondThenClose(data)
                case .closed: return .close
                case .error(let error):
                    req.logger.error("\(error)")
                    return .close
                }
            }
        }

        return lib
    }

    fileprivate func makeClient(
        port: Int,
        peerID: PeerID? = nil,
        logLevel: Logger.Level = .notice
    ) throws -> Application {
        let lib = try Application(.testing, peerID: peerID ?? PeerID(.Ed25519))
        lib.environment.arguments = ["libp2p"]
        lib.security.use(.noise)
        lib.muxers.use(.yamux)
        lib.servers.use(.tcp(host: "127.0.0.1", port: port))

        lib.logger.logLevel = logLevel

        return lib
    }
}

struct Fixtures {
    static let PushRecords: [String] = [
        "0a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c12080436afffd706463312080436afffd70618781a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e303216676f2d697066732f302e31322e322f3065386231323142d3010a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c120203011a650a260024080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c10afa09196e38dc78a171a0a0a08047f000001060fa11a0d0a0b047f00000191020fa1cc031a0a0a080436afffd70646331a0a0a080436afffd70618782a403d2f87d4cdfd3398d928e8e5dd7d095d1bc45140901b6fed098d75917af60c3f82b17378c06424a6970aafd1ded13dab04c7efa1b9834dafc44c18dc9282750e",
        "0a2408011220c288022a7cb6e03d5588910f6b1d4b4b2b087dd5357fc2730c6ea37c85f2dd5a1233044c1c8a39060fa1a50326002408011220becd6efe2f04e29b00127981a734f89e387c8d9f40630d9268c41ea7b6cbddbfa2021236044c1c8a39910214decc03a50326002408011220becd6efe2f04e29b00127981a734f89e387c8d9f40630d9268c41ea7b6cbddbfa2021236044c1c8a399102a655cc03a50326002408011220becd6efe2f04e29b00127981a734f89e387c8d9f40630d9268c41ea7b6cbddbfa202122f042e65e026060fa1a5032212209ff4ccf86b179ee23833926a76cc409bc7440a7d5e0d82661af42654c2aae52da202123b292a03b0c0000300d0000000000117e001060fa1a5032212209ff4ccf86b179ee23833926a76cc409bc7440a7d5e0d82661af42654c2aae52da202123b290064ff9b00000000000000002e65e026060fa1a5032212209ff4ccf86b179ee23833926a76cc409bc7440a7d5e0d82661af42654c2aae52da2021a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f1a0d2f6c69627032702f64637574722208042d4f59960627102a0a697066732f302e312e30320c6b75626f2f302e31352e302f42cb040a2408011220c288022a7cb6e03d5588910f6b1d4b4b2b087dd5357fc2730c6ea37c85f2dd5a120203011adc030a26002408011220c288022a7cb6e03d5588910f6b1d4b4b2b087dd5357fc2730c6ea37c85f2dd5a10d8faf7c7a195c78a171a0a0a08047f000001060fa11a160a142900000000000000000000000000000001060fa11a0d0a0b047f00000191020fa1cc031a190a17290000000000000000000000000000000191020fa1cc031a350a33044c1c8a39060fa1a50326002408011220becd6efe2f04e29b00127981a734f89e387c8d9f40630d9268c41ea7b6cbddbfa2021a380a36044c1c8a39910214decc03a50326002408011220becd6efe2f04e29b00127981a734f89e387c8d9f40630d9268c41ea7b6cbddbfa2021a380a36044c1c8a399102a655cc03a50326002408011220becd6efe2f04e29b00127981a734f89e387c8d9f40630d9268c41ea7b6cbddbfa2021a310a2f042e65e026060fa1a5032212209ff4ccf86b179ee23833926a76cc409bc7440a7d5e0d82661af42654c2aae52da2021a3d0a3b292a03b0c0000300d0000000000117e001060fa1a5032212209ff4ccf86b179ee23833926a76cc409bc7440a7d5e0d82661af42654c2aae52da2021a3d0a3b290064ff9b00000000000000002e65e026060fa1a5032212209ff4ccf86b179ee23833926a76cc409bc7440a7d5e0d82661af42654c2aae52da2022a40532272eb3c87ceed8e1857dc31292ab4be053bf7d398b484854dae0d661bcd7abad68df9795f5c68aaa2d76d202315df56bb48de4722dc1e245764efd60d4e08",
        "0a2408011220aab81057158609017aa07134a7daf0f4193ff657c5b49f3eb3661731b3ced283120804ac1c6987060fa1120b04ac1c698791020fa1cc031a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f63633935383533428a020a2408011220aab81057158609017aa07134a7daf0f4193ff657c5b49f3eb3661731b3ced283120203011a9b010a26002408011220aab81057158609017aa07134a7daf0f4193ff657c5b49f3eb3661731b3ced28310cdb9bfdabb9fc78a171a190a17290000000000000000000000000000000191020fa1cc031a0a0a0804ac1c6987060fa11a0a0a08047f000001060fa11a160a142900000000000000000000000000000001060fa11a0d0a0b04ac1c698791020fa1cc031a0d0a0b047f00000191020fa1cc032a4053f92e58c0b4aede999ef85efb2175b124a0704429473d801e51a686f69f4b5c466817a00c845781a28a50d8092f39fe6a5377260305befa0f4cb398b90ba709",
        "0a240801122087156bc9b39c857f57a4479913780a4b8f9abcf626a40b67931011f0ce49770a121429260100450402e170d0937151478c493e060fa1120b04491e4cb691020fa1cc03121729260100450402e170d0937151478c493e91020fa1cc03121729260100450402e170848359e7a9f401d091020fa1cc03120804491e4cb6060fa1121429260100450402e170848359e7a9f401d0060fa11a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e30320e676f2d697066732f302e372e302f42f0020a240801122087156bc9b39c857f57a4479913780a4b8f9abcf626a40b67931011f0ce49770a120203011a81020a2600240801122087156bc9b39c857f57a4479913780a4b8f9abcf626a40b67931011f0ce49770a10b095bec7edaec78a171a0a0a08047f000001060fa11a160a142900000000000000000000000000000001060fa11a0d0a0b047f00000191020fa1cc031a190a17290000000000000000000000000000000191020fa1cc031a190a1729260100450402e170d0937151478c493e91020fa1cc031a190a1729260100450402e170848359e7a9f401d091020fa1cc031a0a0a0804491e4cb6060fa11a160a1429260100450402e170848359e7a9f401d0060fa11a160a1429260100450402e170d0937151478c493e060fa11a0d0a0b04491e4cb691020fa1cc032a4054fe02ec7e1c4e4ed1e1ec1e26c65417006c10f0a9d6da97d3474b4c83adec860ceda195f1546a57530bd144772173942409a4eae8b17b75aa67a11e08a93c01",
        "0a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c12080436afffd706463312080436afffd70635771a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e303216676f2d697066732f302e31322e322f3065386231323142d3010a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c120203011a650a260024080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c1095c8fdf4f0b2c78a171a0a0a08047f000001060fa11a0d0a0b047f00000191020fa1cc031a0a0a080436afffd70646331a0a0a080436afffd70635772a40718c453be6bd940d95c25c76e000298c29abd16978820887e2c38a5613178ab30b4ee4014683b27121fb7f1a6bd241879e7ef026358dffb4cdd4a24d5be75207",
        "0a240801122094facba934611e9834a55fb2331318fb6f5d342a0121f23540936f137041f828120804ac161587060fa1120b04ac16158791020fa1cc03120804b7833621060fa1120b04b783360a9102339fcc031a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f6363393538353342a5020a240801122094facba934611e9834a55fb2331318fb6f5d342a0121f23540936f137041f828120203011ab6010a2600240801122094facba934611e9834a55fb2331318fb6f5d342a0121f23540936f137041f8281091fe80ec99b5c78a171a0a0a0804ac161587060fa11a0a0a08047f000001060fa11a160a142900000000000000000000000000000001060fa11a0d0a0b04ac16158791020fa1cc031a0d0a0b047f00000191020fa1cc031a190a17290000000000000000000000000000000191020fa1cc031a0d0a0b04b783360a9102339fcc031a0a0a0804b7833621060fa12a40a5e516a8e56fe02c861e72f57ae0550cc761da863c5b03aad6cf8fa920197f02139298b36f25a6ebe2c7f405f3c11aaa170df7b9007c32d8d21d68c1c2509004",
        "0a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c12080436afffd706357712080436afffd7061da41a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e303216676f2d697066732f302e31322e322f3065386231323142d3010a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c120203011a650a260024080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c10d3b2e5b29bbdc78a171a0a0a08047f000001060fa11a0d0a0b047f00000191020fa1cc031a0a0a080436afffd70635771a0a0a080436afffd7061da42a40abd31e8fc78fa9caa5feeb96cb3ab5c4bdfcbf832ce345f308cb4fe4c0811ff9ba4fef0dc630c1c4f1194a5b85940689055f6afc555ad08c769f0f168abecb00",
        "0a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c12080436afffd7061da412080436afffd706b8031a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a132f697066732f6c616e2f6b61642f312e302e301a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f2208042d4f59960627102a0a697066732f302e312e303216676f2d697066732f302e31322e322f3065386231323142d3010a24080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c120203011a650a260024080112206b30f0a74779f2a0761566c1059087459bf4b7c4f014e1cab8ca31f03aa6682c10b7c3cbdbbec4c78a171a0a0a08047f000001060fa11a0d0a0b047f00000191020fa1cc031a0a0a080436afffd7061da41a0a0a080436afffd706b8032a404651a0ae6aee2ee71731b198618feb50d2712189ab53be4a0dbcb7796c4980d31346e432c2a505cdc42c5e34c957dfd1a1889d2aea03a82feca86b9b7fd5400d",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e1302030100011a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242f1040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a3a0a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd10a2fab5f7a284cb8a171a0a0a08047f00000106c5a62a8002627f6034dca5ba81f710525b216fc150265d2ff551bf0e3045eff5050dd59786fdcf4717a0cd461836b481cfd41c45f245693dc6d093ebe032f346a859f28eefb04f79ee8131660c46186009f27215c20efe712488c051375311fbefdf5ba34f3b649b51879a7f695ac4078da62ea66659c5f5f67c0a4e39034361e823e99e691a63b796c230083c08e055817913b1fed311caecf7a54ced794d754b00ae5b088a71a75155a35ff635512884c37af04b984624aceb803d2cc0ab6ddf3e918511c272114ef7a8fa3fe95b5f8a490771bbbfa17344eb4af119034e246c607350c0f9f59a4949d730f4ffd8b3deedb5e8b8c351e0d655f9c09da90fb630be8af87d",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120804c0a8010906c5a61a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242fd040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a460a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd1080e388de9285cb8a171a0a0a0804c0a8010906c5a61a0a0a08047f00000106c5a62a800216a33aa4e437c4d25d3944d504bcbf49121404a4cfafd6663c47ff969b91a39fd1318ffecf5b4e510f6375c6aa989c8212b1ae187bf19563df221a2509426c14cbf82f7e4b74522b52e983d69bc7f62c758bfcc8ee631df87b9485243c22a71702595797b713adfedfd6a07711687d250d816bbbfe9ffe5411eec182a0902674fb56a09275606059300986015bd576962255899a75d2994024db599d4e5cdb91769892f0a0a5a5dd756286b4232cfc749aae952d4168b4723720cc69d123d62abe888c9ca915c3b77ef90deea1eb0829fb4e9d653ccd062ff294c14ced6657d49a942704a346ebf5ab5d081369fac35a2b6eca5ae22df2aa03822363118e6053",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e1302030100011a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242f1040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a3a0a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd1089f589cd8989cb8a171a0a0a08047f00000106c5a62a80020ae96d08757daf64a9662a0ced152d2a7acf787f5af4c19701e2ecc043a7718165bdfcaec53ee061209a238470112edd3293d405a2a51335c64c308c421828e176f07993cea37fb6478ba5dd997306aac2137926eb3d05e362518c7d1c9b4188686aefa7cf8eb6316c74c3ed1783974b6e57c84deb56135e27263e858b8ab5fbeea8caefb220c8c1f65d5df000e98b9fa2ac440662d0d110afc025488afb58c74f211373234ba365e8e2be12ea0216ae289818b2c67a9f94e0e7f52ed06c6a1733ac6707056d204a9a4b69dbf2499482cbb4f9c765ba193aae10b8fb2d13dfa5d06e249497a8ae7ff6d4b4202e89a9b2fe863cf81bc8ed1e2f84799d055a65dd",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120804c0a8010906c5a61a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242fd040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a460a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd10a0e5bfaff989cb8a171a0a0a0804c0a8010906c5a61a0a0a08047f00000106c5a62a80023fdfa74ecb14030a4ba60036a1fc24bcbebcc23206ce145a13a33239a84a15c11ced4295cf9f3c13335b3b31eab8acbbbc7ffe3aed2c9a4caa607ad74d18eea6a5b290187d98b4e45ff2bfa380142adee880157c10dc0744ef7a473f43c47cfd96b3f9114f9106f9523637ce72d3fef78d6bca982f8240425706a5be5ba6cffcd7ecaf967b945b2002d5547d03961967fc927cf6b9bad75a72bf5091f7c3a5c40dc961e40dac90e1deb794bb54a74a4245bbd98802401f32f2159da5e7fb2591a49681ad161784a735c98e2daf6c4cb4980b3719b5e85eadf498be992904b13c25b2109edb7cad4e0b9e793fd90f2a46af1e5dffcfe1511e50fc9c094126f9ca",
        "0a24080112200968974df76f67bc14ab47a34eaf5c13c851db0c5b8b0334f264a86ebd22130d123504889031c1060fa2dd03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a20212412926041380409157000000000000000021060fa2dd03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a202123f2926041380409157000000000000000021060fa1a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a202123604889031c191020fa1cc03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021242292604138040915700000000000000002191020fa1cc03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a202123304889031c1060fa1a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021233048ff4b13a060fa1a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021242292604a880000401d00000000003d9700091020fa1cc03a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021242290064ff9b00000000000000008ff4b13a91020fa1cc03a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a202123f292604a880000401d00000000003d97000060fa1a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021236048ff4b13a91020fa1cc03a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a202f6c69627032702f636972637569742f72656c61792f302e322e302f73746f701a152f6c69627032702f6175746f6e61742f312e302e301a132f697066732f626974737761702f312e322e301a132f697066732f626974737761702f312e312e301a132f697066732f626974737761702f312e302e301a0d2f697066732f626974737761701a032f782f1a0d2f6c69627032702f64637574722208042d4f59960627102a0a697066732f302e312e303218676f2d697066732f302e31332e312f38666663376138613642a7070a24080112200968974df76f67bc14ab47a34eaf5c13c851db0c5b8b0334f264a86ebd22130d120203011ab8060a260024080112200968974df76f67bc14ab47a34eaf5c13c851db0c5b8b0334f264a86ebd22130d10d5f3a9bd848dcb8a171a190a17290000000000000000000000000000000191020fa1cc031a0a0a08047f000001060fa11a160a142900000000000000000000000000000001060fa11a0d0a0b047f00000191020fa1cc031a370a3504889031c1060fa2dd03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021a430a412926041380409157000000000000000021060fa2dd03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021a410a3f2926041380409157000000000000000021060fa1a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021a380a3604889031c191020fa1cc03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021a440a42292604138040915700000000000000002191020fa1cc03a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021a350a3304889031c1060fa1a503260024080112202777ff4daa54d595a68763c542f905ddb40503f1d844dbd0d4bad457d9a6ffd4a2021a350a33048ff4b13a060fa1a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021a440a42292604a880000401d00000000003d9700091020fa1cc03a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021a440a42290064ff9b00000000000000008ff4b13a91020fa1cc03a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021a410a3f292604a880000401d00000000003d97000060fa1a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2021a380a36048ff4b13a91020fa1cc03a50326002408011220aecfd88d47fa8ba27e1dee5f7a7af6d1d93e3c12ddcc01dbe0d58afc15772346a2022a407ff96f971282f7fefc63b685fb14c978619d983dbe22b890605c8068a9447ce2d222e8e08eb4bea35bb02b19590a7a99a1ea93aed87ad81b9db328570b644807",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e1302030100011a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242f1040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a3a0a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd10e998fdd3e799cb8a171a0a0a08047f00000106c5a62a80024580ac0395538c7f1b3a1ec73bcbaf234c564a72cc61d0141db9d6b39b503cfface6e2a78a1451e32fb2159c8265cf3c5e1a5d58e6a40253802a93bbc52b6ad2fbfabd5183bd972a559bb3eaf889c24bb6af31bc87098869bafd7479c644f7a5a3b850642fe6b511b0f7063a43145815990a46cc417bccdf38b87f6847bb956325d0b82a4e751e5b522f196d4865be2b208698d0d08dfc7885d82ef5410b08c081c40bbb039a8bca3a572ac01ca19b92350f401f4b21f5db773dbdff2e298255b7310dc3e700f477ec261ad6ff2d75f32df1a8fa7c7e029556fa0731a972d5ae871119bf97428ac15112c19bf599954328a24bf6b35ff52f29a557535e14877a",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120804c0a8010906c5a61a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242fd040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a460a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd10c9b0aac8c99ccb8a171a0a0a0804c0a8010906c5a61a0a0a08047f00000106c5a62a8002a308f980dacbde477a7c1a4547bb95b9eea7b7d7086d4bc8c31d6c960e1ae2b0a09635e22fff25a329a08342970d82a11e6556575421d1716a9ba302e8221ef96ee8f88bdbb337cd0d7a86f08ca9424a51c9766703d9b90fc6ea2332773eb4de71a22b72f26b129973d1e93a5492c8660a8439f91b9e4bab55c64a365d358c1e34e7d4d1a4b54d29a3bb5624344de4cabdcfe135ff28e78275cfb47281a7183c52cff94a2e841e67291d4515548c7890247a60c5e6eb24223e7afb227530fea13d06640d824da28da8563205785b7630b71a59a8b447f3484947a3b27885d881a5f3df554edde73cdf2b32ce266bfb8514602a91a52367c55c8ef397736505c1",
        "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e1302030100011a132f7032702f69642f64656c74612f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a1b2f6c69627032702f636972637569742f72656c61792f302e312e301a152f6c69627032702f6175746f6e61742f312e302e301a0c2f73627074702f312e302e301a0b2f736673742f322e302e302208042d4f59960627102a0a697066732f302e312e303215676f2d697066732f302e382e302f3438663934653242f1040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100d6ffdc152b37deba1aef9cf544dc1ad21dad40da4a684047ba6f464ccc4228ac12c5adb0fc87056c14f75651bb13f388d60f66881b3c8f5477e323c17c0cf93d4fee1e31cbd9543715df38b879fa2c567200919b16e1410cb30e43c4b6ec4e41848e440e1e7960935dd58bc499c86353ae1ee21536a340e514160939102ed1cf6f1c27cc912d44245a6f951cc85f316e510b6fa754a32d2a505d9d3c2fd50348a531f4e085292a01c5278dc7f2312bc1a09e0d440a915b20b2e5c4fe469509b171e0695bd11fed5512b5dd3d2d5975e4a1c2e6b932a9860eb8996cf67fef1c83d8fe9e4c463d6b7e64afc07b961357684d2f837c616a7293d5b7e4e5b0de2e130203010001120203011a3a0a2212202faa4312eaf073954a17569e30601f84a9cb18d8f1b0f33ac2f747c1ba09e2cd10fbb7ae8bbe9fcb8a171a0a0a08047f00000106c5a62a8002810c0a861ad6c1d531b5aa473a791ab7d086b999e284973b57588a3df00328aa8b5ea22d48546c7acf033e861faf81039b85099dad62a3d98ee501f8f78ccf6e174314f2e6cf2ae0074740cd6db7f444514d7780f51dec321726e377c21c065adf6f9b28d8b4c520f8d324a9237ca8e577c0363b3a02d3ebefa069f7fa811eb8474a317e412ff35ac54edbc4f47fe9df5972882fdb4575665dd74f0a04be213ca89bb99b17b1dd6f4a140272becaac0c2620b2511258d9fc8331a40bbb420261133990fa1941d45429b0ecbe62574a44c28a98ac5d1fb3753480e6ebc40746bee2916c034b1ad7d759dafb92d6ea849fe9248d472b78de7befd322a4bd34eb35",
    ]
}
