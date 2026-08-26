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

import LibP2P
import LibP2PNoise
import LibP2PYAMUX
import Testing

@testable import LibP2PPubSub

@Suite("Libp2p PubSub Floodsub Tests", .timeLimit(.minutes(10)), .serialized)
final class LibP2PPubSubFloodsubTests {

    /// **************************************
    ///     Testing Internal Floodsub
    /// **************************************
    @Test(.timeLimit(.minutes(1)))
    func testLibP2PPubSub_FloodSub() async throws {
        /// Init the libp2p nodes
        let node1 = try makeHost()
        let node2 = try makeHost()

        /// Prepare our expectations
        let expectationNode1ReceivedNode2Subscription = AsyncSemaphore(value: 0)
        let expectationNode1ReceivedNode2Message = AsyncSemaphore(value: 0)
        let expectationNode2ReceivedNode1Subscription = AsyncSemaphore(value: 0)
        let expectationNode2ReceivedNode1Message = AsyncSemaphore(value: 0)

        let node1Message = "banana"
        let node2Message = "pineapple"

        /// Node1 subscribes to topic 'fruit'
        let subscription1 = try node1.pubsub.floodsub.subscribe(
            .init(
                topic: "fruit",
                signaturePolicy: .strictSign,
                validator: .acceptAll,
                messageIDFunc: .hashSequenceNumberAndFromFields
            )
        )
        subscription1.on = { event -> EventLoopFuture<Void> in
            switch event {
            case .newPeer(let peer):
                node1.logger.info("Node1::NewPeer -> \(peer)")
                #expect(peer == node2.peerID)
                expectationNode1ReceivedNode2Subscription.signal()

            case .data(let pubSubMessage):
                node1.logger.info("Node1 -> \(String(data: pubSubMessage.data, encoding: .utf8) ?? "NIL")")
                #expect(String(data: pubSubMessage.data, encoding: .utf8) == node2Message)
                expectationNode1ReceivedNode2Message.signal()

            case .error(let error):
                node1.logger.error("Node1 Error: \(error)")
                Issue.record(error)
            }
            return node1.eventLoopGroup.next().makeSucceededVoidFuture()
        }

        /// Node2 subcribes to topic 'fruit'
        //let subscription2 = try fSub2.subscribe(topic: "fruit")
        let subscription2 = try node2.pubsub.floodsub.subscribe(
            .init(
                topic: "fruit",
                signaturePolicy: .strictSign,
                validator: .acceptAll,
                messageIDFunc: .hashSequenceNumberAndFromFields
            )
        )
        subscription2.on = { event -> EventLoopFuture<Void> in
            switch event {
            case .newPeer(let peer):
                node2.logger.info("Node2::NewPeer -> \(peer)")
                #expect(peer == node1.peerID)
                expectationNode2ReceivedNode1Subscription.signal()

            case .data(let pubSubMessage):
                node2.logger.info("Node2 -> \(String(data: pubSubMessage.data, encoding: .utf8) ?? "NIL")")
                #expect(String(data: pubSubMessage.data, encoding: .utf8) == node1Message)
                expectationNode2ReceivedNode1Message.signal()

            case .error(let error):
                node2.logger.error("Node2 Error: \(error)")
                Issue.record(error)
            }
            return node2.eventLoopGroup.next().makeSucceededVoidFuture()
        }

        /// Start the libp2p nodes
        try await node1.startup()
        try await node2.startup()

        do {
            try await Task.sleep(for: .seconds(1))

            /// Have node1 reach out to node2
            try node1.newStream(
                to: node2.listenAddresses.first!.encapsulate(proto: .p2p, address: node2.peerID.b58String),
                forProtocol: "/floodsub/1.0.0"
            )

            /// Publish some messages...
            node1.eventLoopGroup.next().scheduleTask(in: .seconds(1)) {
                print("Node 1 Publishing Message")
                subscription1.publish(node1Message.data(using: .utf8)!)
            }
            node2.eventLoopGroup.next().scheduleTask(in: .seconds(2)) {
                print("Node 2 Publishing Message")
                subscription2.publish(node2Message.data(using: .utf8)!)
            }

            // Wait for our sequence of events to trigger
            await expectationNode1ReceivedNode2Subscription.wait()
            await expectationNode1ReceivedNode2Message.wait()
            await expectationNode2ReceivedNode1Subscription.wait()
            await expectationNode2ReceivedNode1Message.wait()

            /// Check to see if we can poll our PeerStore for known peers that support '/floodsub/1.0.0'
            let peers = try await node1.peers.getPeers(supportingProtocol: SemVerProtocol("/floodsub/1.0.0")!, on: nil)
                .get()
            #expect(peers.count == 1)
            #expect(peers.first == node2.peerID.b58String)

            /// Dump the current state of our PeerStore
            node1.peers.dumpAll()
        } catch {
            Issue.record(error)
        }

        /// Stop the nodes
        try await node1.asyncShutdown()
        try await node2.asyncShutdown()

        print("All Done!")
    }

