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
//  Created by swift-libp2p
//

import FluentKit
import Foundation
import LibP2P

final class PeerStoreEntry_Multiaddr: Model, @unchecked Sendable {
    public static let schema: String = "_fluent_peerstore_multiaddr"

    struct Create: Migration {
        func prepare(on database: any Database) -> EventLoopFuture<Void> {
            database.schema("_fluent_peerstore_multiaddr")
                .id()
                .field(
                    "peer_id",
                    .uuid,
                    .required,
                    .references("_fluent_peerstore", "id", onDelete: .cascade, onUpdate: .cascade)
                )
                .field("address", .string, .required)
                .unique(on: "peer_id", "address")
                .create()
        }

        func revert(on database: any Database) -> EventLoopFuture<Void> {
            database.schema("_fluent_peerstore_multiaddr").delete()
        }
    }

    public static var migration: any Migration {
        Create()
    }

    @ID(key: .id)
    public var id: UUID?

    @Parent(key: "peer_id")
    public var peer: PeerStoreEntry

    @Field(key: "address")
    public var address: String

    public init() {}

    public init(id: UUID? = nil, peerID: PeerStoreEntry.IDValue, address: Multiaddr) {
        self.$peer.id = peerID
        self.address = address.description
    }
}
