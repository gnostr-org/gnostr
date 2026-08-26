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

import Queues

struct MyAsyncJob: AsyncJob {
    let promise: EventLoopPromise<Void>

    struct Payload: Codable {
        var foo: String
    }

    func dequeue(_: QueueContext, _: Payload) async throws {
        self.promise.succeed()
    }
}