    /// **************************************
    ///     Testing Internal Floodsub Subscriptions
    /// **************************************
    /// Node 1 publishes a continous stream of news
    /// Node 2 Subscribes / Unsubscribes periodically
    /// We assert Node 1 stops sending messages while Node 2 is unsubscribed
    /// We ensure Node 1 updates the subscription changes appropriately
    @Test(.timeLimit(.minutes(1)))
    func testLibP2PPubSub_FloodSub_Subscriptions() async throws {
        /// Init the libp2p nodes
        let node1 = try makeHost()
        let node2 = try makeHost()

        /// Prepare our expectations
        let expectationNode1ReceivedNode2Subscription = AsyncSemaphore(value: 0)
        //let expectationNode1ReceivedNode2Unsubscription = AsyncSemaphore(value: 0)
        let expectationNode1ReceivedNode2SecondSubscription = AsyncSemaphore(value: 0)

        let expectationNode2ReceivedNode1Subscription = AsyncSemaphore(value: 0)
        let expectationNode2ReceivedFirstNode1Message = AsyncSemaphore(value: 0)
        let expectationNode2ReceivedSecondNode1Message = AsyncSemaphore(value: 0)

        let node1Message = "hot news!"
        let subscriptionConfig = PubSub.SubscriptionConfig(
            topic: "news",
            signaturePolicy: .strictSign,
            validator: .acceptAll,
            messageIDFunc: .hashSequenceNumberAndFromFields
        )

        var node2SubscriptionCount = 0
        /// Node1 subscribes to topic 'fruit'
        let subscription1 = try node1.pubsub.floodsub.subscribe(subscriptionConfig)
        subscription1.on = { event -> EventLoopFuture<Void> in
            switch event {
            case .newPeer(let peer):
                node1.logger.info("Node1::NewPeer -> \(peer)")
                #expect(peer == node2.peerID)
                node2SubscriptionCount += 1
                if node2SubscriptionCount == 1 {
                    expectationNode1ReceivedNode2Subscription.signal()
                } else if node2SubscriptionCount == 2 {
                    expectationNode1ReceivedNode2SecondSubscription.signal()
                }

            case .data(let pubSubMessage):
                node1.logger.info("Node1 -> \(pubSubMessage)")
                Issue.record("Node 1 shouldn't receive data during this test")

            case .error(let error):
                node1.logger.error("Node1 Error: \(error)")
                Issue.record(error)
            }
            return node1.eventLoopGroup.next().makeSucceededVoidFuture()
        }

        /// Node2 subcribes to topic 'news'
        //let subscription2 = try fSub2.subscribe(topic: "fruit")
        var node2MessageCount = 0
        let messagesPerBatch = 2
        let subscriptionHandler: (PubSub.SubscriptionEvent) -> EventLoopFuture<Void> = {
            event -> EventLoopFuture<Void> in
            switch event {
            case .newPeer(let peer):
                node2.logger.info("Node2::NewPeer -> \(peer)")
                #expect(peer == node1.peerID)
                expectationNode2ReceivedNode1Subscription.signal()

            case .data(let pubSubMessage):
                node2.logger.info("Node2 -> \(String(data: pubSubMessage.data, encoding: .utf8) ?? "NIL")")
                #expect(String(data: pubSubMessage.data, encoding: .utf8) == node1Message)
                node2MessageCount += 1
                if node2MessageCount == messagesPerBatch {
                    expectationNode2ReceivedFirstNode1Message.signal()
                } else if node2MessageCount == messagesPerBatch * 2 {
                    expectationNode2ReceivedSecondNode1Message.signal()
                }

            case .error(let error):
                node2.logger.error("Node2 Error: \(error)")
                Issue.record(error)
            }
            return node2.eventLoopGroup.next().makeSucceededVoidFuture()
        }

        var subscription2 = try node2.pubsub.floodsub.subscribe(subscriptionConfig)
        subscription2.on = subscriptionHandler

        /// Start the libp2p nodes
        try await node1.startup()
        try await node2.startup()

        do {
            try await Task.sleep(for: .seconds(1))

            /// Have node2 reach out to node1
            //try node2.newStream(to: node1.peerInfo, forProtocol: FloodSub.multicodec)
            try node1.newStream(to: node2.peerInfo, forProtocol: FloodSub.multicodec)

            /// Publish some messages...
            let repeatedTask = node1.eventLoopGroup.next().scheduleRepeatedTask(
                initialDelay: .milliseconds(50),
                delay: .seconds(1)
            ) { task in
                subscription1.publish(node1Message.data(using: .utf8)!)
            }

            /// Wait for initial subscription alerts and the first message to arrive on Node 2
            await expectationNode1ReceivedNode2Subscription.wait()
            await expectationNode2ReceivedNode1Subscription.wait()
            await expectationNode2ReceivedFirstNode1Message.wait()

            /// Unsubscribe Node2 from our `news` subscription
            //try node2.pubsub.floodsub.unsubscribe(topic: "news").wait()
            subscription2.unsubscribe()

            //wait(for: [expectationNode1ReceivedNode2Unsubscription], timeout: 10, enforceOrder: false)

            sleep(1)

            /// Re subscribe Node2 to our `news` subscription
            subscription2 = try node2.pubsub.floodsub.subscribe(subscriptionConfig)
            subscription2.on = subscriptionHandler

            /// Wait for the second subscription alert on Node1 and the second `news` message to arrive at Node2
            await expectationNode1ReceivedNode2SecondSubscription.wait()
            await expectationNode2ReceivedSecondNode1Message.wait()

            try await node2.pubsub.floodsub.unsubscribe(topic: "news").get()

            repeatedTask.cancel()

            sleep(1)

            /// Check to see if we can poll our PeerStore for known peers that support '/chat/1.0.0'
            let peers = try await node1.peers.getPeers(
                supportingProtocol: SemVerProtocol(FloodSub.multicodec)!,
                on: nil
            )
            .get()
            #expect(peers.count == 1)
            #expect(peers.first == node2.peerID.b58String)

            /// Ensure Node1 Subscription count equals 2 (Node2 subscribed twice)
            #expect(node2SubscriptionCount == 2)
            /// Ensure the Node2 received the appropriate number of `news` messages
            #expect(node2MessageCount == messagesPerBatch * 2)
        } catch {
            Issue.record(error)
        }

        /// Stop the nodes
        try await node1.asyncShutdown()
        try await node2.asyncShutdown()

        print("All Done!")
    }

