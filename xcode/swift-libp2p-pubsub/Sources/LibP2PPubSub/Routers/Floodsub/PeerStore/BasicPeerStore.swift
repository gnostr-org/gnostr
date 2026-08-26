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

final class BasicPeerState: PeerStateProtocol, @unchecked Sendable {

    typealias Topic = String
    typealias PID = String
    //typealias Subscriber = (id:PeerID, inbound:LibP2P.Stream?, outbound:LibP2P.Stream?)

    var state: ServiceLifecycleState

    /// A set of ids of all known peers that support floodsub.
    var peers: [PID: PubSub.Subscriber]
    /// A map of subscribed topics to the set of peers in our overlay mesh for that topic.
    var mesh: [Topic: [PID]]
    /// Like mesh, fanout is a map of topics to a set of peers, however, the fanout map contains topics to which we are NOT subscribed.
    var fanout: [Topic: [PID]]

    /// The eventloop that this PeeringState is constrained to
    internal let eventLoop: EventLoop
    /// Our Logger
    private var logger: Logger

    required init(eventLoop: EventLoop) {
        print("PubSub::PeeringState Instantiated...")
        self.eventLoop = eventLoop
        self.logger = Logger(label: "com.swift.libp2p.pubsub.pstate[\(UUID().uuidString.prefix(5))]")
        self.logger.logLevel = .trace  // LOG_LEVEL
        self.state = .stopped

        /// Initialize our caches
        self.peers = [:]
        self.mesh = [:]
        self.fanout = [:]
    }

    func start() throws {
        guard self.state == .stopped else { throw BasePubSub.Errors.alreadyRunning }
        self.logger.info("Starting")

        // Do stuff here, maybe re init our caches??
        self.state = .started
    }

    func stop() throws {
        guard self.state == .started || self.state == .starting else { throw BasePubSub.Errors.alreadyStopped }
        if self.state == .stopping {
            self.logger.info("Force Quiting!")
        }
        self.logger.info("Stopping")

        // Do stuff here, maybe clear our caches??

        self.state = .stopped
    }

    func onPeerConnected(peerID peer: PeerID, stream: LibP2PCore.Stream) -> EventLoopFuture<Void> {
        eventLoop.submit {
            if self.peers[peer.b58String] == nil {
                switch stream.direction {
                case .inbound:
                    self.peers[peer.b58String] = .init(id: peer, inbound: stream)
                case .outbound:
                    self.peers[peer.b58String] = .init(id: peer, outbound: stream)
                }
                self.logger.info("Added \(peer) to our peering state")
            } else {
                switch stream.direction {
                case .inbound:
                    self.peers[peer.b58String]?.attachInbound(stream: stream)
                case .outbound:
                    self.peers[peer.b58String]?.attachOutbound(stream: stream)
                }
                self.logger.warning(
                    "Received a peer connected event for a peer that was already present in our PeeringState"
                )
            }
        }
    }

    func attachInboundStream(
        _ peerID: PeerID,
        inboundStream: LibP2PCore.Stream,
        on loop: EventLoop? = nil
    ) -> EventLoopFuture<Void> {
        eventLoop.submit {
            self.peers[peerID.b58String, default: .init(id: peerID)].attachInbound(stream: inboundStream)
            //            if self.peers[peerID.b58String] == nil {
            //                //Add the new peer to our `peers` list
            //                self.peers[peerID.b58String] = .init(id: peerID, inbound: inboundStream)
            //                self.logger.info("Added \(peerID) to our peering state (peers2)")
            //            } else {
            //                self.peers[peerID.b58String]?.attachInbound(stream: inboundStream)
            //                //self.logger.warning("Received a peer connected event for a peer that was already present in our PeeringState")
            //            }
        }.hop(to: loop ?? self.eventLoop)
    }

    func attachOutboundStream(
        _ peerID: PeerID,
        outboundStream: LibP2PCore.Stream,
        on loop: EventLoop? = nil
    ) -> EventLoopFuture<Void> {
        eventLoop.submit {
            self.peers[peerID.b58String, default: .init(id: peerID)].attachOutbound(stream: outboundStream)
            //            if self.peers[peerID.b58String] == nil {
            //                //Add the new peer to our `peers` list
            //                self.peers[peerID.b58String] = .init(id: peerID, outbound: outboundStream)
            //                self.logger.info("Added \(peerID) to our peering state (peers2)")
            //            } else {
            //                self.peers[peerID.b58String]?.attachOutbound(stream: outboundStream)
            //                //self.logger.warning("Received a peer connected event for a peer that was already present in our PeeringState")
            //            }
        }.hop(to: loop ?? self.eventLoop)
    }

