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

/// A state machine that manages the state of child channels.
///
/// Child channels move through a weird set of states, because they can "exist" before the protocol is aware of them. In particular, there
/// are a few strange states that don't correspond to what the wire protocol looks like.
struct ChildChannelStateMachine {
    private var state: State
    private var id: UInt32

    init(localChannelID: UInt32) {
        self.state = .idle(localChannelID: localChannelID)
        self.id = localChannelID
    }
}

extension ChildChannelStateMachine {
    fileprivate enum State: Hashable {
        /// `idle` represents a child channel that has been allocated locally, but where we haven't asked the wire
        /// protocol to do anything with it yet: that is, we have not requested the channel. Such a channel is purely
        /// virtual. It exists only because we're attempting to start a channel locally and we haven't configured it yet.
        case idle(localChannelID: UInt32)

        /// `requestedLocally` is a channel for which we have sent a channel request, but haven't received a response yet.
        /// This channel is "active on the network" in the sense that the remote peer will come to know of its existence,
        /// but it's not yet a real channel that can perform any kind of I/O, so from the perspective of the user of the
        /// channel this channel isn't active yet.
        case requestedLocally(localChannelID: UInt32)

        /// `requestedRemotely` is a channel for which the remote peer has sent a channel request, but we have not yet
        /// responded. This channel is also "active on the network" in the sense that if our initialization fails we have
        /// to take some kind of action to terminate this channel. However, this channel can't do I/O yet, so from the perspective
        /// of the user of the channel this channel isn't active yet.
        case requestedRemotely(channelID: ChannelIdentifier)

        /// `active` is a channel that has been both requested and accepted. Data can flow freely on this channel in both directions.
        /// We have neither sent nor received either a `CHANNEL_EOF` or a `CHANNEL_CLOSE` so all I/O is flowing appropriately.
        case active(channelID: ChannelIdentifier)

        /// `closedRemotely` is a channel where the remote peer has sent `CHANNEL_CLOSE` but we have not yet sent `CHANNEL_CLOSE` back.
        case closedRemotely(channelID: ChannelIdentifier)

        /// `closedLocally` is a channel where we have sent `CHANNEL_CLOSE` but have not yet received a `CHANNEL_CLOSE` back.
        case closedLocally(channelID: ChannelIdentifier)

        /// `closed` is a channel where we have both sent and received `CHANNEL_CLOSE`.
        /// No further activity is possible. The channel identifier may now be re-used.
        case closed(channelID: ChannelIdentifier)
    }
}

// MARK: Receiving frames

extension ChildChannelStateMachine {
    mutating func receiveChannelOpen(_ message: Message.ChannelOpenMessage) {
        // The channel open message is a request to open the channel. Receiving one means this child channel is on the server side, and this is a remotely-initiated channel.
        switch self.state {
        case .idle(localChannelID: let localID):
            self.state = .requestedRemotely(
                channelID: ChannelIdentifier(channelID: localID)
            )

        case .requestedLocally:
            // We precondition here because the rest of the code should prevent this from happening: there's no way to deliver a
            // channel open request for a channel we requested, because the message cannot carry our channel ID.
            preconditionFailure("Somehow received an open request for a locally-initiated channel")

        case .requestedRemotely, .active, .closedRemotely, .closedLocally, .closed:
            // As above, we precondition here because the rest of the code should prevent this from happening: there's no way to deliver
            // a channel open request for a channel that has a remote ID already!
            preconditionFailure("Received an open request for an active channel")
        }
    }

    mutating func receiveChannelOpenConfirmation(_ message: Message.ChannelOpenConfirmationMessage) throws {
        // Channel open confirmation is sent in response to us having requested an open channel.
        switch self.state {
        case .requestedLocally(localChannelID: let localID):
            precondition(message.recipientChannel == localID)
            self.state = .active(
                channelID: ChannelIdentifier(channelID: localID)
            )

        case .idle:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Open confirmation sent on idle channel"
            )

        case .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Open confirmation sent on remotely-initiated channel"
            )

