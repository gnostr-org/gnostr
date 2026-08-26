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

extension DatabaseID {
    static var test: Self {
        .init(string: "test")
    }
}

struct StaticDatabase: DatabaseConfiguration, DatabaseDriver {
    let database: any Database
    var middleware: [any AnyModelMiddleware] = []

    func makeDriver(for databases: Databases) -> any DatabaseDriver {
        self
    }

    func makeDatabase(with context: DatabaseContext) -> any Database {
        self.database
    }

    func shutdown() {
        // Do nothing.
    }

    func shutdownAsync() async {
        // Do nothing
    }
}
