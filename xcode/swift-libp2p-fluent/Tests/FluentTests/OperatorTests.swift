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

@Suite("Operator Tests")
struct OperatorTests {

    @Test func customOperators() throws {
        // TODO: What does this test...?
        let db = DummyDatabase()

        // name contains string anywhere, prefix, suffix
        #expect(
            Planet.query(on: db)
                .filter(\.$name ~~ "art").query.description
                == #"query read planets filters=[planets[name] contains "art"]"#
        )
        #expect(
            Planet.query(on: db)
                .filter(\.$name =~ "art").query.description
                == #"query read planets filters=[planets[name] startswith "art"]"#
        )
        #expect(
            Planet.query(on: db)
                .filter(\.$name ~= "art").query.description
                == #"query read planets filters=[planets[name] endswith "art"]"#
        )

        // name doesn't contain string anywhere, prefix, suffix
        #expect(
            Planet.query(on: db)
                .filter(\.$name !~ "art").query.description
                == #"query read planets filters=[planets[name] !contains "art"]"#
        )
        #expect(
            Planet.query(on: db)
                .filter(\.$name !=~ "art").query.description
                == #"query read planets filters=[planets[name] !startswith "art"]"#
        )
        #expect(
            Planet.query(on: db)
                .filter(\.$name !~= "art").query.description
                == #"query read planets filters=[planets[name] !endswith "art"]"#
        )

        // name in array
        #expect(
            Planet.query(on: db)
                .filter(\.$name ~~ ["Earth", "Mars"]).query.description
                == #"query read planets filters=[planets[name] ~~ ["Earth", "Mars"]]"#
        )

        // name not in array
        #expect(
            Planet.query(on: db)
                .filter(\.$name !~ ["Earth", "Mars"]).query.description
                == #"query read planets filters=[planets[name] !~~ ["Earth", "Mars"]]"#
        )
    }

    @Test func customOps() async throws {
        try await withApp { app in

            // Setup test db.
            let test = CallbackTestDatabase { query in
                // return the plaintext description of the query
                [TestOutput(["res": "\(query)"])]
            }
            app.databases.use(test.configuration, as: .test)

            let args: [String: QueryBuilder<Planet>] = [
                // name contains string anywhere, prefix, suffix
                #"query read planets filters=[planets[name] contains "art"]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name ~~ "art"),

                #"query read planets filters=[planets[name] startswith "art"]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name =~ "art"),

                #"query read planets filters=[planets[name] endswith "art"]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name ~= "art"),

                // name doesn't contain string anywhere, prefix, suffix
                #"query read planets filters=[planets[name] !contains "art"]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name !~ "art"),

                #"query read planets filters=[planets[name] !startswith "art"]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name !=~ "art"),

                #"query read planets filters=[planets[name] !endswith "art"]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name !~= "art"),

                // name in array
                #"query read planets filters=[planets[name] ~~ ["Earth", "Mars"]]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name ~~ ["Earth", "Mars"]),

                // name not in array
                #"query read planets filters=[planets[name] !~~ ["Earth", "Mars"]]"#:
                    Planet.query(on: app.db)
                    .filter(\.$name !~ ["Earth", "Mars"]),
            ]

            for (desc, query) in args {
                try await ensure(query: query.query, equalsDescription: desc, on: app)
            }
        }

        func ensure(query: DatabaseQuery, equalsDescription desc: String, on app: Application) async throws {
            try await app.db.execute(query: query) { res in
                do {
                    let queryDesc = try res.decode("res", as: String.self)
                    #expect(queryDesc == desc)
                } catch {
                    Issue.record(error)
                }
            }.get()
        }
    }
}

struct RawQueryResult: Codable {
    let query: String
}

private final class Planet: Model, @unchecked Sendable {
    static let schema = "planets"

    @ID(custom: .id)
    var id: Int?

    @Field(key: "name")
    var name: String
}

//private struct DummyDatabase: Database {
//    var inTransaction: Bool {
//        false
//    }
//
//    var context: DatabaseContext {
//        .init(
//            configuration: DummyDatabaseConfiguration(),
//            logger: Logger(label: "fluent"),
//            eventLoop: MultiThreadedEventLoopGroup.singleton.any()
//        )
//    }
//
//    func execute(
//        query: DatabaseQuery,
//        onOutput: @escaping @Sendable (any DatabaseOutput) -> Void
//    ) -> EventLoopFuture<Void> {
//        fatalError()
//    }
//
//    func execute(schema: DatabaseSchema) -> EventLoopFuture<Void> {
//        fatalError()
//    }
//
//    func execute(enum: DatabaseEnum) -> EventLoopFuture<Void> {
//        fatalError()
//    }
//
//    func withConnection<T>(_ closure: @escaping @Sendable (any Database) -> EventLoopFuture<T>) -> EventLoopFuture<T> {
//        fatalError()
//    }
//
//    func transaction<T>(_ closure: @escaping @Sendable (any Database) -> EventLoopFuture<T>) -> EventLoopFuture<T> {
//        fatalError()
//    }
//}
//
//private struct DummyDatabaseConfiguration: DatabaseConfiguration {
//    var middleware: [any FluentKit.AnyModelMiddleware] = []
//
//    func makeDriver(for databases: FluentKit.Databases) -> any FluentKit.DatabaseDriver {
//        DummyDatabaseDriver()
//    }
//}
//
//private struct DummyDatabaseDriver: DatabaseDriver {
//    func makeDatabase(with context: FluentKit.DatabaseContext) -> any FluentKit.Database {
//        DummyDatabase()
//    }
//
//    func shutdown() {}
//}
