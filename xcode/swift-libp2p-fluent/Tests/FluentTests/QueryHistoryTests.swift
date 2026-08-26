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

import Fluent
import LibP2P
import LibP2PTesting
import RoutingKit
import Testing
import XCTFluent

@Suite("Query History Tests")
struct QueryHistoryTests {
    @Test func queryHistoryDisabled() async throws {
        try await withApp { app in
            let test = ArrayTestDatabase()
            app.databases.use(test.configuration, as: .test)

            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])

            app.on("foo") { req -> Response<ByteBuffer> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    let posts = try await Post.query(on: req.db).all()
                    #expect(req.fluent.history.queries.count == 0)
                    let buf = req.allocator.buffer(buffer: .init(bytes: try JSONEncoder().encode(posts)))
                    return .respondThenClose(buf)
                case .closed, .error:
                    return .close
                }
            }

            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.test(ma, protocol: "foo") { res async in
                #expect(res.payload.readableBytes > 0)
            }
        }
    }

    @Test func queryHistoryEnabled() async throws {
        try await withApp { app in
            let test = ArrayTestDatabase()
            app.databases.use(test.configuration, as: .test)

            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])

            app.on("foo") { req -> Response<ByteBuffer> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    req.fluent.history.start()
                    let posts = try await Post.query(on: req.db).all()
                    #expect(req.fluent.history.queries.count == 1)
                    let buf = req.allocator.buffer(buffer: .init(bytes: try JSONEncoder().encode(posts)))
                    return .respondThenClose(buf)
                case .closed, .error:
                    return .close
                }
            }

            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.test(ma, protocol: "foo") { res async in
                #expect(res.payload.readableBytes > 0)
            }
        }
    }

    @Test func queryHistoryEnableAndDisable() async throws {
        try await withApp { app in
            let test = ArrayTestDatabase()
            app.databases.use(test.configuration, as: .test)

            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])
            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])

            app.on("foo") { req -> Response<ByteBuffer> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    req.fluent.history.start()
                    _ = try await Post.query(on: req.db).all()
                    #expect(req.fluent.history.queries.count == 1)
                    req.fluent.history.stop()

                    let posts = try await Post.query(on: req.db).all()
                    #expect(req.fluent.history.queries.count == 1)

                    let buf = req.allocator.buffer(buffer: .init(bytes: try JSONEncoder().encode(posts)))
                    return .respondThenClose(buf)
                case .closed, .error:
                    return .close
                }
            }

            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.test(ma, protocol: "foo") { res async in
                #expect(res.payload.readableBytes > 0)
            }
        }
    }

    @Test func queryHistoryForApp() async throws {
        try await withApp { app in
            app.fluent.history.start()
            let test = ArrayTestDatabase()
            app.databases.use(test.configuration, as: .test)

            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])

            _ = try await Post.query(on: app.db).all()

            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])

            _ = try await Post.query(on: app.db).all()

            test.append([
                TestOutput(["id": 1, "content": "a"]),
                TestOutput(["id": 2, "content": "b"]),
            ])

            app.fluent.history.stop()
            _ = try await Post.query(on: app.db).all()
            #expect(app.fluent.history.queries.count == 2)
        }
    }
}

private final class Post: Model, Codable, Equatable, @unchecked Sendable {
    static func == (lhs: Post, rhs: Post) -> Bool {
        lhs.id == rhs.id && lhs.content == rhs.content
    }

    static var schema: String { "posts" }

    @ID(custom: .id)
    var id: Int?

    @Field(key: "content")
    var content: String

    init() {}

    init(id: Int? = nil, content: String) {
        self.id = id
        self.content = content
    }
}
