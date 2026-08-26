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

struct IPv6 {
    //static func data(for string: String) throws -> Data {
    //    guard let addr = IPv6Address(string) else { throw MultiaddrError.parseIPv6AddressFail }
    //    return addr.rawValue
    //}

    /// Converts an IPv4 string address into it's data representation
    ///
    /// - Note: This code was lifted from [Bouke/DNS](https://github.com/Bouke/DNS/blob/master/Sources/DNS/IP.swift)
    static func data(for string: String) throws -> Data {
        var address = in6_addr()
        guard inet_pton(AF_INET6, string, &address) == 1 else {
            throw MultiaddrError.parseIPv6AddressFail
        }
        #if os(Linux)
        return
            htonl(address.__in6_u.__u6_addr32.0).bytes + htonl(address.__in6_u.__u6_addr32.1).bytes
            + htonl(address.__in6_u.__u6_addr32.2).bytes + htonl(address.__in6_u.__u6_addr32.3).bytes
        #else
        return
            htonl(address.__u6_addr.__u6_addr32.0).bytes + htonl(address.__u6_addr.__u6_addr32.1).bytes
            + htonl(address.__u6_addr.__u6_addr32.2).bytes + htonl(address.__u6_addr.__u6_addr32.3).bytes
        #endif
    }

    static func string(for data: Data) throws -> String {
        guard data.count == MemoryLayout<in6_addr>.size else {
            throw MultiaddrError.parseIPv4AddressFail
        }

        var address = data.withUnsafeBytes { bytesPointer -> in6_addr in
            bytesPointer.load(as: in6_addr.self)
        }

        var output = Data(count: Int(INET6_ADDRSTRLEN))
        guard
            let presentationBytes = output.withUnsafeMutableBytes({
                #if swift(>=5.6)
                inet_ntop(AF_INET6, &address, $0.baseAddress, socklen_t(INET6_ADDRSTRLEN))
                #else
                inet_ntop(AF_INET6, &address, $0, socklen_t(INET6_ADDRSTRLEN))
                #endif
            })
        else {
            return "Invalid IPv6 address"
        }
        return String(cString: presentationBytes)
    }
}

/// Undefined for LE
private func htonl(_ value: UInt32) -> UInt32 {
    value.byteSwapped
}
