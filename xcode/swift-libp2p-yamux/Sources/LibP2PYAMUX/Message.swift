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
// Copyright (c) 2019 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

import NIOCore

// MARK: - Types

enum Message: Equatable {
    enum ParsingError: Error {
        case unknownType(UInt8)
        case incorrectFormat
    }

    // Session Messages (StreamID == 0)
    case sessionOpen(SessionOpenMessage)
    case sessionOpenConfirmation(SessionOpenConfirmationMessage)
    case ping(SessionPingMessage)  // Ping
    case disconnect(SessionDisconnectMessage)  // GoAway

    // Channel Messages
    case channelOpen(ChannelOpenMessage)
    case channelOpenConfirmation(ChannelOpenConfirmationMessage)
    case channelOpenFailure(ChannelOpenFailureMessage)
    case channelWindowAdjust(ChannelWindowAdjustMessage)
    case channelData(ChannelDataMessage)
    case channelClose(ChannelCloseMessage)
    case channelReset(ChannelResetMessage)

    var rank: Int {
        switch self {
        case .sessionOpen: 0
        case .sessionOpenConfirmation: 1
        case .channelOpen: 2
        case .channelOpenConfirmation: 3
        case .channelOpenFailure: 4
        case .channelWindowAdjust: 5
        case .channelData: 6
        case .ping: 7
        case .channelClose: 8
        case .channelReset: 9
        case .disconnect: 10
        }
    }
}

extension Message {
    struct SessionOpenMessage: Equatable {
        // SESSION OPEN
        var payload: UInt32
    }

    struct SessionOpenConfirmationMessage: Equatable {
        // SESSION OPEN CONFIRMATION
        var payload: UInt32
    }

    struct SessionPingMessage: Equatable {
        // SESSION PING

        var payload: UInt32
    }

    struct SessionDisconnectMessage: Equatable {
        // SESSION DISCONNECT / GO AWAY

        var reason: UInt32
        var description: String
        var tag: String
    }

    struct ChannelOpenMessage: Equatable {
        // CHANNEL OPEN

        var senderChannel: UInt32
        var initialWindowSize: UInt32
        var maximumPacketSize: UInt32
    }

    struct ChannelOpenConfirmationMessage: Equatable {
        // CHANNEL OPEN CONFIRMATION

        var recipientChannel: UInt32
        var senderChannel: UInt32
        var initialWindowSize: UInt32
        var maximumPacketSize: UInt32
    }

    struct ChannelOpenFailureMessage: Equatable {
        // CHANNEL OPEN FAILURE

        var recipientChannel: UInt32
        var reasonCode: UInt32
        var description: String
        var language: String
    }

    struct ChannelWindowAdjustMessage: Equatable {
        // CHANNEL WINDOW ADJUST

        var recipientChannel: UInt32
        var bytesToAdd: UInt32
    }

    struct ChannelDataMessage: Equatable {
        // CHANNEL DATA

        var recipientChannel: UInt32
        var data: ByteBuffer
    }

    struct ChannelCloseMessage: Equatable {
        // CHANNEL CLOSE

        var recipientChannel: UInt32
    }

    struct ChannelResetMessage: Equatable {
        // CHANNEL RESET

        var recipientChannel: UInt32
        var reasonCode: UInt32
        var description: String
    }
}
