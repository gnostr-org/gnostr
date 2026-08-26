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

import Atomics
import LibP2PTesting
import NIOConcurrencyHelpers
import Queues
import QueuesTesting
import Testing

@Suite("Queue Tests", .serialized)
struct QueueTests {

    init() async throws {
        try #require(isLoggingConfigured)
    }

    @Test func testLibP2PIntegrationWithInProcessJob() async throws {
        var jobSignal1: EventLoopPromise<String>!
        var jobSignal2: EventLoopPromise<String>!

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            jobSignal1 = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo1(promise: jobSignal1))
            jobSignal2 = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo2(promise: jobSignal2))
            try app.queues.startInProcessJobs(on: .default)

            app.on("bar1") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Foo1.self, .init(foo: "Bar payload 1")).get()
                    return .respondThenClose("job bar dispatched")
                case .closed, .error:
                    return .close
                }
            }

            app.on("bar2") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Foo2.self, .init(foo: "Bar payload 2"))
                    return .respondThenClose("job bar dispatched")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "bar1") { res async in
                #expect(res.payload.string == "job bar dispatched")
            }

            try await app.testing().test(ma, protocol: "bar2") { res async in
                #expect(res.payload.string == "job bar dispatched")
            }

            #expect(try await jobSignal1.futureResult.get() == "Bar payload 1")
            #expect(try await jobSignal2.futureResult.get() == "Bar payload 2")
        }
    }

    @Test func testLibP2PIntegration() async throws {
        var promise: EventLoopPromise<String>!

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            promise = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo1(promise: promise))

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Foo1.self, .init(foo: "bar"))
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

            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job = try #require(app.queues.test.first(Foo1.self))
            #expect(app.queues.test.contains(Foo1.self))
            #expect(job.foo == "bar")

            try await app.queues.queue.worker.run()
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)

            #expect(try await promise.futureResult.get() == "bar")
        }
    }

    @Test func testRunUntilEmpty() async throws {
        var promise1: EventLoopPromise<String>!
        var promise2: EventLoopPromise<String>!

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            promise1 = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo1(promise: promise1))
            promise2 = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo2(promise: promise2))

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Foo1.self, .init(foo: "bar"))
                    try await req.queue.dispatch(Foo1.self, .init(foo: "quux"))
                    try await req.queue.dispatch(Foo2.self, .init(foo: "baz"))
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
            try await app.queues.queue.worker.run()
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)

            #expect(try await promise1.futureResult.get() == "quux")
            #expect(try await promise2.futureResult.get() == "baz")
        }
    }

    @Test func testSettingCustomId() async throws {
        var promise: EventLoopPromise<String>!

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            promise = app.eventLoopGroup.any().makePromise(of: String.self)

            app.queues.add(Foo1(promise: promise))

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(
                        Foo1.self,
                        .init(foo: "bar"),
                        id: JobIdentifier(string: "my-custom-id")
                    )
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

            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            #expect(app.queues.test.jobs.keys.map(\.string).contains("my-custom-id"))

            try await app.queues.queue.worker.run()
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)

            #expect(try await promise.futureResult.get() == "bar")
        }
    }

    @Test func testRepeatingScheduledJob() async throws {
        try await withApp { app in
            let scheduledJob = TestingScheduledJob()
            #expect(scheduledJob.count.load(ordering: .relaxed) == 0)
            app.queues.schedule(scheduledJob).everySecond()
            try app.queues.startScheduledJobs()

            let promise = app.eventLoopGroup.any().makePromise(of: Void.self)
            app.eventLoopGroup.any().scheduleTask(in: .seconds(5)) {
                #expect(scheduledJob.count.load(ordering: .relaxed) > 4)
                promise.succeed()
            }

            try await promise.futureResult.get()
        }
    }

    @Test func testAsyncRepeatingScheduledJob() async throws {
        try await withApp { app in
            let scheduledJob = AsyncTestingScheduledJob()
            #expect(scheduledJob.count.load(ordering: .relaxed) == 0)
            app.queues.schedule(scheduledJob).everySecond()
            try app.queues.startScheduledJobs()

            let promise = app.eventLoopGroup.any().makePromise(of: Void.self)
            app.eventLoopGroup.any().scheduleTask(in: .seconds(5)) {
                #expect(scheduledJob.count.load(ordering: .relaxed) > 4)
                promise.succeed()
            }

            try await promise.futureResult.get()
        }
    }

    @Test func testFailingScheduledJob() async throws {
        try await withApp { app in
            app.queues.schedule(FailingScheduledJob()).everySecond()
            try app.queues.startScheduledJobs()

            let promise = app.eventLoopGroup.any().makePromise(of: Void.self)
            app.eventLoopGroup.any().scheduleTask(in: .seconds(1)) {
                promise.succeed()
            }
            try await promise.futureResult.get()
        }
    }

    @Test func testAsyncFailingScheduledJob() async throws {
        try await withApp { app in
            app.queues.schedule(AsyncFailingScheduledJob()).everySecond()
            try app.queues.startScheduledJobs()

            let promise = app.eventLoopGroup.any().makePromise(of: Void.self)
            app.eventLoopGroup.any().scheduleTask(in: .seconds(1)) {
                promise.succeed()
            }
            try await promise.futureResult.get()
        }
    }

    @Test func testCustomWorkerCount() async throws {
        try await withApp { app in
            // Setup custom ELG with 4 threads
            let eventLoopGroup = MultiThreadedEventLoopGroup(numberOfThreads: 4)

            do {
                let count = app.eventLoopGroup.any().makePromise(of: Int.self)
                app.queues.use(custom: WorkerCountDriver(count: count))
                // Limit worker count to less than 4 threads
                app.queues.configuration.workerCount = 2

                try app.queues.startInProcessJobs(on: .default)
                #expect(try await count.futureResult.get() == 2)
            } catch {
                try? await eventLoopGroup.shutdownGracefully()
                throw error
            }
            try await eventLoopGroup.shutdownGracefully()
        }
    }

    @Test func testSuccessHooks() async throws {
        var promise: EventLoopPromise<String>!
        let successHook = SuccessHook()
        let errorHook = ErrorHook()
        let dispatchHook = DispatchHook()
        let dequeuedHook = DequeuedHook()

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            promise = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo1(promise: promise))
            app.queues.add(successHook)
            app.queues.add(errorHook)
            app.queues.add(dispatchHook)
            app.queues.add(dequeuedHook)

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Foo1.self, .init(foo: "bar"))
                    return .respondThenClose("done")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            #expect(dispatchHook.successHit == false)
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "foo") { res async in
                #expect(res.payload.string == "done")
                #expect(dispatchHook.successHit)
            }

            #expect(successHook.successHit == false)
            #expect(errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job = try #require(app.queues.test.first(Foo1.self))
            #expect(app.queues.test.contains(Foo1.self))
            #expect(job.foo == "bar")
            #expect(dequeuedHook.successHit == false)

            try await app.queues.queue.worker.run()
            #expect(successHook.successHit)
            #expect(errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)
            #expect(dequeuedHook.successHit)

            #expect(try await promise.futureResult.get() == "bar")
        }
    }

    @Test func testAsyncSuccessHooks() async throws {
        var promise: EventLoopPromise<String>!
        let successHook = AsyncSuccessHook()
        let errorHook = AsyncErrorHook()
        let dispatchHook = AsyncDispatchHook()
        let dequeuedHook = AsyncDequeuedHook()

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            promise = app.eventLoopGroup.any().makePromise(of: String.self)
            app.queues.add(Foo1(promise: promise))
            app.queues.add(successHook)
            app.queues.add(errorHook)
            app.queues.add(dispatchHook)
            app.queues.add(dequeuedHook)

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Foo1.self, .init(foo: "bar"))
                    return .respondThenClose("done")
                case .closed, .error:
                    return .close
                }
            }
        }

        try await withApp(configure: configure) { app in
            #expect(await dispatchHook.successHit == false)
            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.testing().test(ma, protocol: "foo") { res async in
                #expect(res.payload.string == "done")
                #expect(await dispatchHook.successHit)
            }

            #expect(await successHook.successHit == false)
            #expect(await errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job = try #require(app.queues.test.first(Foo1.self))
            #expect(app.queues.test.contains(Foo1.self))
            #expect(job.foo == "bar")
            #expect(await dequeuedHook.successHit == false)

            try await app.queues.queue.worker.run()
            #expect(await successHook.successHit)
            #expect(await errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)
            #expect(await dequeuedHook.successHit)

            #expect(try await promise.futureResult.get() == "bar")
        }
    }

    @Test func testFailureHooks() async throws {
        let successHook = SuccessHook()
        let errorHook = ErrorHook()

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            app.queues.add(Bar())
            app.queues.add(successHook)
            app.queues.add(errorHook)

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Bar.self, .init(foo: "bar"), maxRetryCount: 3)
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

            #expect(successHook.successHit == false)
            #expect(errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job = try #require(app.queues.test.first(Bar.self))
            #expect(app.queues.test.contains(Bar.self))
            #expect(job.foo == "bar")

            try await app.queues.queue.worker.run()
            try await app.queues.queue.worker.run()
            try await app.queues.queue.worker.run()
            try await app.queues.queue.worker.run()
            #expect(successHook.successHit == false)
            #expect(errorHook.errorCount == 1)
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)
        }
    }

    @Test func testAsyncFailureHooks() async throws {
        let successHook = AsyncSuccessHook()
        let errorHook = AsyncErrorHook()

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            app.queues.add(Bar())
            app.queues.add(successHook)
            app.queues.add(errorHook)

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Bar.self, .init(foo: "bar"), maxRetryCount: 3)
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

            #expect(await successHook.successHit == false)
            #expect(await errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job = try #require(app.queues.test.first(Bar.self))
            #expect(app.queues.test.contains(Bar.self))
            #expect(job.foo == "bar")

            try await app.queues.queue.worker.run()
            try await app.queues.queue.worker.run()
            try await app.queues.queue.worker.run()
            try await app.queues.queue.worker.run()
            #expect(await successHook.successHit == false)
            #expect(await errorHook.errorCount == 1)
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)
        }
    }

    @Test func testFailureHooksWithDelay() async throws {
        let successHook = SuccessHook()
        let errorHook = ErrorHook()

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            app.queues.add(Baz())
            app.queues.add(successHook)
            app.queues.add(errorHook)

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Baz.self, .init(foo: "baz"), maxRetryCount: 1)
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

            #expect(successHook.successHit == false)
            #expect(errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job1 = try #require(app.queues.test.first(Baz.self))
            #expect(app.queues.test.contains(Baz.self))
            #expect(job1.foo == "baz")

            try await app.queues.queue.worker.run()
            #expect(successHook.successHit == false)
            #expect(errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job2 = try #require(app.queues.test.first(Baz.self))
            #expect(app.queues.test.contains(Baz.self))
            #expect(job2.foo == "baz")

            try await Task.sleep(for: .seconds(1))

            try await app.queues.queue.worker.run()
            #expect(successHook.successHit == false)
            #expect(errorHook.errorCount == 1)
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)
        }
    }

    @Test func testAsyncFailureHooksWithDelay() async throws {
        let successHook = AsyncSuccessHook()
        let errorHook = AsyncErrorHook()

        func configure(_ app: Application) async throws {
            app.queues.use(.test)
            app.queues.add(Baz())
            app.queues.add(successHook)
            app.queues.add(errorHook)

            app.on("foo") { req -> Response<String> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    try await req.queue.dispatch(Baz.self, .init(foo: "baz"), maxRetryCount: 1)
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

            #expect(await successHook.successHit == false)
            #expect(await errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job1 = try #require(app.queues.test.first(Baz.self))
            #expect(app.queues.test.contains(Baz.self))
            #expect(job1.foo == "baz")

            try await app.queues.queue.worker.run()
            #expect(await successHook.successHit == false)
            #expect(await errorHook.errorCount == 0)
            #expect(app.queues.test.queue.count == 1)
            #expect(app.queues.test.jobs.count == 1)
            let job2 = try #require(app.queues.test.first(Baz.self))
            #expect(app.queues.test.contains(Baz.self))
            #expect(job2.foo == "baz")

            try await Task.sleep(for: .seconds(1))

            try await app.queues.queue.worker.run()
            #expect(await successHook.successHit == false)
            #expect(await errorHook.errorCount == 1)
            #expect(app.queues.test.queue.count == 0)
            #expect(app.queues.test.jobs.count == 0)
        }
    }

    @Test func testStuffThatIsntActuallyUsedAnywhere() async throws {
        func configure(_ app: Application) async throws {
            app.queues.use(.test)
        }

        try await withApp(configure: configure) { app in
            #expect(app.queues.queue(.default).key == "libp2p_queues[default]")
            //#expect(QueuesEventLoopPreference.indifferent.delegate(for: app.eventLoopGroup) != nil)
            //#expect(QueuesEventLoopPreference.delegate(on: app.eventLoopGroup.any()).delegate(for: app.eventLoopGroup) != nil)
        }
    }
}

