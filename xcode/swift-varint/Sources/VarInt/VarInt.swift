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
//  VarInt.swift
//
//  Created by Teo on 01/10/15.
//  Licensed under MIT See LICENCE file in the root of this project for details.
//
//  Modified by Brandon Toms on 09/12/20.
//
//  This code provides "varint" encoding of 64-bit integers.
//     It is based on the Go implementation of the Google Protocol Buffers
//     varint specification.
//     Varints are a method of serializing integers using one or more bytes.
//     Smaller numbers take a smaller number of bytes.
//     For more details see ipfs/QmXJXJMai4p88HMsp2TPP1EtZxfSZQ1vyRtN5dGKvQ6MCw
//     The encoding is:
//     -   Unsigned integers are serialized 7 bits at a time, starting with the
//         least significant bits.
//     -   The most significant bit (msb) in each output byte indicates if there
//         is a continuation byte.
//     -   Signed integers are mapped to unsigned integers using "zig-zag" encoding:
//         Positive values x are written as 2*x + 0,
//         Negative values x are written as 2*(~x) + 1
//         So, negative values are complemented and whether to complement is encoded
//         in bit 0.

import Foundation

enum VarIntError: Error {
    case inputStreamRead
    case overflow
}

public typealias DecodedUVarInt = (value: UInt64, bytesRead: Int)
public typealias DecodedVarInt = (value: Int64, bytesRead: Int)

/// putVarInt encodes a UInt64 into a buffer and returns it.
public func putUVarInt(_ value: UInt64) -> [UInt8] {
    var buffer = [UInt8]()
    var val: UInt64 = value

    while val >= 0x80 {
        buffer.append((UInt8(truncatingIfNeeded: val) | 0x80))
        val >>= 7
    }

    buffer.append(UInt8(val))
    return buffer
}

/// uVarInt decodes an UInt64 from a byte buffer and returns the value and the number of bytes greater than 0 that were read.
/// - Note: If an error occurs the value will be 0 and the number of bytes n is <= 0 with the following meaning:
///   - n == 0: buf too small
///   - n  < 0: value larger than 64 bits (overflow)
///   - and -n is the number of bytes read
///
public func uVarInt(_ buffer: [UInt8]) -> DecodedUVarInt {
    var output: UInt64 = 0
    var counter = 0
    var shifter: UInt64 = 0

    for byte in buffer {
        if byte < 0x80 {
            if counter > 9 || counter == 9 && byte > 1 {
                return (0, -(counter + 1))
            }
            return (output | UInt64(byte) << shifter, counter + 1)
        }

        output |= UInt64(byte & 0x7f) << shifter
        shifter += 7
        counter += 1
    }
    return (0, 0)
}

/// putVarInt encodes an Int64 into a buffer and returns it.
public func putVarInt(_ value: Int64) -> [UInt8] {
    let unsignedValue = UInt64(value) << 1

    return putUVarInt(unsignedValue)
}

/// varInt decodes an Int64 from a byte buffer and returns the value and the number of bytes greater than 0 that were read.
/// - Note: If an error occurs the value will be 0 and the number of bytes n is <= 0 with the following meaning:
///   - n == 0: buf too small
///   - n  < 0: value larger than 64 bits (overflow) and -n is the number of bytes read
///
public func varInt(_ buffer: [UInt8]) -> DecodedVarInt {
    let (unsignedValue, bytesRead) = uVarInt(buffer)
    var value = Int64(unsignedValue >> 1)

    if unsignedValue & 1 != 0 { value = ~value }

    return (value, bytesRead)
}

