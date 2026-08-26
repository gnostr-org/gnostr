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

import Foundation

extension PeerID: Equatable {
    public static func == (lhs: PeerID, rhs: PeerID) -> Bool {
        lhs.id == rhs.id || lhs.isEquivalent(to: rhs)
    }
    public static func == (lhs: [UInt8], rhs: PeerID) -> Bool {
        lhs == rhs.id
    }
    public static func == (lhs: Data, rhs: PeerID) -> Bool {
        lhs.byteArray == rhs.id
    }
}
