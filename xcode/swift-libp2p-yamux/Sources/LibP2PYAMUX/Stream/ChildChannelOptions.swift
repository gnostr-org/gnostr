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

import NIOCore

/// The various channel options specific to `ChildChannel`s.
///
/// Please note that some of NIO's regular `ChannelOptions` are valid on `ChildChannel`s.
public struct ChildChannelOptions: Sendable {
    /// See: ``ChildChannelOptions/Types/LocalChannelIdentifierOption``.
    public static let localChannelIdentifier: ChildChannelOptions.Types.LocalChannelIdentifierOption = .init()

    /// See: ``ChildChannelOptions/Types/RemoteChannelIdentifierOption``.
    public static let remoteChannelIdentifier: ChildChannelOptions.Types.RemoteChannelIdentifierOption = .init()

    /// See: ``ChildChannelOptions/Types/PeerMaximumMessageLengthOption``.
    public static let peerMaximumMessageLength: ChildChannelOptions.Types.PeerMaximumMessageLengthOption = .init()
}

extension ChildChannelOptions {
    /// Types for the ``ChildChannelOptions``.
    public enum Types {}
}

extension ChildChannelOptions.Types {
    /// ``ChildChannelOptions/Types/LocalChannelIdentifierOption`` allows users to query the channel number assigned locally for a given channel.
    public struct LocalChannelIdentifierOption: ChannelOption, Sendable {
        public typealias Value = UInt32

        public init() {}
    }

    /// ``ChildChannelOptions/Types/RemoteChannelIdentifierOption`` allows users to query the channel number assigned by the remote peer for a given channel.
    public struct RemoteChannelIdentifierOption: ChannelOption, Sendable {
        public typealias Value = UInt32?

        public init() {}
    }

    /// ``ChildChannelOptions/Types/PeerMaximumMessageLengthOption`` allows users to query the maximum packet size value reported by the remote peer for a given channel.
    public struct PeerMaximumMessageLengthOption: ChannelOption, Sendable {
        public typealias Value = UInt32

        public init() {}
    }
}
