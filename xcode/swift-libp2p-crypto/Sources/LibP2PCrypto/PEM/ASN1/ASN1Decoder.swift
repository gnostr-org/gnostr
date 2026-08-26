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
    /// A simple ASN1 parser that will recursively iterate over a root node and return a Node tree.
    /// The root node can be any of the supported nodes described in `Node`. If the parser encounters a sequence
    /// it will recursively parse its children.
    enum Decoder {

        enum DecodingError: Error {
            case noType
            case invalidType(value: UInt8)
        }

        /// Parses ASN1 data and returns its root node.
        ///
        /// - Parameter data: ASN1 data to parse
        /// - Returns: Root ASN1 Node
        /// - Throws: A DecodingError if anything goes wrong, or if an unknown node was encountered
        static func decode(data: Data) throws -> Node {
            let scanner = Scanner(data: data)
            let node = try decodeNode(scanner: scanner)
            return node
        }

        /// Parses an ASN1 given an existing scanner.
        /// @warning: this will modify the state (ie: position) of the provided scanner.
        ///
        /// - Parameter scanner: Scanner to use to consume the data
        /// - Returns: Parsed node
        /// - Throws: A DecodingError if anything goes wrong, or if an unknown node was encountered
        private static func decodeNode(scanner: Scanner) throws -> Node {

            let firstByte = try scanner.consume(length: 1).firstByte

            // Sequence
            if firstByte == IDENTIFIERS.SEQUENCE {
                let length = try scanner.consumeLength()
                let data = try scanner.consume(length: length)
                let nodes = try decodeSequence(data: data)
                return .sequence(nodes: nodes)
            }

            // Integer
            if firstByte == IDENTIFIERS.INTEGER {
                let length = try scanner.consumeLength()
                let data = try scanner.consume(length: length)
                return .integer(data: data)
            }

            // Object identifier
            if firstByte == IDENTIFIERS.OBJECTID {
                let length = try scanner.consumeLength()
                let data = try scanner.consume(length: length)
                return .objectIdentifier(data: data)
            }

            // Null
            if firstByte == IDENTIFIERS.NULL {
                _ = try scanner.consume(length: 1)
                return .null
            }

            // Bit String
            if firstByte == IDENTIFIERS.BITSTRING {
                let length = try scanner.consumeLength()

                // There's an extra byte (0x00) after the bit string length in all the keys I've encountered.
                // I couldn't find a specification that referenced this extra byte, but let's consume it and discard it.
                _ = try scanner.consume(length: 1)

                let data = try scanner.consume(length: length - 1)
                return .bitString(data: data)
            }

            // Octet String
            if firstByte == IDENTIFIERS.OCTETSTRING {
                let length = try scanner.consumeLength()
                let data = try scanner.consume(length: length)
                return .octetString(data: data)
            }

            // EC Curves Cont 0 identifier (obj id)
            if firstByte == IDENTIFIERS.EC_OBJECT {
                let length = try scanner.consumeLength()
                let data = try scanner.consume(length: length)
                //print("Found an EC Curve Obj ID: [\(data.map { "\($0)" }.joined(separator: ","))]")
                //return .ecObject(data: data)
                return .objectIdentifier(data: data)
            }

            // EC Curves Cont 1 identifier (bit string)
            if firstByte == IDENTIFIERS.EC_BITS {
                let length = try scanner.consumeLength()

                // There's an extra byte (0x00) after the bit string length in all the keys I've encountered.
                // I couldn't find a specification that referenced this extra byte, but let's consume it and discard it.
                _ = try scanner.consume(length: 1)

                let data = try scanner.consume(length: length - 1)
                //print("Found an EC Curve Bit String: [\(data.map { "\($0)" }.joined(separator: ","))]")
                //return .bitString(data: data)
                return .ecBits(data: data)
            }

            throw DecodingError.invalidType(value: firstByte)
        }

        /// Parses an ASN1 sequence and returns its child nodes
        ///
        /// - Parameter data: ASN1 data
        /// - Returns: A list of ASN1 nodes
        /// - Throws: A DecodingError if anything goes wrong, or if an unknown node was encountered
        private static func decodeSequence(data: Data) throws -> [Node] {
            let scanner = Scanner(data: data)
            var nodes: [Node] = []
            while !scanner.isComplete {
                let node = try decodeNode(scanner: scanner)
                nodes.append(node)
            }
            return nodes
        }
    }
}
