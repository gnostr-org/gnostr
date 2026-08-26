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
@preconcurrency import RediStack
import Redis
import Testing

extension String {
    var int: Int? { Int(self) }
}

@Suite("Redis Tests", .serialized)
struct RedisTests {
    var redisConfig: RedisConfiguration!

    init() throws {
        #expect(isLoggingConfigured)
        #if os(Linux)
        redisConfig = try RedisConfiguration(
            hostname: Environment.get("REDIS_HOSTNAME") ?? "localhost",
            port: Environment.get("REDIS_PORT")?.int ?? 6379,
            pool: .init(connectionRetryTimeout: .milliseconds(100))
        )
        #else
        redisConfig = try RedisConfiguration(
            hostname: "localhost",
            port: 6379,
            pool: .init(connectionRetryTimeout: .milliseconds(100))
        )
        #endif
    }

}

// MARK: Core RediStack integration
extension RedisTests {

    @Test func testApplicationRedis() throws {
        let app = Application()
        defer { app.shutdown() }

        app.redis.configuration = redisConfig
        try app.boot()

        let info = try app.redis.send(command: "INFO").wait()
        #expect((info.string ?? "").contains("redis_version"))
    }

    @Test func testApplicationRedisAsync() async throws {
        let app = try await makeApp()

        app.redis.configuration = redisConfig
        try await app.startup()

        do {
            let info = try await app.redis.send(command: "INFO")
            #expect((info.string ?? "").contains("redis_version"))
        } catch {
            Issue.record(error)
        }

        try await app.asyncShutdown()
    }

    //func testRouteHandlerRedis() throws {
    //    let app = Application()
    //    defer { app.shutdown() }
    //
    //    app.redis.configuration = redisConfig
    //
    //    app.get("test") { req in
    //        req.redis.send(command: "INFO").map {
    //            $0.description
    //        }
    //    }
    //
    //    try app.test(.GET, "test") { res in
    //        XCTAssertContains(res.body.string, "redis_version")
    //    }
    //}
}

// MARK: Configuration Validation

extension RedisTests {
    @Test func testInitConfigurationURL() throws {
        let urlStr = URL(string: "redis://name:password@localhost:6379/0")

        let redisConfiguration = try RedisConfiguration(url: urlStr!)

        #expect(redisConfiguration.password == "password")
        #expect(redisConfiguration.database == 0)
    }
}

// MARK: Redis extensions
extension RedisTests {
    @Test func testCodable() throws {
        let app = Application()
        defer { app.shutdown() }
        app.redis.configuration = redisConfig
        try app.boot()

        struct Hello: Codable {
            var message: String
            var array: [Int]
            var dict: [String: Bool]
        }

        let hello = Hello(
            message: "world",
            array: [1, 2, 3],
            dict: ["yes": true, "false": false]
        )
        try app.redis.set("hello", toJSON: hello).wait()

        let get = try app.redis.get("hello", asJSON: Hello.self).wait()
        #expect(get?.message == "world")
        #expect(get?.array.first == 1)
        #expect(get?.array.last == 3)
        #expect(get?.dict["yes"] == true)
        #expect(get?.dict["false"] == false)

        let _ = try app.redis.delete(["hello"]).wait()
    }

    @Test func testCodableAsync() async throws {
        let app = try await makeApp()

        app.redis.configuration = redisConfig
        try await app.startup()

        struct Hello: Codable {
            var message: String
            var array: [Int]
            var dict: [String: Bool]
        }

        let hello = Hello(
            message: "world",
            array: [1, 2, 3],
            dict: ["yes": true, "false": false]
        )
        do {
            try await app.redis.set("hello", toJSON: hello)

            let get = try await app.redis.get("hello", asJSON: Hello.self)
            #expect(get?.message == "world")
            #expect(get?.array.first == 1)
            #expect(get?.array.last == 3)
            #expect(get?.dict["yes"] == true)
            #expect(get?.dict["false"] == false)

            let _ = try await app.redis.delete(["hello"])
        } catch {
            Issue.record(error)
        }

        try await app.asyncShutdown()
    }