        case .active, .closedLocally, .closedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Duplicate open confirmation received."
            )
        }
    }

    mutating func receiveChannelOpenFailure(_ message: Message.ChannelOpenFailureMessage) throws {
        // Channel open failure is sent in response to us having requested an open channel. This is an immediate
        // transition to closed.
        switch self.state {
        case .requestedLocally(localChannelID: let localID):
            precondition(message.recipientChannel == localID)
            self.state = .closed(channelID: ChannelIdentifier(channelID: localID))

        case .idle:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Open failure sent on idle channel")

        case .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Open failure sent on remotely-initiated channel"
            )

        case .active, .closedLocally, .closedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Duplicate open failure received.")
        }
    }

    mutating func receiveChannelClose(_ message: Message.ChannelCloseMessage) throws {
        // We can get channel close at any point after the channel is active.
        switch self.state {
        case .active(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closedRemotely(channelID: channelID)

        case .closedLocally(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .idle:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Received close on idle")

        case .requestedLocally, .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Received close before channel was open."
            )

        case .closedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Received close on closed channel.")
        }
    }

    mutating func receiveChannelReset(_ message: Message.ChannelResetMessage) throws {
        // We can get channel reset at almost any point
        switch self.state {
        case .requestedLocally(let channelID):
            let id = ChannelIdentifier(channelID: channelID)
            precondition(message.recipientChannel == id.channelID)
            self.state = .closed(channelID: id)

        case .active(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .closedRemotely(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .closedLocally(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .idle:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Received Reset on idle")

        case .requestedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Received Reset out of sequence.")
        }
    }

    mutating func receiveChannelWindowAdjust(_ message: Message.ChannelWindowAdjustMessage) throws {
        switch self.state {
        case .active(let channelID),
            .closedLocally(let channelID):
            precondition(message.recipientChannel == channelID.channelID)

        case .idle:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Received channel window adjust on idle"
            )

        case .requestedLocally, .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Received channel window adjust before channel was open."
            )

        case .closedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Received window adjust on closed channel."
            )
        }
    }

    mutating func receiveChannelData(_ message: Message.ChannelDataMessage) throws {
        switch self.state {
        case .active(let channelID),
            .closedLocally(let channelID):
            // We allow data in closed locally because there may be a timing problem here.
            precondition(message.recipientChannel == channelID.channelID)

        case .idle:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Received channel EOF on idle")

        case .requestedLocally, .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Received channel data before channel was open."
            )

        case .closedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Received data on closed channel.")
        }
    }
}

// MARK: Sending frames

extension ChildChannelStateMachine {
    mutating func sendChannelOpen(_ message: Message.ChannelOpenMessage) {
        // The channel open message is a request to open the channel. Sending one means this child channel is on the client side,
        // and this is a locally-initiated channel.
        switch self.state {
        case .idle(localChannelID: let localID):
            precondition(localID == message.senderChannel)
            self.state = .requestedLocally(localChannelID: localID)

        case .requestedLocally, .requestedRemotely, .active, .closedLocally, .closedRemotely, .closed:
            // The code should prevent us from sending channel open twice.
            preconditionFailure("Attempted to send duplicate channel open")
        }
    }

