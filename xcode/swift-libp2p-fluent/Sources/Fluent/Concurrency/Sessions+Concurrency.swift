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

//public import FluentKit
//public import LibP2P

//extension Model where Self: SessionAuthenticatable, Self.SessionID == Self.IDValue {
//    public static func asyncSessionAuthenticator(
//        _ databaseID: DatabaseID? = nil
//    ) -> any AsyncAuthenticator {
//        AsyncDatabaseSessionAuthenticator<Self>(databaseID: databaseID)
//    }
//}

//private struct AsyncDatabaseSessionAuthenticator<User>: AsyncSessionAuthenticator
//where User: SessionAuthenticatable, User: Model, User.SessionID == User.IDValue {
//    let databaseID: DatabaseID?
//
//    func authenticate(sessionID: User.SessionID, for request: Request) async throws {
//        if let user = try await User.find(sessionID, on: request.db(self.databaseID)) {
//            request.auth.login(user)
//        }
//    }
//}