    //func testRequestConnectionLeasing() throws {
    //    let app = Application()
    //    defer { app.shutdown() }
    //    app.redis.configuration = self.redisConfig
    //
    //    app.get("test") {
    //        $0.redis
    //            .withBorrowedClient { client in
    //                return client.send(command: "MULTI")
    //                    .flatMap { _ in client.send(command: "PING") }
    //                    .flatMap { queuedResponse -> EventLoopFuture<RESPValue> in
    //                        XCTAssertEqual(queuedResponse.string, "QUEUED")
    //                        return client.send(command: "EXEC")
    //                    }
    //            }
    //            .map { result -> [String] in
    //                guard let response = result.array else { return [] }
    //                return response.compactMap(String.init(fromRESP:))
    //            }
    //    }
    //
    //    try app.test(.GET, "test") {
    //        XCTAssertEqual($0.body.string, #"["PONG"]"#)
    //    }
    //}

    @Test func testApplicationConnectionLeasing() throws {
        let app = Application()
        defer { app.shutdown() }

        app.redis.configuration = self.redisConfig
        try app.boot()

        let result = try app.redis
            .withBorrowedConnection { client in
                client.send(command: "MULTI")
                    .flatMap { _ in client.send(command: "PING") }
                    .flatMap { queuedResponse -> EventLoopFuture<RESPValue> in
                        #expect(queuedResponse.string == "QUEUED")
                        return client.send(command: "EXEC")
                    }
            }
            .map { result -> [String] in
                guard let response = result.array else { return [] }
                return response.compactMap(String.init(fromRESP:))
            }
            .wait()

        #expect(result == ["PONG"])
    }

    private func makeApp() async throws -> Application {
        let app = try await Application.make(peerID: .ephemeral())
        app.environment.arguments = ["libp2p"]
        return app
    }
}

// MARK: Vapor integration
extension RedisTests {
    //func testSessions() throws {
    //    let app = Application(.testing)
    //    defer { app.shutdown() }
    //
    //    app.redis.configuration = redisConfig
    //
    //    // Configure sessions.
    //    app.sessions.use(.redis)
    //    app.middleware.use(app.sessions.middleware)
    //
    //    // Setup routes.
    //    app.get("set", ":value") { req -> HTTPStatus in
    //        req.session.data["name"] = req.parameters.get("value")
    //        return .ok
    //    }
    //    app.get("get") { req -> String in
    //        req.session.data["name"] ?? "n/a"
    //    }
    //    app.get("del") { req -> HTTPStatus in
    //        req.session.destroy()
    //        return .ok
    //    }
    //
    //    // Store session id.
    //    var sessionID: String?
    //    try app.test(.GET, "/set/vapor") { res in
    //        sessionID = res.headers.setCookie?["vapor-session"]?.string
    //        XCTAssertEqual(res.status, .ok)
    //    }
    //    XCTAssertFalse(try XCTUnwrap(sessionID).contains("vrs-"), "session token has the redis key prefix!")
    //
    //    try app.test(.GET, "/get", beforeRequest: { req in
    //        var cookies = HTTPCookies()
    //        cookies["vapor-session"] = .init(string: sessionID!)
    //        req.headers.cookie = cookies
    //    }) { res in
    //        XCTAssertEqual(res.status, .ok)
    //        XCTAssertEqual(res.body.string, "vapor")
    //    }
    //}

    @Test func testCache() throws {
        let app = Application()
        defer { app.shutdown() }

        app.redis.configuration = redisConfig
        app.caches.use(.redis)
        try app.boot()

        #expect(throws: Never.self) {
            try app.redis.send(command: "DEL", with: [.init(from: "foo")])
                .wait()
        }
        try #expect(app.cache.get("foo", as: String.self).wait() == nil)
        try app.cache.set("foo", to: "bar").wait()
        try #expect(app.cache.get("foo", as: String.self).wait() == "bar")