final class DispatchHook: JobEventDelegate, @unchecked Sendable {
    var successHit = false

    func dispatched(job _: JobEventData, eventLoop: any EventLoop) -> EventLoopFuture<Void> {
        self.successHit = true
        return eventLoop.makeSucceededVoidFuture()
    }
}

final class SuccessHook: JobEventDelegate, @unchecked Sendable {
    var successHit = false

    func success(jobId _: String, eventLoop: any EventLoop) -> EventLoopFuture<Void> {
        self.successHit = true
        return eventLoop.makeSucceededVoidFuture()
    }
}

final class ErrorHook: JobEventDelegate, @unchecked Sendable {
    var errorCount = 0

    func error(jobId _: String, error _: any Error, eventLoop: any EventLoop) -> EventLoopFuture<Void> {
        self.errorCount += 1
        return eventLoop.makeSucceededVoidFuture()
    }
}

final class DequeuedHook: JobEventDelegate, @unchecked Sendable {
    var successHit = false

    func didDequeue(jobId _: String, eventLoop: any EventLoop) -> EventLoopFuture<Void> {
        self.successHit = true
        return eventLoop.makeSucceededVoidFuture()
    }
}

actor AsyncDispatchHook: AsyncJobEventDelegate {
    var successHit = false
    func dispatched(job _: JobEventData) async throws { self.successHit = true }
}

