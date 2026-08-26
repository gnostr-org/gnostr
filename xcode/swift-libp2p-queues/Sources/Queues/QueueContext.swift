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

import LibP2P
import Logging
import NIOCore

/// The context for a queue.
public struct QueueContext: Sendable {
    /// The name of the queue
    public let queueName: QueueName

    /// The configuration object
    public let configuration: QueuesConfiguration

    /// The application object
    public let application: Application

    /// The logger object
    public var logger: Logger

    /// An event loop to run the process on
    public let eventLoop: any EventLoop

    /// Creates a new JobContext
    /// - Parameters:
    ///   - queueName: The name of the queue
    ///   - configuration: The configuration object
    ///   - application: The application object
    ///   - logger: The logger object
    ///   - eventLoop: An event loop to run the process on
    public init(
        queueName: QueueName,
        configuration: QueuesConfiguration,
        application: Application,
        logger: Logger,
        on eventLoop: any EventLoop
    ) {
        self.queueName = queueName
        self.configuration = configuration
        self.application = application
        self.logger = logger
        self.eventLoop = eventLoop
    }

    /// Returns the default job `Queue`
    public var queue: any Queue {
        self.queues(.default)
    }

    /// Returns the specific job `Queue` for the given queue name
    /// - Parameter queue: The queue name
    public func queues(_ queue: QueueName) -> any Queue {
        self.application.queues.queue(
            queue,
            logger: self.logger,
            on: self.eventLoop
        )
    }
}
