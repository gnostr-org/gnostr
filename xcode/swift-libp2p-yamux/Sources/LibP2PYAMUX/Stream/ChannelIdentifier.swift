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
//===----------------------------------------------------------------------===//
//
// This source file is part of the SwiftNIO open source project
//
// Copyright (c) 2020 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

/// The identifier for a single YAMUX channel.
///
/// The client side should use odd ID's, and the server even.
/// Additionally, the 0 ID is reserved to represent the session.
struct ChannelIdentifier {
    /// The number used to identify this channel.
    var channelID: UInt32
}

extension ChannelIdentifier: Equatable {}

extension ChannelIdentifier: Hashable {}

extension ChannelIdentifier: CustomStringConvertible {
    var description: String {
        "ChannelIdentifier(\(self.channelID))"
    }
}