actor AsyncSuccessHook: AsyncJobEventDelegate {
    var successHit = false
    func success(jobId _: String) async throws { self.successHit = true }
}

actor AsyncErrorHook: AsyncJobEventDelegate {
    var errorCount = 0
    func error(jobId _: String, error _: any Error) async throws { self.errorCount += 1 }
}

actor AsyncDequeuedHook: AsyncJobEventDelegate {
    var successHit = false
    func didDequeue(jobId _: String) async throws { self.successHit = true }
}

final class WorkerCountDriver: QueuesDriver, @unchecked Sendable {
    let count: EventLoopPromise<Int>
    let lock: NIOLock
    var recordedEventLoops: Set<ObjectIdentifier>

    init(count: EventLoopPromise<Int>) {
        self.count = count
        self.lock = .init()
        self.recordedEventLoops = []
    }

    func makeQueue(with context: QueueContext) -> any Queue {
        WorkerCountQueue(driver: self, context: context)
    }

    func record(eventLoop: any EventLoop) {
        self.lock.lock()
        defer { self.lock.unlock() }
        let previousCount = self.recordedEventLoops.count
        self.recordedEventLoops.insert(.init(eventLoop))
        if self.recordedEventLoops.count == previousCount {
            // we've detected all unique event loops now
            self.count.succeed(previousCount)
        }
    }

