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
//
//extension QueryBuilder {
//    public func paginate(
//        for request: Request
//    ) -> EventLoopFuture<Page<Model>> {
//        do {
//            let page = try request.query.decode(PageRequest.self)
//            return self.paginate(page)
//        } catch {
//            return request.eventLoop.makeFailedFuture(error)
//        }
//    }
//}

//extension Page: @retroactive /*Content,*/ ResponseEncodable, @retroactive RequestDecodable, @retroactive AsyncResponseEncodable where T: Codable {}

//extension Page: @retroactive AsyncRequestDecodable where T: Codable {
//    public static func decodeRequest(_ request: LibP2P.Request) async throws -> FluentKit.Page<T> {
//        // Read in payload
//
//        // Decode it
//
//        // Return it
//
//    }
//}
