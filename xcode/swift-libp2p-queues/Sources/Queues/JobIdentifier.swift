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

import struct Foundation.UUID

/// An identifier for a job
public struct JobIdentifier: Hashable, Equatable, Sendable {
    /// The string value of the ID
    public let string: String

    /// Creates a new id from a string
    public init(string: String) {
        self.string = string
    }

    /// Creates a new id with a default UUID value
    public init() {
        self.init(string: UUID().uuidString)
    }
}
