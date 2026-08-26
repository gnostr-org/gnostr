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

extension Application.PubSubServices.Provider {
    public static var gossipsub: Self {
        .init {
            $0.pubsub.use { app -> GossipSub in
                let gsub = try! GossipSub(group: app.eventLoopGroup, libp2p: app)
                app.lifecycle.use(gsub)
                return gsub
            }
        }
    }

    public static func gossipsub(emitSelf: Bool) -> Self {
        .init {
            $0.pubsub.use { app -> GossipSub in
                let gsub = try! GossipSub(group: app.eventLoopGroup, libp2p: app, emitSelf: emitSelf)
                app.lifecycle.use(gsub)
                return gsub
            }
        }
    }
}

extension Application.PubSubServices {

    public var gossipsub: GossipSub {
        guard let gsub = self.service(for: GossipSub.self) else {
            fatalError(
                "Gossipsub accessed without instantiating it first. Use app.pubsub.use(.gossipsub) to initialize a shared Gossipsub instance."
            )
        }
        return gsub
    }

    //    public var gossipsub: GossipSub {
    //        let lock = self.application.locks.lock(for: Key.self)
    //        lock.lock()
    //        defer { lock.unlock() }
    //        if let existing = self.application.storage[Key.self] {
    //            return existing
    //        }
    //        let new = ClientBootstrap(group: self.application.eventLoopGroup)
    //            // Enable SO_REUSEADDR.
    //            .channelOption(ChannelOptions.socketOption(.so_reuseaddr), value: 1)
    //            .channelInitializer { channel in
    //                // Do we install the upgrader here or do we let the Connection install the handlers???
    //                //channel.pipeline.addHandlers(upgrader.channelHandlers(mode: .initiator)) // The MSS Handler itself needs to have access to the Connection Delegate
    //                channel.eventLoop.makeSucceededVoidFuture()
    //            }
    //
    //        self.application.storage.set(Key.self, to: new)
    //
    //        return new
    //    }

}
