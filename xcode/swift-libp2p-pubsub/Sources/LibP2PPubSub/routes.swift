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

//func routes(_ app: Application) throws {
//    app.group("pubsub") { rendezvous in
//
//        rendezvous.on("1.0.0") { req -> EventLoopFuture<ResponseType<ByteBuffer>> in
//            guard req.streamDirection == .inbound else {
//                req.logger.error("PubSub::Error -> PubSub route handler only accepts inbound streams")
//                return req.eventLoop.makeSucceededFuture(.close)
//            }
//
//            switch req.event {
//            case .ready:
//                return req.eventLoop.makeSucceededFuture(.stayOpen)
//
//            case .data:
//                req.logger.warning("PubSub::TODO::Implement Me")
//                return req.eventLoop.makeSucceededFuture(.stayOpen)
//
//            case .closed:
//                return req.eventLoop.makeSucceededFuture(.close)
//
//            case .error(let error):
//                req.logger.error("PubSub::Error -> \(error)")
//                return req.eventLoop.makeSucceededFuture(.close)
//            }
//        }
//    }
//}