    mutating func sendChannelOpenConfirmation(_ message: Message.ChannelOpenConfirmationMessage) {
        // Channel open confirmation is sent by us in response to the peer having requested an open channel.
        switch self.state {
        case .requestedRemotely(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            precondition(message.senderChannel == channelID.channelID)
            self.state = .active(channelID: channelID)

        case .idle:
            // In the idle state we haven't either sent a channel open or received one. This is not really possible.
            preconditionFailure("Somehow received open confirmation for idle channel")

        case .requestedLocally:
            preconditionFailure("Sent open confirmation on locally initiated channel.")

        case .active, .closedLocally, .closedRemotely, .closed:
            preconditionFailure("Duplicate open confirmation sent.")
        }
    }

    mutating func sendChannelOpenFailure(_ message: Message.ChannelOpenFailureMessage) {
        // Channel open failure is sent in response to the peer having requested an open channel. This is an immediate
        // transition to closed.
        switch self.state {
        case .requestedRemotely(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .idle:
            // In the idle state we haven't either sent a channel open or received one. This is not really possible.
            preconditionFailure("Somehow received open confirmation for idle channel")

        case .requestedLocally:
            preconditionFailure("Sent open failure on locally initiated channel.")

        case .active, .closedLocally, .closedRemotely, .closed:
            preconditionFailure("Duplicate open failure sent.")
        }
    }

    mutating func sendChannelClose(_ message: Message.ChannelCloseMessage) throws {
        // We can send channel close at any point after the channel is active.
        switch self.state {
        case .active(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closedLocally(channelID: channelID)

        case .closedRemotely(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .idle(let channelID):
            self.state = .closed(channelID: .init(channelID: channelID))

        case .requestedLocally, .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Sent close before channel was open."
            )

        case .closedLocally, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Sent close on closed channel.")
        }
    }

    mutating func sendChannelReset(_ message: Message.ChannelResetMessage) throws {
        // We can send channel reset at any point after the channel is active.
        switch self.state {
        case .requestedLocally(let localChannelID):
            precondition(message.recipientChannel == localChannelID)
            self.state = .closed(channelID: .init(channelID: localChannelID))

        case .active(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .closedRemotely(let channelID):
            precondition(message.recipientChannel == channelID.channelID)
            self.state = .closed(channelID: channelID)

        case .idle:
            // In the idle state we haven't either sent a channel open or received one. This is not really possible.
            preconditionFailure("Somehow received channel reset for idle channel")

        case .requestedRemotely:
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Sent reset before channel was open."
            )

        case .closedLocally, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Sent reset on closed channel.")
        }
    }

    mutating func sendChannelWindowAdjust(_ message: Message.ChannelWindowAdjustMessage) throws {
        switch self.state {
        case .active(let channelID):
            precondition(message.recipientChannel == channelID.channelID)

        case .idle:
            // In the idle state we haven't either sent a channel open or received one. This is not really possible.
            preconditionFailure("Somehow received channel EOF for idle channel")

        case .requestedLocally, .requestedRemotely, .closedLocally, .closedRemotely, .closed:
            preconditionFailure("Sent channel window adjust on channel in invalid state")
        }
    }

    mutating func sendChannelData(_ message: Message.ChannelDataMessage) throws {
        switch self.state {
        case .active(let channelID):
            precondition(message.recipientChannel == channelID.channelID)

        case .closedLocally, .closedRemotely, .closed:
            throw YAMUX.Error.protocolViolation(protocolName: "channel", violation: "Sent data on closed channel.")

        case .idle:
            // In the idle state we haven't either sent a channel open or received one. This is not really possible.
            preconditionFailure("Somehow received channel EOF for idle channel")

        case .requestedLocally, .requestedRemotely:
            preconditionFailure("Sent data before channel active")
        }
    }
}

// MARK: Other state changes

extension ChildChannelStateMachine {
    /// Called when TCP EOF is received. This forcibly shuts down the channel from any state.
    ///
    /// Must not be called on closed channels.
    mutating func receiveTCPEOF() {
        switch self.state {
        case .closed:
            preconditionFailure("Channel already closed")

        case .idle(localChannelID: let localID),
            .requestedLocally(localChannelID: let localID):
            self.state = .closed(channelID: ChannelIdentifier(channelID: localID))

        case .requestedRemotely(let channelID),
            .active(let channelID),
            .closedLocally(let channelID),
            .closedRemotely(let channelID):
            self.state = .closed(channelID: channelID)
        }
    }
}

// MARK: Helper computed properties

extension ChildChannelStateMachine {
    /// Whether this channel is currently active on the network.
    var isActiveOnNetwork: Bool {
        switch self.state {
        case .idle, .closed:
            return false
        case .requestedLocally, .requestedRemotely, .active, .closedLocally, .closedRemotely:
            return true
        }
    }

    /// Whether this channel is closed.
    var isClosed: Bool {
        switch self.state {
        case .closed:
            return true
        case .idle, .requestedLocally, .requestedRemotely, .active, .closedLocally, .closedRemotely:
            return false
        }
    }

    /// Whether `Channel.isActive` should be true.
    var isActiveOnChannel: Bool {
        switch self.state {
        case .active, .closedLocally, .closedRemotely:
            return true
        case .idle, .requestedLocally, .requestedRemotely, .closed:
            return false
        }
    }

    // Whether we've sent a channel close message.
    var sentClose: Bool {
        switch self.state {
        case .closedLocally, .closed:
            return true
        case .idle, .requestedLocally, .requestedRemotely, .active, .closedRemotely:
            return false
        }
    }

    /// The local identifier for this channel. We always know this identifier.
    var localChannelIdentifier: UInt32 {
        switch self.state {
        case .idle(localChannelID: let localID),
            .requestedLocally(localChannelID: let localID):
            return localID

        case .requestedRemotely(let channelID),
            .active(let channelID),
            .closedRemotely(let channelID),
            .closedLocally(let channelID),
            .closed(let channelID):
            return channelID.channelID
        }
    }

    // The remote identifier for this channel. We only know this when the remote peer has told us.
    var remoteChannelIdentifier: UInt32? {
        switch self.state {
        case .idle, .requestedLocally:
            return nil

        case .requestedRemotely(let channelID),
            .active(let channelID),
            .closedRemotely(let channelID),
            .closedLocally(let channelID),
            .closed(let channelID):
            return channelID.channelID
        }
    }

    // Whether we have activated yet.
    var awaitingActivation: Bool {
        switch self.state {
        case .idle, .requestedLocally, .requestedRemotely:
            return true

        case .active, .closedLocally, .closedRemotely, .closed:
            return false
        }
    }
}

extension ChildChannelStateMachine {
    /// An action to take in response to a specific operation.
    enum Action {
        /// Ignore this message: do nothing.
        case ignore

        /// The message should be processed as normal.
        case process
    }
}

extension ChildChannelStateMachine: CustomStringConvertible {
    public var description: String {
        "ChildChannelState[\(self.id)]: \(self.state)"
    }
}