    func detachInboundStream(_ peerID: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit {
            self.peers[peerID.b58String]?.detachInboundStream()
        }.hop(to: loop ?? self.eventLoop)
    }

    func detachOutboundStream(_ peerID: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit {
            self.peers[peerID.b58String]?.detachOutboundStream()
        }.hop(to: loop ?? self.eventLoop)
    }

    func onPeerDisconnected(_ peer: PeerID) -> EventLoopFuture<Void> {
        eventLoop.submit {
            self.peers.removeValue(forKey: peer.b58String)
        }
    }

    /// Adds a new peer (who supports our base PubSub protocol (aka floodsub / gossipsub)) to the peers cache
    func addNewPeer(_ peer: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            if self.peers[peer.b58String] == nil {
                self.peers[peer.b58String] = .init(id: peer)
                self.logger.info("Added \(peer) to our peering state")
                return true
            } else {
                self.logger.warning(
                    "Received a peer connected event for a peer that was already present in our PeeringState"
                )
                return false
            }
        }.hop(to: loop ?? self.eventLoop)
    }

    /// Removes the specified peer from our peers cache
    func removePeer(_ peer: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit {
            self.peers.removeValue(forKey: peer.b58String)
        }.hop(to: loop ?? self.eventLoop)
    }

    /// This is called when we receive an RPC message from a peer containing the topics
    func update(topics: [Topic], for peer: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit {
            let pid = peer.b58String
            for topic in topics {
                // if we're subscribed to topic, ensure that this peer is in our mesh cache
                if var subs = self.mesh[topic] {
                    if !subs.contains(pid) {
                        subs.append(pid)
                        self.mesh[topic] = subs
                    }
                } else {  // add the (topic:peer) entry to our fanout cache
                    if var subs = self.fanout[topic] {
                        /// Add the peer to the existing topic entry...
                        if !subs.contains(pid) {
                            subs.append(pid)
                            self.fanout[topic] = subs
                        }
                    } else {
                        /// Create a new topic entry...
                        self.fanout[topic] = [pid]
                    }
                }
            }
        }.hop(to: loop ?? self.eventLoop)
    }

    /// This is called when we receive an RPC message from a peer containing the topics
    func update(subscriptions: [Topic: Bool], for peer: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit {
            let pid = peer.b58String
            for (topic, subscribed) in subscriptions {
                if subscribed == true {
                    // if we're subscribed to topic, ensure that this peer is in our mesh cache
                    if var subs = self.mesh[topic] {
                        if !subs.contains(pid) {
                            subs.append(pid)
                            self.mesh[topic] = subs
                            self.logger.trace("Added \(peer) to mesh[\(topic)] cache")
                        }
                    } else {  // add the (topic:peer) entry to our fanout cache
                        if var subs = self.fanout[topic] {
                            /// Add the peer to the existing topic entry...
                            if !subs.contains(pid) {
                                subs.append(pid)
                                self.fanout[topic] = subs
                                self.logger.trace("Added \(peer) to existing fanout[\(topic)] cache")
                            }
                        } else {
                            /// Create a new topic entry...
                            self.fanout[topic] = [pid]
                            self.logger.trace("Added \(peer) to new fanout[\(topic)] cache")
                        }
                    }
                } else {  // Unregister this PID from our fanout and mesh for the specified topic
                    if let subs = self.mesh[topic], subs.contains(pid) {
                        self.mesh[topic]?.removeAll(where: { $0 == pid })
                        self.logger.trace("Removed \(peer) from mesh[\(topic)] cache")
                    }
                    if let subs = self.fanout[topic], subs.contains(pid) {
                        self.fanout[topic]?.removeAll(where: { $0 == pid })
                        self.logger.trace("Removed \(peer) from fanout[\(topic)] cache")
                    }
                }
            }
            self.logger.info("Updated subscriptions for \(peer)")
        }.hop(to: loop ?? self.eventLoop)
    }

    func topicSubscriptions(on loop: EventLoop? = nil) -> EventLoopFuture<[Topic]> {
        eventLoop.submit { () -> [Topic] in
            self.mesh.map { $0.key }
        }.hop(to: loop ?? eventLoop)
    }

    /// This method updates our PeerState to reflect a new subscription
    ///
    /// It will...
    /// - Create a new entry in our Subscription Mesh for the specified topic
    /// - Bootstrap the new entry with any known peers that also subscribe to the topic
    /// Returns a list of PeerIDs that can be used to send grafting messages to
    func subscribeSelf(to topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<[PID]> {
        eventLoop.submit { () -> [PID] in
            /// Make sure we're not already subscribed...
            if let peers = self.mesh[topic] { return peers }

            /// Check to see if we're aware of the topic (is it in our fanout set)
            if let knownTopic = self.fanout.removeValue(forKey: topic) {
                self.logger.trace("Upgrading `\(topic)` subscription from fanout to mesh")
                /// Copy the topic/peer entry over to our subscribed mesh (are we allowed to do this? Or do we need to wait for RPC messages from peers to add them to our mesh set)
                self.mesh[topic] = knownTopic
                return knownTopic
            } else {
                self.logger.trace("Subscribing self to `\(topic)`")
                /// This is a new topic that we're not aware of, so make an empty entry
                self.mesh[topic] = []
                return []
            }
        }
    }

    /// This method updates our PeerState to reflect a subscription removal
    ///
    /// It will remove the latest known peer subscription state from our Subcription Mesh and transfer
    /// that state into our fanout set for future reference.
    /// Returns a list of PeerIDs that can be used to send unsub messages to
    func unsubscribeSelf(from topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<[PID]> {
        eventLoop.submit { () -> [PID] in
            guard self.state == .started || self.state == .stopping else { return [] }
            /// Check to see if we're aware of the topic (is it in our fanout set)
            if let knownTopic = self.mesh.removeValue(forKey: topic) {
                self.logger.trace("Downgrading `\(topic)` subscription from mesh to fanout")
                // Should we transfer this entry back to our fanout set?
                self.fanout.updateValue(knownTopic, forKey: topic)
                /// return the list of peers that are effected by this unsubing
                return knownTopic
            } else {
                self.logger.trace("Unsubscribing self from `\(topic)`")
            }

            return []
        }
    }

    func metaPeerIDs(on loop: EventLoop? = nil) -> EventLoopFuture<[Topic: [PeerID]]> {
        eventLoop.submit { () -> [Topic: [PeerID]] in
            var metaPeers: [Topic: [PeerID]] = [:]
            for (topic, pids) in self.fanout {
                metaPeers[topic] = self.idsToPeers(pids)
            }
            return metaPeers
        }.hop(to: loop ?? eventLoop)
    }

    private func idToPeer(_ id: PID) -> PeerID? {
        guard eventLoop.inEventLoop else { return nil }
        return self.peers[id]?.id
    }

    private func idsToPeers(_ ids: [PID]) -> [PeerID] {
        guard eventLoop.inEventLoop else { return [] }
        return ids.compactMap { self.peers[$0]?.id }
    }

    /// Returns a list of all known peers subscribed to the specified topic
    ///
    /// - TODO: Make this mo better... right now we do a ton of work to extract the PeerID for each subscriber (this could be solved if we changed peers to a dictionary with the PID as the key).
    func peersSubscribedTo(topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<[PeerID]> {
        eventLoop.submit { () -> [PeerID] in
            let subbed = self.mesh[topic] ?? []
            let known = self.fanout[topic] ?? []
            return self.idsToPeers(subbed + known)
        }.hop(to: loop ?? eventLoop)
    }

    func peersSubscribedTo(topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<[PubSub.Subscriber]> {
        eventLoop.submit { () -> [PubSub.Subscriber] in
            let subbed = self.mesh[topic] ?? []
            return subbed.compactMap {
                self.peers[$0]
            }
        }.hop(to: loop ?? eventLoop)
    }

    func getAllPeers(on loop: EventLoop?) -> EventLoopFuture<[PubSub.Subscriber]> {
        eventLoop.submit { () -> [PubSub.Subscriber] in
            self.peers.map { $0.value }
        }.hop(to: loop ?? eventLoop)
    }

    func streamsFor(_ peer: PeerID, on loop: EventLoop? = nil) -> EventLoopFuture<PubSub.Subscriber> {
        eventLoop.submit { () throws -> PubSub.Subscriber in
            if let p = self.peers[peer.b58String] {
                return p
            } else {
                throw Errors.unknownPeerID
            }
        }.hop(to: loop ?? eventLoop)
    }

    typealias Subscriptions = (full: [Topic], meta: [Topic])

    /// Returns the subscriber info (PeerID and Stream) for the specified b58string peer id
    func subscriptionForID(_ id: PID) -> EventLoopFuture<(PubSub.Subscriber, Subscriptions)> {
        eventLoop.submit { () throws -> (PubSub.Subscriber, Subscriptions) in
            guard let sub = self.peers[id] else { throw Errors.unknownPeerID }
            let full: [Topic] = self.mesh.compactMap { topic in
                if topic.value.contains(id) { return topic.key }
                return nil
            }
            let meta: [Topic] = self.fanout.compactMap { topic in
                if topic.value.contains(id) { return topic.key }
                return nil
            }

            return (sub, (full: full, meta: meta))
        }
    }

    /// This method returns true if the peer is a full peers
    /// false if the peer is a meta data only peer
    /// and throws an error if the peer id is unknown
    /// - TODO: shouldn't this be topic specific??
    //    func isFullPeer(_ peer: PeerID) -> EventLoopFuture<Bool> {
    //        eventLoop.submit { () -> Bool in
    //            var isPeer = false
    //            var isFull = false
    //            let id = peer.b58String
    //
    //            /// Check our mesh cache for the peer id
    //            for (_, subs) in self.mesh {
    //                if subs.contains(id) {
    //                    isPeer = true
    //                    isFull = true
    //                    break
    //                }
    //            }
    //            /// If we found the peer in our Mesh cach, they're a full peer, return true!
    //            if isPeer && isFull { return true }
    //
    //            /// Lets proceed to check the fanout...
    //            for (_, subs) in self.fanout {
    //                if subs.contains(id) {
    //                    isPeer = true
    //                    isFull = false
    //                    break
    //                }
    //            }
    //
    //            /// We found the peer but they're a metadata only peer...
    //            if isPeer { return false }
    //
    //            /// If we don't have record of this peer, throw an error
    //            self.logger.error("Error while checking isFullPeer, unknown PeerID:\(peer)")
    //            throw Errors.unknownPeerID
    //        }
    //    }

    //    func makeFullPeer(_ peer: PeerID, for topic: String) -> EventLoopFuture<Void> {
    //        eventLoop.submit { () -> Void in
    //            let pid = peer.b58String
    //            if var subs = self.mesh[topic] {
    //                if !subs.contains( pid ) {
    //                    /// Add the peer to our mesh cache
    //                    subs.append(pid)
    //                    self.mesh[topic] = subs
    //                } else {
    //                    /// The peer is already a full peer in our mesh cache
    //                    return
    //                }
    //            } else {
    //                /// We don't have an entry for this topic yet
    //                self.mesh[topic] = [pid]
    //            }
    //
    //            /// Make sure we remove the PID from our fanout cache
    //            self.fanout[topic]?.removeAll(where: { $0 == pid } )
    //        }
    //    }
    //
    //    func makeMetaPeer(_ peer: PeerID, for topic: String) -> EventLoopFuture<Void> {
    //        eventLoop.submit { () -> Void in
    //            let pid = peer.b58String
    //            if var subs = self.fanout[topic] {
    //                if !subs.contains( pid ) {
    //                    /// Add the peer to our fanout cache
    //                    subs.append(pid)
    //                    self.fanout[topic] = subs
    //                    //self.logger.info("Added peer \(pid) to fanout on topic \(topic)")
    //                } else {
    //                    /// The peer is already a meta data only peer in our fanout cache
    //                    //self.logger.info("Peer is already a meta peer, nothing to do")
    //                    return
    //                }
    //            } else {
    //                /// We don't have an entry for this topic yet
    //                self.fanout[topic] = [pid]
    //                //self.logger.info("Created new fanout for topic \(topic). And downgraded peer \(pid) to meta peer.")
    //            }
    //
    //            /// Make sure we remove the PID from our full message mesh cache
    //            self.mesh[topic]?.removeAll(where: { $0 == pid } )
    //            //self.logger.info("Removed \(pid) from mesh[\(topic)]")
    //            //self.logger.info("Remaining Full Peers for topic `\(topic)` -> \(self.mesh[topic]?.compactMap { $0.prefix(5) }.joined(separator: ", ") ?? "nil")")
    //            //self.logger.info("\(self.mesh)")
    //        }
    //    }

    enum Errors: Error {
        case unknownPeerID
        case unknownTopic
    }

}
