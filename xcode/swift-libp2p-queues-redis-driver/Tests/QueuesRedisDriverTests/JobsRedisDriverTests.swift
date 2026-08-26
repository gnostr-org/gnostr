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
import LibP2PTesting
import NIOCore
import Queues
import QueuesTesting
import Testing

@preconcurrency import protocol Redis.RedisClient
import struct Redis.RedisKey

@testable import QueuesRedisDriver

@Suite("Jobs Redis Driver Tests", .serialized)
struct JobsRedisDriverTests {

    /// This defaults to `localhost:6379`
    /// - Note: Github CI Workflows
    ///     - Our PR workflow sets the `REDIS_HOSTNAME` and `REDIS_PORT` ENV values
    ///     - ENV values are disregarded on MacOS CI runners
    /// - Note: We use a redis conf (instead of a url) due to the url parser throwing an InvalidRedisURL error
    ///     - `redis://redis:6379`          --> Error Invalid Redis URL
    ///     - `redis://localhost:6379` --> OK
    ///     - `redis://127.0.0.1:6379` --> OK
    static var redisConf: RedisConfiguration {
        get throws {
            #if os(Linux)
            return try RedisConfiguration(
                hostname: Environment.get("REDIS_HOSTNAME") ?? "localhost",
                port: Environment.get("REDIS_PORT")?.int ?? 6379,
                pool: .init(connectionRetryTimeout: .milliseconds(100))
            )
            #else
            return try RedisConfiguration(
                hostname: "localhost",
                port: 6379,
                pool: .init(connectionRetryTimeout: .milliseconds(100))
            )
            #endif
        }
    }

    /// Ensures that a sample job runs on our asyncTest driver
    @Test func testApplication_TestDriver() async throws {
        let email = Email()

        func configure(_ app: Application) async throws {
            app.queues.use(.asyncTest)
            app.queues.add(email)

            app.on("send-email") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Email.self, .init(to: "tanner@vapor.codes"))
                    return .respondThenClose("OK")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "send-email") { res in
                #expect(res.payload.string == "OK")
            }

