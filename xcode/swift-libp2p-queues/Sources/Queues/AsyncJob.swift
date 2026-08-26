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

/// A task that can be queued for future execution.
public protocol AsyncJob: Job {
    associatedtype Payload

    /// Called when it's this Job's turn to be dequeued.
    ///
    /// - Parameters:
    ///   - context: The ``QueueContext``.
    ///   - payload: The typed job payload.
    func dequeue(
        _ context: QueueContext,
        _ payload: Payload
    ) async throws

    /// Called when there is an error at any stage of the Job's execution.
    ///
    /// - Parameters:
    ///   - context: The ``QueueContext``.
    ///   - error: The error returned by the job.
    ///   - payload: The typed job payload.
    func error(
        _ context: QueueContext,
        _ error: any Error,
        _ payload: Payload
    ) async throws
}

extension AsyncJob {
    /// Default implementation of ``AsyncJob/error(_:_:_:)-8627d``.
    public func error(_ context: QueueContext, _ error: any Error, _ payload: Payload) async throws {}
}

extension AsyncJob {
    /// Forward ``Job/dequeue(_:_:)`` to ``AsyncJob/dequeue(_:_:)-9g26t``.
    public func dequeue(_ context: QueueContext, _ payload: Payload) -> EventLoopFuture<Void> {
        context.eventLoop.makeFutureWithTask {
            try await self.dequeue(context, payload)
        }
    }

    /// Forward ``Job/error(_:_:_:)-2brrj`` to ``AsyncJob/error(_:_:_:)-8627d``
    public func error(_ context: QueueContext, _ error: any Error, _ payload: Payload) -> EventLoopFuture<Void> {
        context.eventLoop.makeFutureWithTask {
            try await self.error(context, error, payload)
        }
    }

}
