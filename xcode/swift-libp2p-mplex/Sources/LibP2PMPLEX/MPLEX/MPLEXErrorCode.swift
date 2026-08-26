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
//===----------------------------------------------------------------------===//
//
// This source file is part of the SwiftNIO open source project
//
// Copyright (c) 2017-2021 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

import NIOConcurrencyHelpers
import NIOCore

/// An MPLEX error code.
public struct MPLEXErrorCode: Sendable {
    /// The underlying network representation of the error code.
    public var networkCode: Int {
        get {
            Int(self._networkCode.withLockedValue { $0 })
        }
        set {
            self._networkCode.withLockedValue { $0 = UInt32(newValue) }
        }
    }

    /// The underlying network representation of the error code.
    fileprivate let _networkCode: NIOLockedValueBox<UInt32>

    /// Create a MPELX error code from the given network value.
    public init(networkCode: Int) {
        self._networkCode = .init(UInt32(networkCode))
    }

    /// Create a `MPLEXErrorCode` from the 32-bit integer it corresponds to.
    internal init(_ networkInteger: UInt32) {
        self._networkCode = .init(networkInteger)
    }

    /// The associated condition is not a result of an error. For example,
    /// a GOAWAY might include this code to indicate graceful shutdown of
    /// a connection.
    public static let noError = MPLEXErrorCode(networkCode: 0x0)

    /// The endpoint detected an unspecific protocol error. This error is
    /// for use when a more specific error code is not available.
    public static let protocolError = MPLEXErrorCode(networkCode: 0x01)

    /// The endpoint encountered an unexpected internal error.
    public static let internalError = MPLEXErrorCode(networkCode: 0x02)

    /// The endpoint sent a SETTINGS frame but did not receive a
    /// response in a timely manner.
    public static let settingsTimeout = MPLEXErrorCode(networkCode: 0x04)

    /// The endpoint received a frame after a stream was half-closed.
    public static let streamClosed = MPLEXErrorCode(networkCode: 0x05)

    /// The endpoint received a frame with an invalid size.
    public static let frameSizeError = MPLEXErrorCode(networkCode: 0x06)

    /// The endpoint refused the stream prior to performing any
    /// application processing.
    public static let refusedStream = MPLEXErrorCode(networkCode: 0x07)

    /// Used by the endpoint to indicate that the stream is no
    /// longer needed.
    public static let cancel = MPLEXErrorCode(networkCode: 0x08)

    /// The connection established in response to a CONNECT request
    /// was reset or abnormally closed.
    public static let connectError = MPLEXErrorCode(networkCode: 0x0a)

    /// The underlying transport has properties that do not meet
    /// minimum security requirements.
    public static let inadequateSecurity = MPLEXErrorCode(networkCode: 0x0c)

}

extension MPLEXErrorCode: Equatable {
    public static func == (lhs: MPLEXErrorCode, rhs: MPLEXErrorCode) -> Bool {
        lhs.networkCode == rhs.networkCode
    }
}

extension MPLEXErrorCode: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(self.networkCode)
    }
}

extension MPLEXErrorCode: CustomDebugStringConvertible {
    public var debugDescription: String {
        let errorCodeDescription: String
        switch self {
        case .noError:
            errorCodeDescription = "No Error"
        case .protocolError:
            errorCodeDescription = "ProtocolError"
        case .internalError:
            errorCodeDescription = "Internal Error"
        case .settingsTimeout:
            errorCodeDescription = "Settings Timeout"
        case .streamClosed:
            errorCodeDescription = "Stream Closed"
        case .frameSizeError:
            errorCodeDescription = "Frame Size Error"
        case .refusedStream:
            errorCodeDescription = "Refused Stream"
        case .cancel:
            errorCodeDescription = "Cancel"
        case .connectError:
            errorCodeDescription = "Connect Error"
        case .inadequateSecurity:
            errorCodeDescription = "Inadequate Security"
        default:
            errorCodeDescription = "Unknown Error"
        }

        return "MPLEXErrorCode<0x\(String(self.networkCode, radix: 16)) \(errorCodeDescription)>"
    }
}

extension UInt32 {
    /// Create a 32-bit integer corresponding to the given `MPLEXErrorCode`.
    public init(mplexErrorCode code: MPLEXErrorCode) {
        self = code._networkCode.withLockedValue { $0 }
    }
}

extension Int {
    /// Create an integer corresponding to the given `MPLEXErrorCode`.
    public init(mplexErrorCode code: MPLEXErrorCode) {
        self = code.networkCode
    }
}

extension ByteBuffer {
    /// Serializes a `MPLEXErrorCode` into a `ByteBuffer` in the appropriate endianness
    /// for use in MPLEX.
    ///
    /// - parameters:
    ///     - code: The `MPLEXErrorCode` to serialize.
    /// - returns: The number of bytes written.
    public mutating func write(mplexErrorCode code: MPLEXErrorCode) -> Int {
        self.writeInteger(UInt32(mplexErrorCode: code))
    }
}
