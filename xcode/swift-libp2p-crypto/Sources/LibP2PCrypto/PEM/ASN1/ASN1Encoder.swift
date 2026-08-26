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

import Foundation

extension ASN1 {
    enum Encoder {
        /// Encodes an ASN1Node into its byte representation
        ///
        /// - Parameter node: The Node to encode
        /// - Returns: The encoded bytes as a UInt8 array
        public static func encode(_ node: ASN1.Node) -> [UInt8] {
            switch node {
            case .integer(let integer):
                return IDENTIFIERS.INTEGER.bytes + asn1LengthPrefixed(integer.byteArray)
            case .bitString(let bits):
                return IDENTIFIERS.BITSTRING.bytes + asn1LengthPrefixed([0x00] + bits.byteArray)
            case .octetString(let octet):
                return IDENTIFIERS.OCTETSTRING.bytes + asn1LengthPrefixed(octet.byteArray)
            case .null:
                return IDENTIFIERS.NULL.bytes
            case .objectIdentifier(let oid):
                return IDENTIFIERS.OBJECTID.bytes + asn1LengthPrefixed(oid.byteArray)
            case .ecObject(let ecObj):
                return IDENTIFIERS.EC_OBJECT.bytes + asn1LengthPrefixed(ecObj.byteArray)
            case .ecBits(let ecBits):
                return IDENTIFIERS.EC_BITS.bytes + asn1LengthPrefixed(ecBits.byteArray)
            case .sequence(let nodes):
                return IDENTIFIERS.SEQUENCE.bytes
                    + asn1LengthPrefixed(
                        nodes.reduce(
                            into: [UInt8](),
                            { partialResult, node in
                                partialResult += encode(node)
                            }
                        )
                    )
            }
        }

        /// Calculates and returns the ASN.1 length Prefix for a chunk of data
        private static func asn1LengthPrefix(_ bytes: [UInt8]) -> [UInt8] {
            if bytes.count >= 0x80 {
                var lengthAsBytes = withUnsafeBytes(of: bytes.count.bigEndian, Array<UInt8>.init)
                while lengthAsBytes.first == 0 { lengthAsBytes.removeFirst() }
                return [(0x80 + UInt8(lengthAsBytes.count))] + lengthAsBytes
            } else {
                return [UInt8(bytes.count)]
            }
        }

        /// Returns the provided bytes with the appropriate ASN.1 length prefix prepended
        private static func asn1LengthPrefixed(_ bytes: [UInt8]) -> [UInt8] {
            asn1LengthPrefix(bytes) + bytes
        }
    }
}