            #expect(email.sent == [])
            try await app.queues.queue.worker.run().get()
            #expect(email.sent == [.init(to: "tanner@vapor.codes")])
        }
    }

    @Test func testApplication() async throws {
        let email = Email()

        func configure(_ app: Application) async throws {
            try app.queues.use(.redis(JobsRedisDriverTests.redisConf))
            app.queues.add(email)

            app.on("send-email") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Email.self, .init(to: "tanner@vapor.codes"))
                    return .respondThenClose("OK")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "send-email") { res in
                #expect(res.payload.string == "OK")
            }

            #expect(email.sent == [])
            try await app.queues.queue.worker.run().get()
            #expect(email.sent == [.init(to: "tanner@vapor.codes")])

            // Remove the queue keys from redis in case we do back to back tests
            let redis = try #require(app.queues.queue as? RedisClient)
            try await clear(redis: redis)
        }
    }

    @Test func testFailedJobLoss() async throws {
        let failedJob = FailingJob()
        let jobId = JobIdentifier()

        func configure(_ app: Application) async throws {
            try app.queues.use(.redis(JobsRedisDriverTests.redisConf))
            app.queues.add(failedJob)

            app.on("test") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(FailingJob.self, ["foo": "bar"], id: jobId)
                    return .respondThenClose("OK")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "test") { res in
                #expect(res.payload.string == "OK")
            }

            await #expect(throws: FailingJob.Failure.self) {
                try await app.queues.queue.worker.run().get()
            }

            // ensure the failed job is still present in storage
            let redis = try #require(app.queues.queue as? RedisClient)
            let job = try #require(
                await redis.get(RedisKey("job:\(jobId.string)"), asJSON: JobData.self).get()
            )
            #expect(job.jobName == "FailingJob")

            // Remove the queue keys from redis in case we do back to back tests
            try await clear(redis: redis)
        }
    }

    @Test func testDateEncoding() async throws {
        let jobId = JobIdentifier()

        func configure(_ app: Application) async throws {
            try app.queues.use(.redis(JobsRedisDriverTests.redisConf))
            app.queues.add(DelayedJob(recordIssue: false))

            app.on("delay-job") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(
                        DelayedJob.self,
                        .init(name: "libp2p"),
                        delayUntil: Date(timeIntervalSince1970: 1_609_477_200),
                        id: jobId
                    )
                    return .respondThenClose("OK")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "delay-job") { res in
                #expect(res.payload.string == "OK")
            }

            // Verify the delayUntil date is encoded as the correct epoch time
            let redis = try #require(app.queues.queue as? RedisClient)
            let job = try await redis.get(RedisKey("job:\(jobId.string)")).get()
            let dict = try JSONSerialization.jsonObject(with: job.data!, options: .allowFragments) as! [String: Any]

            #expect(dict["jobName"] as! String == "DelayedJob")
            #expect(dict["delayUntil"] as! Int == 1_609_477_200)

            // Remove the queue keys from redis in case we do back to back tests
            try await clear(redis: redis)
        }
    }

    @Test func testDelayedJobIsRemovedFromProcessingQueue() async throws {
        let jobId = JobIdentifier()

        func configure(_ app: Application) async throws {
            try app.queues.use(.redis(JobsRedisDriverTests.redisConf))
            app.queues.add(DelayedJob(recordIssue: true))

            app.on("delay-job") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(
                        DelayedJob.self,
                        .init(name: "libp2p"),
                        delayUntil: Date().addingTimeInterval(3600),
                        id: jobId
                    )
                    return .respondThenClose("OK")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "delay-job") { res in
                #expect(res.payload.string == "OK")
            }

            // Verify that a delayed job isn't still in processing after it's been put back in the queue
            try await app.queues.queue.worker.run().get()

            try await Task.sleep(for: .milliseconds(10))

            let redis = try #require(app.queues.queue as? RedisClient)
            let value = try await redis.lrange(
                from: RedisKey("libp2p_queues[default]-processing"),
                indices: 0...10,
                as: String.self
            ).get()
            let originalQueue = try await redis.lrange(
                from: RedisKey("libp2p_queues[default]"),
                indices: 0...10,
                as: String.self
            ).get()
            #expect(value.count == 0)
            #expect(originalQueue.count == 1)
            #expect(originalQueue.contains(jobId.string))

            // Remove the queue keys from redis in case we do back to back tests
            try await clear(redis: redis)
        }
    }

    func clear(redis: RedisClient) async throws {
        // Remove the queue keys from redis in case we do back to back tests
        let deleted = try await redis.delete([
            RedisKey("libp2p_queues[default]"),
            RedisKey("libp2p_queues[default]-processing"),
        ])
        #expect(deleted >= 0)
    }
}

var hostname: String {
    ProcessInfo.processInfo.environment["REDIS_HOSTNAME"] ?? "localhost"
}

final class Email: Job {
    struct Message: Codable, Equatable {
        let to: String
    }

    private let _sent: NIOLockedValueBox<[Message]>
    var sent: [Message] {
        get { _sent.withLockedValue { $0 } }
        set { _sent.withLockedValue { $0 = newValue } }
    }

    init() {
        self._sent = .init([])
    }

    func dequeue(_ context: QueueContext, _ message: Message) -> EventLoopFuture<Void> {
        self.sent.append(message)
        context.logger.info("sending email \(message)")
        return context.eventLoop.makeSucceededFuture(())
    }
}

final class DelayedJob: Job {
    struct Message: Codable, Equatable {
        let name: String
    }

    let recordIssue: Bool

    init(recordIssue: Bool) {
        self.recordIssue = recordIssue
    }

    func dequeue(_ context: QueueContext, _ message: Message) -> EventLoopFuture<Void> {
        context.logger.info("Hello \(message.name)")
        if recordIssue { Issue.record("Delayed Job Should Not Have Run") }
        return context.eventLoop.makeSucceededFuture(())
    }
}

struct FailingJob: Job {
    struct Failure: Error {}

    init() {}

    func dequeue(_ context: QueueContext, _ message: [String: String]) -> EventLoopFuture<Void> {
        context.eventLoop.makeFailedFuture(Failure())
    }

    func error(_ context: QueueContext, _ error: Error, _ payload: [String: String]) -> EventLoopFuture<Void> {
        context.eventLoop.makeFailedFuture(Failure())
    }
}

extension String {
    var int: Int? { Int(self) }
}
