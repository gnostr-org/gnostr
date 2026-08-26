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
//  CryptoSwift
//

#if canImport(Darwin)
import Darwin
#elseif canImport(Glibc)
import Glibc
#elseif canImport(ucrt)
import ucrt
#endif

extension FixedWidthInteger {
    @inlinable
    func bytes(totalBytes: Int = MemoryLayout<Self>.size) -> [UInt8] {
        arrayOfBytes(value: self.littleEndian, length: totalBytes)
        // TODO: adjust bytes order
        // var value = self.littleEndian
        // return withUnsafeBytes(of: &value, Array.init).reversed()
    }
}

@_specialize(where T == Int)
@_specialize(where T == UInt)
@_specialize(where T == UInt8)
@_specialize(where T == UInt16)
@_specialize(where T == UInt32)
@_specialize(where T == UInt64)
@inlinable
func arrayOfBytes<T: FixedWidthInteger>(value: T, length totalBytes: Int = MemoryLayout<T>.size) -> [UInt8] {
    let valuePointer = UnsafeMutablePointer<T>.allocate(capacity: 1)
    valuePointer.pointee = value

    let bytesPointer = UnsafeMutablePointer<UInt8>(OpaquePointer(valuePointer))
    var bytes = [UInt8](repeating: 0, count: totalBytes)
    for j in 0..<min(MemoryLayout<T>.size, totalBytes) {
        bytes[totalBytes - 1 - j] = (bytesPointer + j).pointee
    }

    valuePointer.deinitialize(count: 1)
    valuePointer.deallocate()

    return bytes
}
