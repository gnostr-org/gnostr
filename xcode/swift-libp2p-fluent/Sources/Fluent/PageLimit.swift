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

public struct PageLimit: Sendable {
    public let value: Int?

    public static var noLimit: Self {
        .init(value: nil)
    }
}

extension PageLimit {
    public init(_ value: Int) {
        self.value = value
    }
}

extension PageLimit: ExpressibleByIntegerLiteral {
    public init(integerLiteral value: IntegerLiteralType) {
        self.value = value
    }
}