/// readUVarInt reads an encoded unsigned integer from the reader and returns it as an UInt64
public func readUVarInt(_ reader: InputStream) throws -> UInt64 {
    var value: UInt64 = 0
    var shifter: UInt64 = 0
    var index = 0

    repeat {
        var buffer = [UInt8](repeating: 0, count: 10)

        if reader.read(&buffer, maxLength: 1) < 0 {
            throw VarIntError.inputStreamRead
        }

        let buf = buffer[0]

        if buf < 0x80 {
            if index > 9 || index == 9 && buf > 1 {
                throw VarIntError.overflow
            }
            return value | UInt64(buf) << shifter
        }
        value |= UInt64(buf & 0x7f) << shifter
        shifter += 7
        index += 1
    } while true
}

/// readVarInt reads an encoded signed integer from the reader and returns it as an Int64
public func readVarInt(_ reader: InputStream) throws -> Int64 {

    let unsignedValue = try readUVarInt(reader)
    var value = Int64(unsignedValue >> 1)

    if unsignedValue & 1 != 0 {
        value = ~value
    }

    return value
}

/// Computes the number of bytes that would be needed to store a 32-bit varint.
///
/// - Parameter value: The number whose varint size should be calculated.
/// - Returns: The size, in bytes, of the 32-bit varint.
public func encodedSize(of value: UInt32) -> Int {
    if (value & (~0 << 7)) == 0 {
        return 1
    }
    if (value & (~0 << 14)) == 0 {
        return 2
    }
    if (value & (~0 << 21)) == 0 {
        return 3
    }
    if (value & (~0 << 28)) == 0 {
        return 4
    }
    return 5
}

/// Computes the number of bytes that would be needed to store a signed 32-bit varint, if it were
/// treated as an unsigned integer with the same bit pattern.
///
/// - Parameter value: The number whose varint size should be calculated.
/// - Returns: The size, in bytes, of the 32-bit varint.
public func encodedSize(of value: Int32) -> Int {
    if value >= 0 {
        return encodedSize(of: UInt32(bitPattern: value))
    } else {
        // Must sign-extend.
        return encodedSize(of: Int64(value))
    }
}

/// Computes the number of bytes that would be needed to store a 64-bit varint.
///
/// - Parameter value: The number whose varint size should be calculated.
/// - Returns: The size, in bytes, of the 64-bit varint.
public func encodedSize(of value: Int64) -> Int {
    // Handle two common special cases up front.
    if (value & (~0 << 7)) == 0 {
        return 1
    }
    if value < 0 {
        return 10
    }

    // Divide and conquer the remaining eight cases.
    var value = value
    var n = 2

    if (value & (~0 << 35)) != 0 {
        n += 4
        value >>= 28
    }
    if (value & (~0 << 21)) != 0 {
        n += 2
        value >>= 14
    }
    if (value & (~0 << 14)) != 0 {
        n += 1
    }
    return n
}

/// Computes the number of bytes that would be needed to store an unsigned 64-bit varint, if it
/// were treated as a signed integer witht he same bit pattern.
///
/// - Parameter value: The number whose varint size should be calculated.
/// - Returns: The size, in bytes, of the 64-bit varint.
public func encodedSize(of value: UInt64) -> Int {
    encodedSize(of: Int64(bitPattern: value))
}

/// Counts the number of distinct varints in a packed byte buffer.
public func countVarintsInBuffer(start: UnsafePointer<UInt8>, count: Int) -> Int {
    // We don't need to decode all the varints to count how many there
    // are.  Just observe that every varint has exactly one byte with
    // value < 128. So we just count those...
    var n = 0
    var ints = 0
    while n < count {
        if start[n] < 128 {
            ints += 1
        }
        n += 1
    }
    return ints
}

extension Array where Element == UInt8 {
    /// Returns a UInt8 array as a string it's Binary representation
    /// ```
    /// Array<UInt8>[1].asBinaryChunks() -> "00000001"
    /// ```
    func asBinaryChunks() -> String {
        self.map {
            var str = String($0, radix: 2)
            if str.count < 8 { str = String(repeating: "0", count: 8 - str.count) + str }
            return str
        }.joined(separator: " ")
    }
}