    enum NetworkStructure: CaseIterable {
        case linear
        case circular
        case beacon
        case beacon2beacon
    }

    /// **************************************
    ///  Testing FloodSub Message Propogation
    /// **************************************
    /// This test launches the specified number of nodes, registers floodsub and subscribes to the 'fruit' topic.
    /// We then itterate through the nodes and connect each one to exactly one other using various structures
    /// 1) Linear chain grouping - each node knows about the node ahead of it in our array and thats it (messages are propogated through the array in a linear fashion)
    /// 2) Beacon Grouping - each node is linked to the first node (index 0) and only the first node. (messages are beaconed out of the main node)
    /// 3) TODO: Dense Multi Cluster with sparse connections between them. (10 nodes, first 5 are tightly interconnected, second 5 are tightly interconnected, node 0 and 5 bridge the clusters)
    /// Each node then publishes a message and we wait / ensure that all nodes subscribed to the topic receive all messages.
    /// - Note: OLD  10 Interconnected Nodes results in about 19mb of ram (both Plaintext and Noise), 20 Nodes -> 28mb
    /// - Note: NEW 10 Interconnected Nodes results in about 28mb of ram (both Plaintext and Noise), 20 Nodes -> 46.4mb, 50 Nodes -> 100mb
    @Test(.timeLimit(.minutes(2)), arguments: NetworkStructure.allCases)
    func testLibP2PPubSub_FloodSub_NNodes(_ structureToTest: NetworkStructure) async throws {

        class Node {
            let libp2p: Application
            let expectation: AsyncSemaphore
            let tag: String
            let messageToSend: String
            var messagesReceived: [String]
            var handler: PubSub.SubscriptionHandler!

            init(libp2p: Application, expectation: AsyncSemaphore, tag: String, messageToSend: String) {
                self.libp2p = libp2p
                self.expectation = expectation
                self.tag = tag
                self.messageToSend = messageToSend
                self.messagesReceived = [messageToSend]
                self.handler = nil
            }
        }

        /// Consider the ConenctionManagers max concurrent connections param while setting this number (especially for the beacon structure) (the default is 25 connections)
        let nodesToTest: Int = 5

        guard nodesToTest > 2 else {
            Issue.record("We need at least 3 nodes to accurately perform this test...")
            return
        }

        /// Init the libp2p nodes, floodsub routers, and prepare our expectations
        var nodes: [Node] = try (0..<nodesToTest).map { idx in
            let node = try makeHost()
            node.connectionManager.use(connectionType: BasicConnectionLight.self)
            return Node(
                libp2p: node,
                expectation: AsyncSemaphore(value: 0),
                tag: "Node\(idx) received all messages",
                messageToSend: "Hello from node \(idx) 🍌"
            )
        }

        /// For each node, subscribe to the topic 'fruit' and register our on 'fruit' subscription handler...
        for node in nodes {
            /// Subscribe to topic using a SubscriptionConfiguration (this lets us define our subscription behavior on a per topic level)
            node.handler = try node.libp2p.pubsub.floodsub.subscribe(
                PubSub.SubscriptionConfig(
                    topic: "fruit",
                    signaturePolicy: .strictSign,
                    validator: .acceptAll,
                    messageIDFunc: .hashSequenceNumberAndFromFields
                )
            )
            node.handler.on = { event -> EventLoopFuture<Void> in
                switch event {
                case .newPeer(let peer):
                    node.libp2p.logger.debug("Node[\(node.libp2p.peerID)]::NewPeer -> \(peer)")

                case .data(let pubSubMessage):
                    node.libp2p.logger.debug("Node[\(node.libp2p.peerID)]::Data -> \(pubSubMessage)")
                    node.messagesReceived.append(String(data: pubSubMessage.data, encoding: .utf8)!)
                    if node.messagesReceived.count == nodesToTest {
                        node.expectation.signal()
                    }

                case .error(let error):
                    node.libp2p.logger.error("Node[\(node.libp2p.peerID)]::Error -> \(error)")
                    Issue.record(error)
                }
                return node.libp2p.eventLoopGroup.next().makeSucceededVoidFuture()
            }
        }

        /// Start the libp2p nodes
        for node in nodes {
            try await node.libp2p.startup()
        }

        print("Structuring Peers - \(structureToTest)")

        do {
            /// ******************************************
            /// The following logic determines the structure of the network
            /// ******************************************
            switch structureToTest {
            case .linear:
                /// Have each node reach out to the next node in our array... (a linear chain set up)
                ///
                /// Network Structure Diagram
                ///  n -> ... -> n
                for (idx, node) in nodes.enumerated() {
                    guard nodes.count > (idx + 1) else { continue }
                    let nextPeerInfo = nodes[idx + 1].libp2p.peerInfo
                    try? node.libp2p.newStream(
                        to: nextPeerInfo,
                        forProtocol: FloodSub.multicodec
                    )
                }

            case .circular:
                /// If we tie the ends together, we have a circular network graph
                /// - Note: with 3 Nodes, this results in 6 wasted / redundant message propogations, 7 Nodes -> 14 redundant messages...
                /// Network Structure Diagram
                ///  n -> ... -> n -,
                ///  ^                  |
                ///  '----------------'
                for (idx, node) in nodes.enumerated() {
                    guard nodes.count > (idx + 1) else {
                        try node.libp2p.newStream(
                            to: nodes[0].libp2p.peerInfo,
                            forProtocol: FloodSub.multicodec
                        )
                        continue
                    }
                    try node.libp2p.newStream(
                        to: nodes[idx + 1].libp2p.peerInfo,
                        forProtocol: FloodSub.multicodec
                    )
                }

            case .beacon:
                /// Have each node reach out to the zeroeth node (a beacon set up)
                ///
                /// Network Structure Diagram
                ///  n
                ///  : \
                ///     -- n
                ///  : /
                ///  n
                ///
                for (idx, node) in nodes.enumerated() {
                    guard idx != 0 else { continue }
                    try node.libp2p.newStream(to: nodes[0].libp2p.peerInfo, forProtocol: FloodSub.multicodec)
                }

            case .beacon2beacon:
                /// Splits the network into Evens & Odds then connects node 0 and 1 to bridge the devide...
                ///
                /// Network Structure Diagram
                ///  n               n
                ///    \          /
                ///  n -- n -- n -- n
                ///    /          \
                ///  n               n
                ///
                for (idx, node) in nodes.enumerated() {
                    guard idx != 0 else { continue }
                    if idx == 1 {
                        /// Have Node1 reach out to Node0
                        try node.libp2p.newStream(
                            to: nodes[0].libp2p.peerInfo,
                            forProtocol: FloodSub.multicodec
                        )
                        continue
                    }
                    if idx % 2 == 0 {
                        /// If the node is an even number (have it reach out to Node0, our even beacon node)
                        try node.libp2p.newStream(
                            to: nodes[0].libp2p.peerInfo,
                            forProtocol: FloodSub.multicodec
                        )
                    } else {
                        /// Otherwise the node must be odd (have it reach out to Node1, our odd beacon node)
                        try node.libp2p.newStream(
                            to: nodes[1].libp2p.peerInfo,
                            forProtocol: FloodSub.multicodec
                        )
                    }
                }
            }

            /// Publish some messages...
            for node in nodes {
                node.libp2p.eventLoopGroup.next().scheduleTask(in: .milliseconds(Int64.random(in: 500...2_000))) {
                    //node.5!.publish(node.3.data(using: .utf8)!)
                    node.libp2p.pubsub.publish(node.messageToSend.data(using: .utf8)!.byteArray, toTopic: "fruit")
                }
            }

            /// Wait for each node to receive each message
            for node in nodes {
                await node.expectation.wait()
            }
            //waitForExpectations(timeout: 10)

            /// Wait an additional 2 seconds to ensure message propogation doesn't echo through the network causing duplicates
            try await Task.sleep(for: .seconds(2))

            nodes.first!.libp2p.peers.dumpAll()

            /// Close all connections
            for node in nodes {
                try? await node.libp2p.connections.closeAllConnections().get()
            }
        } catch {
            Issue.record(error)
        }

        /// Stop the nodes
        for node in nodes { try await node.libp2p.asyncShutdown() }

        /// Ensure that each node received every message...
        for node in nodes {
            #expect(node.messagesReceived.count == nodes.count)
            for i in (0..<nodesToTest) {
                #expect(node.messagesReceived.contains("Hello from node \(i) 🍌"))
            }
            print("Node[\(node.libp2p.peerID)] Messages Received:")
            print(node.messagesReceived)
        }

        /// Remove all nodes and ensure memory is released....
        nodes.removeAll()

        print("All Done!")
    }

