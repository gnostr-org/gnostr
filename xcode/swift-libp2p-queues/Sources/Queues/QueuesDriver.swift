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

/// A new driver for Queues
public protocol QueuesDriver: Sendable {
    /// Create or look up a named ``Queue`` instance.
    ///
    /// - Parameter context: The context for jobs on the queue. Also provides the queue name.
    func makeQueue(with context: QueueContext) -> any Queue

    /// Shuts down the driver
    func shutdown()

    /// Shut down the driver asynchronously. Helps avoid calling `.wait()`
    func asyncShutdown() async
}

extension QueuesDriver {
    public func asyncShutdown() async {
        self.shutdown()
    }
}
