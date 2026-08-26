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
import NIOConcurrencyHelpers
import RoutingKit
import Testing
import XCTFluent

@Suite("Repository Tests")
struct RepositoryTests {
    @Test func repositoryPatternStatic() async throws {
        try await withApp { app in
            let posts: NIOLockedValueBox<[Post]> = .init([
                .init(content: "a"),
                .init(content: "b"),
            ])

            app.posts.use {
                TestPostRepository(posts: posts.withLockedValue { $0 }, eventLoop: $0.eventLoop)
            }

            app.on("foo") { req -> Response<ByteBuffer> in
                switch req.event {
                case .ready: return .stayOpen
                case .data:
                    let posts = try await req.posts.all()
                    let buf = req.allocator.buffer(buffer: .init(bytes: try JSONEncoder().encode(posts)))
                    return .respondThenClose(buf)
                case .closed, .error:
                    return .close
                }
            }

            let ma = try Multiaddr("/ip4/127.0.0.1/tcp/10000")
            try await app.test(ma, protocol: "foo") { res async in
                expectJSONEquals(res.payload.string, posts.withLockedValue { $0 })
            }

            posts.withLockedValue { $0.append(.init(content: "c")) }

            try await app.test(ma, protocol: "foo") { res async in
                expectJSONEquals(res.payload.string, posts.withLockedValue { $0 })
            }
        }
    }

    //    @Test func repositoryPatternDatabase() async throws {
    //        try await withApp { app in
    //            let test = ArrayTestDatabase()
    //            app.databases.use(test.configuration, as: .test)
    //
    //            app.posts.use { req in
    //                DatabasePostRepository(database: req.db(.test))
    //            }
    //
    //            app.get("foo") { req -> [Post] in
    //                try await req.posts.all()
    //            }
    //
    //            let posts: [Post] = [
    //                .init(id: 1, content: "a"),
    //                .init(id: 2, content: "b"),
    //            ]
    //
    //            test.append([
    //                TestOutput(["id": 1, "content": "a"]),
    //                TestOutput(["id": 2, "content": "b"]),
    //            ])
    //
    //            try await app.test(.GET, "foo") { res async in
    //                #expect(res.status == .ok)
    //                expectJSONEquals(res.body.string, posts)
    //            }
    //        }
    //    }
}

extension ByteBuffer {
    var string: String {
        self.getString(at: self.readerIndex, length: self.readableBytes)!
    }
}

extension Request {
    fileprivate var posts: any PostRepository {
        self.application.posts.makePosts!(self)
    }
}

extension Application {
    private struct PostRepositoryKey: StorageKey {
        typealias Value = PostRepositoryFactory
    }

    fileprivate var posts: PostRepositoryFactory {
        get { self.storage[PostRepositoryKey.self] ?? .init() }
        set { self.storage[PostRepositoryKey.self] = newValue }
    }
}

// not actually Sendable but the compiler doesn't need to know that
private struct PostRepositoryFactory: @unchecked Sendable {
    var makePosts: (@Sendable (Request) -> any PostRepository)?

    mutating func use(_ makePosts: @escaping @Sendable (Request) -> any PostRepository) {
        self.makePosts = makePosts
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

private struct TestPostRepository: PostRepository {
    let posts: [Post]
    let eventLoop: any EventLoop

    func all() async throws -> [Post] { self.posts }
}

private struct DatabasePostRepository: PostRepository {
    let database: any Database

    func all() async throws -> [Post] { try await self.database.query(Post.self).all() }
}

private protocol PostRepository {
    func all() async throws -> [Post]
}
