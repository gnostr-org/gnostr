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
// Copyright (c) 2017-2021 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

import NIOCore

/// An YAMUX network error code.
extension YAMUX {
    public struct NetworkError: Sendable {
        /// The underlying network representation of the error code.
        public let code: UInt32

        /// Create a YAMUX error code from the given network value.
        public init(networkCode: Int) {
            self.code = UInt32(networkCode)
        }

        /// Create a `NetworkError` from the 32-bit integer it corresponds to.
        internal init(_ networkInteger: UInt32) {
            self.code = networkInteger
        }

        /// The associated condition is not a result of an error. For example,
        /// a GOAWAY might include this code to indicate graceful shutdown of
        /// a connection.
        public static let noError = NetworkError(networkCode: 0x0)

        /// The endpoint detected an unspecific protocol error. This error is
        /// for use when a more specific error code is not available.
        public static let protocolError = NetworkError(networkCode: 0x01)

        /// The endpoint encountered an unexpected internal error.
        public static let internalError = NetworkError(networkCode: 0x02)
    }
}

extension YAMUX.NetworkError: Equatable {}

extension YAMUX.NetworkError: Hashable {}

extension YAMUX.NetworkError: CaseIterable {
    public static var allCases: [YAMUX.NetworkError] {
        [.noError, .protocolError, .internalError]
    }
}

extension YAMUX.NetworkError: CustomDebugStringConvertible {
    public var debugDescription: String {
        let errorCodeDescription: String
        switch self {
        case .noError:
            errorCodeDescription = "No Error"
        case .protocolError:
            errorCodeDescription = "ProtocolError"
        case .internalError:
            errorCodeDescription = "Internal Error"
        default:
            errorCodeDescription = "Unknown Error"
        }

        return "ErrorCode<0x\(String(self.code, radix: 16)) \(errorCodeDescription)>"
    }
}

extension ByteBuffer {
    /// Serializes a `NetworkError` into a `ByteBuffer` in the appropriate endianness
    /// for use in YAMUX.
    ///
    /// - parameters:
    ///     - code: The `NetworkError` to serialize.
    /// - returns: The number of bytes written.
    public mutating func write(networkError error: YAMUX.NetworkError) -> Int {
        self.writeInteger(error.code, as: UInt32.self)
    }
}
