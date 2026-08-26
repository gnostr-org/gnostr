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
//  Original Asn1Parser.swift from SwiftyRSA
//
//  Created by Lois Di Qual on 5/9/17.
//  Copyright © 2017 Scoop. All rights reserved.
//
//  Modified by Brandon Toms on 5/1/22
//

import Foundation

enum ASN1 {
    internal enum IDENTIFIERS: UInt8, Equatable {
        case SEQUENCE = 0x30
        case INTEGER = 0x02
        case OBJECTID = 0x06
        case NULL = 0x05
        case BITSTRING = 0x03
        case OCTETSTRING = 0x04
        case EC_OBJECT = 0xA0
        case EC_BITS = 0xA1

        static func == (lhs: UInt8, rhs: IDENTIFIERS) -> Bool {
            lhs == rhs.rawValue
        }

        var bytes: [UInt8] {
            switch self {
            case .NULL:
                return [self.rawValue, 0x00]
            default:
                return [self.rawValue]
            }
        }
    }

    /// An ASN1 node
    enum Node: CustomStringConvertible {
        /// An array of more `ASN1.Node`s
        case sequence(nodes: [Node])
        /// An integer
        case integer(data: Data)
        /// An objectIdentifier
        case objectIdentifier(data: Data)
        /// A null object
        case null
        /// A bitString
        case bitString(data: Data)
        /// An octetString
        case octetString(data: Data)

        //Exteneded Params

        /// Elliptic Curve specific objectIdentifier
        case ecObject(data: Data)
        /// Elliptic Curve specific bitString
        case ecBits(data: Data)

        var description: String {
            ASN1.printNode(self, level: 0)
        }

    }

    internal static func printNode(_ node: ASN1.Node, level: Int) -> String {
        var str: [String] = []
        let prefix = String(repeating: "\t", count: level)
        switch node {
        case .integer(let int):
            str.append("\(prefix)Integer: \(int.toHexString())")
        case .bitString(let bs):
            str.append("\(prefix)BitString: \(bs.toHexString())")
        case .null:
            str.append("\(prefix)NULL")
        case .objectIdentifier(let oid):
            str.append("\(prefix)ObjectID: \(oid.toHexString())")
        case .octetString(let os):
            str.append("\(prefix)OctetString: \(os.toHexString())")
        case .ecObject(let ecObj):
            str.append("\(prefix)EC Object: \(ecObj.toHexString())")
        case .ecBits(let ecBits):
            str.append("\(prefix)EC Bits: \(ecBits.toHexString())")
        case .sequence(let nodes):
            str.append("\(prefix)Sequence:")
            for node in nodes {
                str.append(printNode(node, level: level + 1))
            }
        }
        return str.joined(separator: "\n")
    }
}
