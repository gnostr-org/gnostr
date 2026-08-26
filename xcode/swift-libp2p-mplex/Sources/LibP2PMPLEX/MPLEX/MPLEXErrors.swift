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
// Copyright (c) 2017-2018 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

public protocol NIOMPLEXError: Equatable, Error {}

public enum MuxerError: Error {
    case custom(String)
}

/// Errors that NIO raises when handling MPLEX connections.
public enum NIOMPLEXErrors {

    public static func noSuchStream(streamID: MPLEXStreamID, file: String = #file, line: UInt = #line) -> NoSuchStream {
        NoSuchStream(streamID: streamID, file: file, line: line)
    }

    public static func streamClosed(
        streamID: MPLEXStreamID,
        errorCode: MPLEXErrorCode,
        file: String = #file,
        line: UInt = #line
    ) -> StreamClosed {
        StreamClosed(streamID: streamID, errorCode: errorCode, file: file, line: line)
    }

    public static func noStreamIDAvailable(file: String = #file, line: UInt = #line) -> NoStreamIDAvailable {
        NoStreamIDAvailable(file: file, line: line)
    }

    public static func streamError(streamID: MPLEXStreamID, baseError: Error) -> StreamError {
        StreamError(streamID: streamID, baseError: baseError)
    }

    /// An attempt was made to issue a write on a stream that does not exist.
    public struct NoSuchStream: NIOMPLEXError {
        /// The stream ID that was used that does not exist.
        public var streamID: MPLEXStreamID

        /// The location where the error was thrown.
        public let location: String

        @available(*, deprecated, renamed: "noSuchStream")
        public init(streamID: MPLEXStreamID) {
            self.init(streamID: streamID, file: #file, line: #line)
        }

        fileprivate init(streamID: MPLEXStreamID, file: String, line: UInt) {
            self.streamID = streamID
            self.location = _location(file: file, line: line)
        }

        public static func == (lhs: NoSuchStream, rhs: NoSuchStream) -> Bool {
            lhs.streamID == rhs.streamID
        }
    }

    /// A stream was closed.
    public struct StreamClosed: NIOMPLEXError {
        /// The stream ID that was closed.
        public var streamID: MPLEXStreamID

        /// The error code associated with the closure.
        public var errorCode: MPLEXErrorCode

        /// The file and line where the error was created.
        public let location: String

        @available(*, deprecated, renamed: "streamClosed")
        public init(streamID: MPLEXStreamID, errorCode: MPLEXErrorCode) {
            self.init(streamID: streamID, errorCode: errorCode, file: #file, line: #line)
        }

        fileprivate init(streamID: MPLEXStreamID, errorCode: MPLEXErrorCode, file: String, line: UInt) {
            self.streamID = streamID
            self.errorCode = errorCode
            self.location = _location(file: file, line: line)
        }

        public static func == (lhs: StreamClosed, rhs: StreamClosed) -> Bool {
            lhs.streamID == rhs.streamID && lhs.errorCode == rhs.errorCode
        }
    }

    /// The channel does not yet have a stream ID, as it has not reached the network yet.
    public struct NoStreamIDAvailable: NIOMPLEXError {
        private let file: String
        private let line: UInt

        /// The location where the error was thrown.
        public var location: String {
            _location(file: self.file, line: self.line)
        }

        @available(*, deprecated, renamed: "noStreamIDAvailable")
        public init() {
            self.init(file: #file, line: #line)
        }

        fileprivate init(file: String, line: UInt) {
            self.file = file
            self.line = line
        }

        public static func == (lhs: NoStreamIDAvailable, rhs: NoStreamIDAvailable) -> Bool {
            true
        }
    }

    /// A StreamError was hit during outbound frame processing.
    ///
    /// Stream errors are wrappers around another error of some other kind that occurred on a specific stream.
    /// As they are a wrapper error, they carry a "real" error in the `baseError`. Additionally, they cannot
    /// meaningfully be `Equatable`, so they aren't. There's also no additional location information: that's
    /// provided by the base error.
    public struct StreamError: Error {
        private final class Storage: Sendable {
            let streamID: MPLEXStreamID
            let baseError: Error

            init(streamID: MPLEXStreamID, baseError: Error) {
                self.baseError = baseError
                self.streamID = streamID
            }

            func copy() -> Storage {
                Storage(
                    streamID: self.streamID,
                    baseError: self.baseError
                )
            }
        }

        private var storage: Storage

        public var baseError: Error {
            get {
                self.storage.baseError
            }
        }

        public var streamID: MPLEXStreamID {
            get {
                self.storage.streamID
            }
        }

        public var description: String {
            "StreamError(streamID: \(self.streamID), baseError: \(self.baseError))"
        }

        fileprivate init(streamID: MPLEXStreamID, baseError: Error) {
            self.storage = .init(streamID: streamID, baseError: baseError)
        }
    }
}

/// This enum covers errors that are thrown internally for messaging reasons. These should
/// not leak.
internal enum InternalError: Error {
    case attemptedToCreateStream

    case codecError(code: MPLEXErrorCode)
}

extension InternalError: Hashable {}

private func _location(file: String, line: UInt) -> String {
    "\(file):\(line)"
}

private final class StringAndLocationStorage: Equatable {
    var value: String
    var file: String
    var line: UInt

    var location: String {
        _location(file: self.file, line: self.line)
    }

    init(_ value: String, file: String, line: UInt) {
        self.value = value
        self.file = file
        self.line = line
    }

    func copy() -> StringAndLocationStorage {
        StringAndLocationStorage(self.value, file: self.file, line: self.line)
    }

    static func == (lhs: StringAndLocationStorage, rhs: StringAndLocationStorage) -> Bool {
        // Only compare the value. The 'file' is not relevant here.
        lhs.value == rhs.value
    }
}
