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

/// Describes a job that can be scheduled and repeated
public protocol AsyncScheduledJob: ScheduledJob {
    var name: String { get }

    /// The method called when the job is run
    /// - Parameter context: A `JobContext` that can be used
    func run(context: QueueContext) async throws
}

extension AsyncScheduledJob {
    public var name: String { "\(Self.self)" }

    public func run(context: QueueContext) -> EventLoopFuture<Void> {
        context.eventLoop.makeFutureWithTask {
            try await self.run(context: context)
        }
    }
}
