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

import Foundation
import LibP2P
import NIOCore

extension Request {
    /// Get the default ``Queue``.
    public var queue: any Queue {
        self.queues(.default)
    }

    /// Create or look up an instance of a named ``Queue`` and bind it to this request's event loop.
    ///
    /// - Parameter queue: The queue name
    public func queues(_ queue: QueueName, logger: Logger? = nil) -> any Queue {
        self.application.queues.queue(queue, logger: logger ?? self.logger, on: self.eventLoop)
    }
}
