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

import NIOCore

//extension ByteBuffer {
//    /// A helper function for complex readers that will reset a buffer on nil or on error, as though the read
//    /// never occurred.
//    internal mutating func rewindOnNilOrError<T>(_ body: (inout ByteBuffer) throws -> T?) rethrows -> T? {
//        let originalSelf = self
//
//        let returnValue: T?
//        do {
//            returnValue = try body(&self)
//        } catch {
//            self = originalSelf
//            throw error
//        }
//
//        if returnValue == nil {
//            self = originalSelf
//        }
//
//        return returnValue
//    }
//}