    /// **************************************
    ///      Testing JS Interoperability
    /// **************************************
    /// In order to run this example, launch the js-libp2p/examples/pubsub example then connect to one of the two peers that example spawns
    /// - Note: Using a shell / terminal window execute the following commands...
    /// ```
    /// git clone https://github.com/libp2p/js-libp2p.git //if you dont have the js-libp2p repo yet
    /// cd js-libp2p
    /// npm install
    /// cd examples
    /// npm install
    /// cd pubsub
    /// node 1.js
    /// // copy the port number of the peer and place it in the `newStream` call below
    /// ```
    @Test(.externalIntegrationTestsEnabled, .timeLimit(.minutes(1)))
    func testFloodsubJSInterop() async throws {
        let app = try await Application.make(.testing, peerID: PeerID(.Ed25519))
        app.logger.logLevel = .trace
        app.environment.arguments = [app.environment.arguments.first ?? "xctest"]

        /// Configure our networking stack!
        app.servers.use(.tcp(host: "127.0.0.1", port: 10000))
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.floodsub)

        let topic = "news"
        var expectedMessageCount = 5
        let messageExpectation = AsyncSemaphore(value: 0)

        try await app.startup()

        let subscription = try app.pubsub.floodsub.subscribe(
            .init(
                topic: topic,
                signaturePolicy: .strictSign,
                validator: .acceptAll,
                messageIDFunc: .hashSequenceNumberAndFromFields
            )
        )
        subscription.on = { event -> EventLoopFuture<Void> in
            switch event {
            case .newPeer(let peer):
                print("[\(peer)]: Subscribed to \(topic)!")
                app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) {
                    app.pubsub.publish("Hello from swift!".data(using: .utf8)!.byteArray, toTopic: topic)
                }

            case .data(let pubSubMessage):
                print(String(data: pubSubMessage.data, encoding: .utf8) ?? "NIL")
                expectedMessageCount -= 1
                if expectedMessageCount == 0 {
                    messageExpectation.signal()
                }

            case .error(let error):
                app.logger.error("Error: \(error)")
            }
            return app.eventLoopGroup.next().makeSucceededVoidFuture()
        }

        try? app.newStream(to: Multiaddr("/ip4/192.168.1.19/tcp/51249"), forProtocol: "/ipfs/ping/1.0.0")

        await messageExpectation.wait()

        let _ = app.pubsub.publish("Goodbyte from swift!".data(using: .utf8)!.byteArray, toTopic: topic)

        subscription.unsubscribe()

        print("Shutting down libp2p chat...")
        app.peers.dumpAll()

        try await app.asyncShutdown()
    }

    @Test(.externalIntegrationTestsEnabled, .timeLimit(.minutes(1)))
    func testExternalPing() async throws {
        let app = try await Application.make(.testing, peerID: PeerID(.Ed25519))
        app.logger.logLevel = .trace
        app.environment.arguments = [app.environment.arguments.first ?? "xctest"]

        /// Configure our networking stack!
        app.servers.use(.tcp(host: "127.0.0.1", port: 10000))
        app.security.use(.noise)
        app.muxers.use(.yamux)

        try await app.startup()

        // No MPLEX Support
        let peerToDial = try Multiaddr("/ip4/139.178.91.71/tcp/4001/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN")

        let latency = try await app.identify.ping(addr: peerToDial)
        print("Latency: \(latency)")

        try await Task.sleep(for: .milliseconds(50))

        app.peers.dumpAll()

        try await app.asyncShutdown()
    }

    @Test(.externalIntegrationTestsEnabled, .timeLimit(.minutes(1)))
    func testExternalFloodsubConnections() async throws {
        let app = try await Application.make(.testing, peerID: PeerID(.Ed25519))
        app.logger.logLevel = .trace
        app.environment.arguments = [app.environment.arguments.first ?? "xctest"]

        /// Configure our networking stack!
        app.servers.use(.tcp(host: "127.0.0.1", port: 10000))
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.floodsub)

        let topic = "news"
        //let topic = "/orbitdb/zdpuAxoBaqad71bpU92QUkkMvtZt279URxKEh9ab9DNvdPZi9/capsule-orbit-production-2-posts"
        //let topic = "/ipfs-pubsub-direct-channel/v1/\(app.peerID.b58String)/12D3KooWBidnLf4iRGgZpeFVCqQjNzAsSx2opZPbG8o9tpCf2rG5"
        var expectedMessageCount = 1
        let messageExpectation = AsyncSemaphore(value: 0)

        try await app.startup()

        let subscription = try app.pubsub.floodsub.subscribe(
            .init(
                topic: topic,
                signaturePolicy: .strictSign,
                validator: .acceptAll,
                messageIDFunc: .hashSequenceNumberAndFromFields
            )
        )
        subscription.on = { event -> EventLoopFuture<Void> in
            switch event {
            case .newPeer(let peer):
                print("[\(peer)]: Subscribed to \(topic)!")
                app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) {
                    app.pubsub.publish("Hello from swift!".data(using: .utf8)!.byteArray, toTopic: topic)
                }

            case .data(let pubSubMessage):
                print(String(data: pubSubMessage.data, encoding: .utf8) ?? "NIL")
                expectedMessageCount -= 1
                if expectedMessageCount == 0 {
                    messageExpectation.signal()
                }

            case .error(let error):
                app.logger.error("Error: \(error)")
            }
            return app.eventLoopGroup.next().makeSucceededVoidFuture()
        }

        // These are all outdated
        let peerToDial = try Multiaddr(
            "/ip4/20.80.20.28/tcp/4001/p2p/12D3KooWH2jndcSD6MC7cvs5zJNfMgHJFBc8zpebNS3L2HGXvQnS"
        )
        //let peerToDial = try Multiaddr("/ip4/23.239.22.148/tcp/4001/p2p/12D3KooWBidnLf4iRGgZpeFVCqQjNzAsSx2opZPbG8o9tpCf2rG5")
        //let peerToDial = try Multiaddr("/ip4/139.178.88.229/tcp/4001/p2p/12D3KooWK3rWCYssQkQHHm5q1K1qHUBRgmEp18sHDnxRRtL5kPsb")

        let latency = try await app.identify.ping(addr: peerToDial)
        print("Latency: \(latency)")

        //await messageExpectation.wait()
        try await Task.sleep(for: .seconds(10))

        let _ = app.pubsub.publish("Goodbyte from swift!".data(using: .utf8)!.byteArray, toTopic: topic)

        subscription.unsubscribe()

        print("Shutting down libp2p chat...")
        app.peers.dumpAll()

        try await app.asyncShutdown()
    }

    var nextPort: Int = 10200
    private func makeHost() throws -> Application {
        let lib = try Application(.testing, peerID: PeerID(.Ed25519))
        lib.environment.arguments = [lib.environment.arguments.first ?? "xctest"]
        lib.connectionManager.use(connectionType: BasicConnectionLight.self)
        lib.logger.logLevel = .info
        lib.security.use(.noise)
        lib.muxers.use(.yamux)
        lib.pubsub.use(.floodsub)
        lib.servers.use(.tcp(host: "127.0.0.1", port: nextPort))

        nextPort += 1

        return lib
    }

}