        // Test expiration
        try app.cache.set("foo2", to: "bar2", expiresIn: .seconds(1)).wait()
        try #expect(
            app.cache.get("foo2", as: String.self).wait() == "bar2"
        )
        sleep(2)
        try #expect(app.cache.get("foo2", as: String.self).wait() == nil)
    }

    @Test func testCacheAsync() async throws {
        let app = try await makeApp()

        app.redis.configuration = redisConfig
        app.caches.use(.redis)
        try await app.startup()

        do {
            await #expect(throws: Never.self) {
                try await app.redis.send(command: "DEL", with: [.init(from: "foo")])
            }
            try await #expect(app.cache.get("foo", as: String.self) == nil)
            try await app.cache.set("foo", to: "bar")
            try await #expect(app.cache.get("foo", as: String.self) == "bar")

            // Test expiration
            try await app.cache.set("foo2", to: "bar2", expiresIn: .seconds(1))
            try await #expect(
                app.cache.get("foo2", as: String.self) == "bar2"
            )
            try await Task.sleep(for: .seconds(1))
            try await #expect(app.cache.get("foo2", as: String.self) == nil)
        } catch {
            Issue.record(error)
        }

        try await app.asyncShutdown()
    }

    @Test func testCacheCustomCoders() throws {
        let app = Application()
        defer { app.shutdown() }

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        app.redis.configuration = redisConfig
        app.caches.use(.redis(encoder: encoder, decoder: decoder))
        try app.boot()

        let date = Date(timeIntervalSince1970: 10_000_000_000)
        let isoDate = ISO8601DateFormatter().string(from: date)

        try app.cache.set("test", to: date).wait()
        let rawValue = try #require(
            try app.redis.get("test", as: String.self).wait()
        )
        #expect(rawValue == #""\#(isoDate)""#)
        let value = try #require(
            try app.cache.get("test", as: Date.self).wait()
        )
        #expect(value == date)
    }

    @Test func testCacheCustomCoders_Async() async throws {
        let app = try await makeApp()

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        app.redis.configuration = redisConfig
        app.caches.use(.redis(encoder: encoder, decoder: decoder))
        try await app.startup()

        do {
            let date = Date(timeIntervalSince1970: 10_000_000_000)
            let isoDate = ISO8601DateFormatter().string(from: date)

            try await app.cache.set("test", to: date)
            let rawValue = try #require(
                try await app.redis.get("test", asJSON: String.self)
            )
            #expect(rawValue == isoDate)
            let value = try #require(
                try await app.cache.get("test", as: Date.self)
            )
            #expect(value == date)
        } catch {
            Issue.record(error)
        }

        try await app.asyncShutdown()
    }

    @Test func testRedisClientCustomCoders() throws {
        let app = Application()
        defer { app.shutdown() }

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        app.redis.configuration = redisConfig

        try app.boot()

        let date = Date(timeIntervalSince1970: 10_000_000_000)
        let isoDate = ISO8601DateFormatter().string(from: date)

        try app.redis.set("test", toJSON: date, jsonEncoder: encoder).wait()
        let rawValue = try #require(
            try app.redis.get("test", as: String.self).wait()
        )
        #expect(rawValue == #""\#(isoDate)""#)
        let value = try #require(
            try app.redis.get("test", asJSON: Date.self, jsonDecoder: decoder)
                .wait()
        )
        #expect(value == date)
    }

    @Test func testRedisClientCustomCoders_Async() async throws {
        let app = try await makeApp()

        let encoder = JSONEncoder()
        encoder.dateEncodingStrategy = .iso8601
        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601

        app.redis.configuration = redisConfig

        try await app.startup()

        do {
            let date = Date(timeIntervalSince1970: 10_000_000_000)
            let isoDate = ISO8601DateFormatter().string(from: date)

            try await app.redis.set("test", toJSON: date, jsonEncoder: encoder)
            let rawValue = try #require(
                try await app.redis.get("test", asJSON: String.self)
            )
            #expect(rawValue == isoDate)
            let value = try #require(
                try await app.redis.get("test", asJSON: Date.self, jsonDecoder: decoder)
            )
            #expect(value == date)
        } catch {
            Issue.record(error)
        }

        try await app.asyncShutdown()
    }
}

// MARK: Test Helpers

let isLoggingConfigured: Bool = {
    var env = Environment.testing
    try! LoggingSystem.bootstrap(from: &env)
    return true
}()
