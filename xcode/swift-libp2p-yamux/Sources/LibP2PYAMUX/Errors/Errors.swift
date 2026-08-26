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

/// An error thrown by YAMUX.
///
/// For extensibility purposes, ``YAMUX.Error``s are composed of two parts. The first part is an
/// error type. This is like an enum, but extensible, and identifies the kind of error programmatically.
/// The second part is some opaque diagnostic data. This is not visible to your code, but is used to
/// help provide extra information for diagnostic purposes when logging this error.
///
/// Note that due to this construction ``YAMUX.Error`` is not equatable: only the ``YAMUX.Error/type`` is. This is deliberate,
/// as it is possible two errors have the same type but a different underlying cause or diagnostic data. For
/// this reason, if you need to compare two ``YAMUX.Error`` values you should explicitly compare their ``YAMUX.Error/type``.
extension YAMUX {
    public struct Error: Swift.Error {
        /// The type of this error, used to identify the kind of error that has been thrown.
        public var type: ErrorType

        private var diagnostics: String?
    }
}

// MARK: - Internal helper functions for error construction.

// These are never inlined as they are inherently cold path functions.
extension YAMUX.Error {
    @inline(never)
    internal static func invalidMessage(reason: String) -> YAMUX.Error {
        YAMUX.Error(type: .invalidMessage, diagnostics: reason)
    }

    internal static let invalidPacketFormat = YAMUX.Error(type: .invalidPacketFormat, diagnostics: nil)

    @inline(never)
    internal static func protocolViolation(protocolName: String, violation: String) -> YAMUX.Error {
        YAMUX.Error(type: .protocolViolation, diagnostics: "Protocol \(protocolName) violated due to \(violation)")
    }

    @inline(never)
    internal static func unsupportedVersion(_ version: String) -> YAMUX.Error {
        YAMUX.Error(
            type: .unsupportedVersion,
            diagnostics: "Version \(version) offered by the remote peer is not supported"
        )
    }

    @inline(never)
    internal static func channelSetupRejected(reasonCode: UInt32, reason: String) -> YAMUX.Error {
        YAMUX.Error(type: .channelSetupRejected, diagnostics: "Reason: \(reasonCode) \(reason)")
    }

    @inline(never)
    internal static func flowControlViolation(currentWindow: UInt32, increment: UInt32) -> YAMUX.Error {
        YAMUX.Error(
            type: .flowControlViolation,
            diagnostics: "Window size \(currentWindow), bad increment \(increment)"
        )
    }

    internal static let creatingChannelAfterClosure = YAMUX.Error(type: .creatingChannelAfterClosure, diagnostics: nil)

    internal static let tcpShutdown = YAMUX.Error(type: .tcpShutdown, diagnostics: nil)

    internal static let unknownChildChannel = YAMUX.Error(type: .unknownChildChannel, diagnostics: nil)

    @inline(never)
    internal static func unknownPacketType(diagnostic: String) -> YAMUX.Error {
        YAMUX.Error(type: .unknownPacketType, diagnostics: diagnostic)
    }

    @inline(never)
    internal static func unknownPacketFlag(diagnostic: String) -> YAMUX.Error {
        YAMUX.Error(type: .unknownPacketFlag, diagnostics: diagnostic)
    }

    @inline(never)
    internal static func unsupportedChannelEvent(event: String) -> YAMUX.Error {
        YAMUX.Error(type: .unsupportedChannelEvent, diagnostics: event)
    }
}

// MARK: - YAMUX.Error CustomStringConvertible conformance.

extension YAMUX.Error: CustomStringConvertible {
    public var description: String {
        "YAMUX.Error.\(self.type.description)\(self.diagnostics.map { ": \($0)" } ?? "")"
    }
}

// MARK: - Definition of YAMUX.Error.ErrorType

extension YAMUX.Error {
    /// The types of YAMUX.Error that can be encountered.
    public struct ErrorType {
        private enum Base: Hashable {
            case invalidMessage
            case invalidPacketFormat
            case protocolViolation
            case unsupportedVersion
            case channelSetupRejected
            case flowControlViolation
            case creatingChannelAfterClosure
            case tcpShutdown
            case unknownChildChannel
            case unknownPacketType
            case unknownPacketFlag
            case unsupportedChannelEvent
        }

        private var base: Base

        private init(_ base: Base) {
            self.base = base
        }

        /// An invalid message was received.
        public static let invalidMessage: ErrorType = .init(.invalidMessage)

        /// The packet format is invalid.
        public static let invalidPacketFormat: ErrorType = .init(.invalidPacketFormat)

        /// One of the YAMUX protocols was violated.
        public static let protocolViolation: ErrorType = .init(.protocolViolation)

        /// The version offered by the remote peer is unsupported by this implementation.
        public static let unsupportedVersion: ErrorType = .init(.unsupportedVersion)

        /// The remote peer rejected a request to setup a new channel.
        public static let channelSetupRejected: ErrorType = .init(.channelSetupRejected)

        /// The remote peer violated the YAMUX flow control rules.
        public static let flowControlViolation: ErrorType = .init(.flowControlViolation)

        /// The user attempted to create a channel after the handler was closed.
        public static let creatingChannelAfterClosure: ErrorType = .init(.creatingChannelAfterClosure)

        /// The TCP connection was shut down without cleanly closing the YAMUX session.
        public static let tcpShutdown: ErrorType = .init(.tcpShutdown)

        /// Our parent application requested a ChildChannel that we're unaware of (one that our muxer didn't spawn).
        public static let unknownChildChannel: ErrorType = .init(.unknownChildChannel)

        /// An packet type that we don't recognise was received.
        public static let unknownPacketType: ErrorType = .init(.unknownPacketType)

        /// An packet flag that we don't recognise was received.
        public static let unknownPacketFlag: ErrorType = .init(.unknownPacketFlag)

        /// We don't support Channel Events at the moment.
        public static let unsupportedChannelEvent: ErrorType = .init(.unsupportedChannelEvent)
    }
}

// MARK: - YAMUX.Error.ErrorType Hashable conformance

extension YAMUX.Error.ErrorType: Hashable {}

// MARK: - YAMUX.Error.ErrorType Sendable conformance

extension YAMUX.Error.ErrorType: Sendable {}

// MARK: - YAMUX.Error.ErrorType CustomStringConvertible conformance

extension YAMUX.Error.ErrorType: CustomStringConvertible {
    public var description: String {
        String(describing: self.base)
    }
}

extension YAMUX.Error: Equatable {}
