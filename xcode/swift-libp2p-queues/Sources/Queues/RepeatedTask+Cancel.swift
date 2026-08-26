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

import Logging
import NIOCore

extension RepeatedTask {
    func syncCancel(on eventLoop: any EventLoop) {
        do {
            let promise = eventLoop.makePromise(of: Void.self)
            self.cancel(promise: promise)
            try promise.futureResult.wait()
        } catch {
            Logger(label: "libp2p.queues.repeatedtask").debug(
                "Failed cancelling repeated task",
                metadata: ["error": "\(error)"]
            )
        }
    }

    func asyncCancel(on eventLoop: any EventLoop) async {
        do {
            let promise = eventLoop.makePromise(of: Void.self)
            self.cancel(promise: promise)
            try await promise.futureResult.get()
        } catch {
            Logger(label: "libp2p.queues.repeatedtask").debug(
                "Failed cancelling repeated task",
                metadata: ["error": "\(error)"]
            )
        }
    }
}