    func shutdown() {
        // nothing
    }

    private struct WorkerCountQueue: Queue {
        let driver: WorkerCountDriver
        var context: QueueContext

        func get(_: JobIdentifier) -> EventLoopFuture<JobData> { fatalError() }
        func set(_: JobIdentifier, to _: JobData) -> EventLoopFuture<Void> { fatalError() }
        func clear(_: JobIdentifier) -> EventLoopFuture<Void> { fatalError() }
        func pop() -> EventLoopFuture<JobIdentifier?> {
            self.driver.record(eventLoop: self.context.eventLoop)
            return self.context.eventLoop.makeSucceededFuture(nil)
        }

        func push(_: JobIdentifier) -> EventLoopFuture<Void> { fatalError() }
    }
}

struct FailingScheduledJob: ScheduledJob {
    func run(context: QueueContext) -> EventLoopFuture<Void> { context.eventLoop.makeFailedFuture(Failure()) }
}

struct AsyncFailingScheduledJob: AsyncScheduledJob {
    func run(context _: QueueContext) async throws { throw Failure() }
}

struct TestingScheduledJob: ScheduledJob {
    var count = ManagedAtomic<Int>(0)

    func run(context: QueueContext) -> EventLoopFuture<Void> {
        self.count.wrappingIncrement(ordering: .relaxed)
        return context.eventLoop.makeSucceededVoidFuture()
    }
}

