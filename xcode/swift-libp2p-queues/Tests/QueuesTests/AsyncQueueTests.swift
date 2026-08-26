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

import LibP2PTesting
import Queues
import Testing

@Suite("Async Queue Tests", .serialized)
struct AsyncQueueTests {

    init() async throws {
        try #require(isLoggingConfigured)
    }

    @Test func testAsyncJobWithSyncQueue() async throws {
        var promise: EventLoopPromise<Void>!

        func configure(_ app: Application) async throws {
            app.queues.use(.test)

            promise = app.eventLoopGroup.any().makePromise(of: Void.self)
            app.queues.add(MyAsyncJob(promise: promise))

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(MyAsyncJob.self, .init(foo: "bar"))
                    try await req.queue.dispatch(MyAsyncJob.self, .init(foo: "baz"))
                    try await req.queue.dispatch(MyAsyncJob.self, .init(foo: "quux"))
                    return .respondThenClose("done")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "foo") { res async in
                #expect(res.payload.string == "done")
            }

            #expect(app.queues.test.queue.count == 3)
            #expect(app.queues.test.jobs.count == 3)
            let job = try #require(app.queues.test.first(MyAsyncJob.self))
            #expect(app.queues.test.contains(MyAsyncJob.self))
            #expect(job.foo == "bar")

            try await app.queues.queue.worker.run().get()
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)

            await #expect(throws: Never.self) { try await promise.futureResult.get() }
        }
    }

    @Test func testAsyncJobWithAsyncQueue() async throws {
        var promise: EventLoopPromise<Void>!

        func configure(_ app: Application) async throws {
            app.queues.use(.asyncTest)

            promise = app.eventLoopGroup.any().makePromise(of: Void.self)
            app.queues.add(MyAsyncJob(promise: promise))

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(MyAsyncJob.self, .init(foo: "bar"))
                    try await req.queue.dispatch(MyAsyncJob.self, .init(foo: "baz"))
                    try await req.queue.dispatch(MyAsyncJob.self, .init(foo: "quux"))
                    return .respondThenClose("done")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "foo") { res async in
                #expect(res.payload.string == "done")
            }

            #expect(app.queues.asyncTest.queue.count == 3)
            #expect(app.queues.asyncTest.jobs.count == 3)
            let job = try #require(app.queues.asyncTest.first(MyAsyncJob.self))
            #expect(app.queues.asyncTest.contains(MyAsyncJob.self))
            #expect(job.foo == "bar")

            try await app.queues.queue.worker.run().get()
            #expect(app.queues.asyncTest.queue.count == 0)
            #expect(app.queues.asyncTest.jobs.count == 0)

            await #expect(throws: Never.self) { try await promise.futureResult.get() }
        }
    }
}
