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
import Testing
import XCTFluent

@Suite("Cache Tests")
struct CacheTests {
    @Test func cacheMigrationName() {
        #expect(CacheEntry.migration.name == "Fluent.CacheEntry.Create")
    }

    @Test func cacheGet() async throws {
        try await withApp { app in
            // Setup test db.
            let test = ArrayTestDatabase()
            app.databases.use(test.configuration, as: .test)
            app.migrations.add(CacheEntry.migration)

            // Configure cache.
            app.caches.use(.fluent)

            // simulate cache miss
            test.append([])
            do {
                let foo = try await app.cache.get("foo", as: String.self)
                #expect(foo == nil)
            } catch {
                Issue.record(error)
            }

            // simulate cache hit
            test.append([TestOutput(["key": "foo", "value": #""bar""#])])
            do {
                let foo = try await app.cache.get("foo", as: String.self)
                #expect(foo == "bar")
            } catch {
                Issue.record(error)
            }
        }
    }

    @Test func cacheSet() async throws {
        try await withApp { app in
            // Setup a Canary to ensure our callback is triggered
            let canary = Canary()
            // Setup test db.
            let test = CallbackTestDatabase { query in
                switch query.input[0] {
                case .dictionary(let dict):
                    switch dict["value"] {
                    case .bind(let value as String):
                        #expect(value == #""bar""#)
                    default:
                        Issue.record("unexpected value")
                    }
                default:
                    Issue.record("unexpected input")
                }
                // trigger the canary
                canary.trigger()
                return [TestOutput(["id": UUID()])]
            }
            app.databases.use(test.configuration, as: .test)
            app.migrations.add(CacheEntry.migration)

            // Configure cache.
            app.caches.use(.fluent)

            try await app.cache.set("foo", to: "bar")
            // Ensure the canary was triggered
            #expect(canary.flag == true)
        }
    }
}

struct Canary: Sendable {
    private let _flag: NIOLockedValueBox<Bool> = .init(false)
    public var flag: Bool {
        get { _flag.withLockedValue { $0 } }
        set { _flag.withLockedValue { $0 = newValue } }
    }

    public func trigger() {
        self._flag.withLockedValue { $0 = true }
    }
}
