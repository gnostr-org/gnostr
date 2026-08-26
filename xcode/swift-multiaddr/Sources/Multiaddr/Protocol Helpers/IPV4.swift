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
//  Created by Luke Reichold
//  Modified by Brandon Toms on 5/1/22.
//

import Foundation

//import Network

struct IPv4 {
    //static func data(for string: String) throws -> Data {
    //    guard let addr = IPv4Address(string) else { throw MultiaddrError.parseIPv4AddressFail }
    //    return addr.rawValue
    //}

    /// Converts an IPv4 string address into it's data representation
    ///
    /// - Note: This code was lifted from [Bouke/DNS](https://github.com/Bouke/DNS/blob/master/Sources/DNS/IP.swift)
    static func data(for string: String) throws -> Data {
        var address = in_addr()
        guard inet_pton(AF_INET, string, &address) == 1 else {
            throw MultiaddrError.parseIPv4AddressFail
        }
        return address.s_addr.byteSwapped.bytes
    }

    static func string(for data: Data) throws -> String {
        guard data.count == MemoryLayout<UInt32>.size else {
            throw MultiaddrError.parseIPv4AddressFail
        }
        guard let uint32 = data.uint32 else {
            throw MultiaddrError.parseIPv4AddressFail
        }
        var output = Data(count: Int(INET_ADDRSTRLEN))
        var address = in_addr(s_addr: uint32)

        guard
            let presentationBytes = output.withUnsafeMutableBytes({
                #if swift(>=5.6)
                inet_ntop(AF_INET, &address, $0.baseAddress, socklen_t(INET_ADDRSTRLEN))
                #else
                inet_ntop(AF_INET, &address, $0, socklen_t(INET_ADDRSTRLEN))
                #endif
            })
        else {
            return "Invalid IPv4 address"
        }
        return String(cString: presentationBytes)
    }
}

extension BinaryInteger {
    // returns little endian; use .bigEndian.bytes for BE.
    var bytes: Data {
        var copy = self
        return withUnsafePointer(to: &copy) {
            Data(Data(bytes: $0, count: MemoryLayout<Self>.size).reversed())
        }
    }
}