struct AsyncTestingScheduledJob: AsyncScheduledJob {
    var count = ManagedAtomic<Int>(0)
    func run(context _: QueueContext) async throws { self.count.wrappingIncrement(ordering: .relaxed) }
}

struct Foo1: Job {
    let promise: EventLoopPromise<String>

    struct Data: Codable {
        var foo: String
    }

    func dequeue(_ context: QueueContext, _ data: Data) -> EventLoopFuture<Void> {
        self.promise.succeed(data.foo)
        return context.eventLoop.makeSucceededVoidFuture()
    }

    func error(_ context: QueueContext, _ error: any Error, _: Data) -> EventLoopFuture<Void> {
        self.promise.fail(error)
        return context.eventLoop.makeSucceededVoidFuture()
    }
}

struct Foo2: Job {
    let promise: EventLoopPromise<String>

    struct Data: Codable {
        var foo: String
    }

    func dequeue(_ context: QueueContext, _ data: Data) -> EventLoopFuture<Void> {
        self.promise.succeed(data.foo)
        return context.eventLoop.makeSucceededVoidFuture()
    }

    func error(_ context: QueueContext, _ error: any Error, _: Data) -> EventLoopFuture<Void> {
        self.promise.fail(error)
        return context.eventLoop.makeSucceededVoidFuture()
    }
}

struct Bar: Job {
    enum Error: Swift.Error {
        case badRequest
    }

    struct Data: Codable {
        var foo: String
    }

    func dequeue(_ context: QueueContext, _: Data) -> EventLoopFuture<Void> {
        context.eventLoop.makeFailedFuture(Error.badRequest)
    }

    func error(_ context: QueueContext, _: any Swift.Error, _: Data) -> EventLoopFuture<Void> {
        context.eventLoop.makeSucceededVoidFuture()
    }
}

struct Baz: Job {
    enum Error: Swift.Error {
        case badRequest
    }

    struct Data: Codable {
        var foo: String
    }

    func dequeue(_ context: QueueContext, _: Data) -> EventLoopFuture<Void> {
        context.eventLoop.makeFailedFuture(Error.badRequest)
    }

    func error(_ context: QueueContext, _: any Swift.Error, _: Data) -> EventLoopFuture<Void> {
        context.eventLoop.makeSucceededVoidFuture()
    }

    func nextRetryIn(attempt: Int) -> Int {
        attempt
    }
}
