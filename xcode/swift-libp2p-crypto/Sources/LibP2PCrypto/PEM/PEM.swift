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

import CryptoSwift
import Foundation

extension LibP2PCrypto {
    public struct PEM {

        public enum Error: Swift.Error {
            /// An error occured while encoding the PEM file
            case encodingError
            /// An error occured while decoding the PEM file
            case decodingError
            /// Encountered an unsupported PEM type
            case unsupportedPEMType
            /// Encountered an invalid/unexpected PEM format
            case invalidPEMFormat(String? = nil)
            /// Encountered an invalid/unexpected PEM header string/delimiter
            case invalidPEMHeader
            /// Encountered an invalid/unexpected PEM footer string/delimiter
            case invalidPEMFooter
            /// Encountered a invalid/unexpected parameters while attempting to decode a PEM file
            case invalidParameters
            /// Encountered an unsupported Cipher algorithm while attempting to decrypt an encrypted PEM file
            case unsupportedCipherAlgorithm([UInt8])
            /// Encountered an unsupported Password Derivation algorithm while attempting to decrypt an encrypted PEM file
            case unsupportedPBKDFAlgorithm([UInt8])
            /// The instiating types objectIdentifier does not match that of the PEM file
            case objectIdentifierMismatch(got: [UInt8], expected: [UInt8])
        }

        // MARK: Add support for additional PEM types here

        /// General PEM Classification
        internal enum PEMType {
            // Direct DER Exports for RSA Keys (special case)
            case publicRSAKeyDER
            case privateRSAKeyDER

            // Generale PEM Headers
            case publicKey
            case privateKey
            case encryptedPrivateKey
            case ecPrivateKey

            // Others
            //case certificate

            init(headerBytes: ArraySlice<UInt8>) throws {
                guard headerBytes.count > 10 else { throw PEM.Error.unsupportedPEMType }
                let bytes = headerBytes.dropFirst(5).dropLast(5)
                switch bytes {
                //"BEGIN RSA PUBLIC KEY"
                case [
                    0x42, 0x45, 0x47, 0x49, 0x4e, 0x20, 0x52, 0x53, 0x41, 0x20, 0x50, 0x55, 0x42, 0x4c, 0x49, 0x43,
                    0x20,
                    0x4b, 0x45, 0x59,
                ]:
                    self = .publicRSAKeyDER

                //"BEGIN RSA PRIVATE KEY"
                case [
                    0x42, 0x45, 0x47, 0x49, 0x4e, 0x20, 0x52, 0x53, 0x41, 0x20, 0x50, 0x52, 0x49, 0x56, 0x41, 0x54,
                    0x45,
                    0x20, 0x4b, 0x45, 0x59,
                ]:
                    self = .privateRSAKeyDER

                //"BEGIN PUBLIC KEY"
                case [0x42, 0x45, 0x47, 0x49, 0x4e, 0x20, 0x50, 0x55, 0x42, 0x4c, 0x49, 0x43, 0x20, 0x4b, 0x45, 0x59]:
                    self = .publicKey

                //"BEGIN PRIVATE KEY"
                case [
                    0x42, 0x45, 0x47, 0x49, 0x4e, 0x20, 0x50, 0x52, 0x49, 0x56, 0x41, 0x54, 0x45, 0x20, 0x4b, 0x45,
                    0x59,
                ]:
                    self = .privateKey

                //"BEGIN ENCRYPTED PRIVATE KEY"
                case [
                    0x42, 0x45, 0x47, 0x49, 0x4e, 0x20, 0x45, 0x4e, 0x43, 0x52, 0x59, 0x50, 0x54, 0x45, 0x44, 0x20,
                    0x50,
                    0x52, 0x49, 0x56, 0x41, 0x54, 0x45, 0x20, 0x4b, 0x45, 0x59,
                ]:
                    self = .encryptedPrivateKey

                //"BEGIN EC PRIVATE KEY"
                case [
                    0x42, 0x45, 0x47, 0x49, 0x4e, 0x20, 0x45, 0x43, 0x20, 0x50, 0x52, 0x49, 0x56, 0x41, 0x54, 0x45,
                    0x20,
                    0x4b, 0x45, 0x59,
                ]:
                    self = .ecPrivateKey

                default:
                    print("Unsupported PEM Type: \(Data(bytes).toHexString())")
                    throw PEM.Error.unsupportedPEMType
                }
            }

            /// This PEM type's header string (expressed as the utf8 decoded byte representation)
            var headerBytes: [UInt8] {
                switch self {
                case .publicRSAKeyDER:
                    return "-----BEGIN RSA PUBLIC KEY-----".bytes
                case .privateRSAKeyDER:
                    return "-----BEGIN RSA PRIVATE KEY-----".bytes
                case .publicKey:
                    return "-----BEGIN PUBLIC KEY-----".bytes
                case .privateKey:
                    return "-----BEGIN PRIVATE KEY-----".bytes
                case .encryptedPrivateKey:
                    return "-----BEGIN ENCRYPTED PRIVATE KEY-----".bytes
                case .ecPrivateKey:
                    return "-----BEGIN EC PRIVATE KEY-----".bytes
                }
            }

            /// This PEM type's footer string (expressed as the utf8 decoded byte representation)
            var footerBytes: [UInt8] {
                switch self {
                case .publicRSAKeyDER:
                    return "-----END RSA PUBLIC KEY-----".bytes
                case .privateRSAKeyDER:
                    return "-----END RSA PRIVATE KEY-----".bytes
                case .publicKey:
                    return "-----END PUBLIC KEY-----".bytes
                case .privateKey:
                    return "-----END PRIVATE KEY-----".bytes
                case .encryptedPrivateKey:
                    return "-----END ENCRYPTED PRIVATE KEY-----".bytes
                case .ecPrivateKey:
                    return "-----END EC PRIVATE KEY-----".bytes
                }
            }
        }

        /// Converts UTF8 Encoding of PEM file into a PEMType and the base64 decoded key data
        /// - Parameter data: The `UTF8` encoding of the PEM file
        /// - Returns: A tuple containing the PEMType, and the actual base64 decoded PEM data (with the headers and footers removed).
        internal static func pemToData(
            _ data: [UInt8]
        ) throws -> (type: PEMType, bytes: [UInt8], objectIdentifiers: [[UInt8]]) {
            let fiveDashes = ArraySlice<UInt8>(repeating: 0x2D, count: 5)  // "-----".bytes.toHexString()
            let chunks = data.split(separator: 0x0a)  // 0x0a == "\n" `new line` char
            guard chunks.count > 2 else {
                throw PEM.Error.invalidPEMFormat(
                    "expected at least 3 chunks, a header, body and footer, but got \(chunks.count)"
                )
            }

            // Enforce a valid PEM header
            guard let header = chunks.first,
                header.count > 10,
                header.prefix(5) == fiveDashes,
                header.suffix(5) == fiveDashes
            else {
                throw PEM.Error.invalidPEMHeader
            }

            // Enforce a valid PEM footer
            guard let footer = chunks.last,
                footer.count > 10,
                footer.prefix(5) == fiveDashes,
                footer.suffix(5) == fiveDashes
            else {
                throw PEM.Error.invalidPEMFooter
            }

            // Attempt to classify the PEMType based on the header
            //
            // - Note: This just gives us a general idea of what direction to head in. Headers that don't match the underlying data will end up throwing an Error later
            let pemType: PEMType = try PEMType(headerBytes: header)

            guard let base64 = String(data: Data(chunks[1..<chunks.count - 1].joined()), encoding: .utf8) else {
                throw Error.invalidPEMFormat("Unable to join chunked body data")
            }
            guard let pemData = Data(base64Encoded: base64) else {
                throw Error.invalidPEMFormat("Body of PEM isn't valid base64 encoded")
            }

            let asn1 = try ASN1.Decoder.decode(data: pemData)

            // return the PEMType and PEM Data (without header & footer)
            return (
                type: pemType, bytes: pemData.byteArray, objectIdentifiers: objIdsInSequence(asn1).map { $0.byteArray }
            )
        }

        /// Traverses a Node tree and returns all instances of objectIds
        internal static func objIdsInSequence(_ node: ASN1.Node) -> [Data] {
            if case .objectIdentifier(let id) = node {
                return [id]
            } else if case .sequence(let nodes) = node {
                return objIdsInSequence(nodes)
            }
            return []
        }

        /// Traverses a Node tree and returns all instances of objectIds
        internal static func objIdsInSequence(_ nodes: [ASN1.Node]) -> [Data] {
            var objs: [Data] = []

            for node in nodes {
                if case .objectIdentifier(let id) = node {
                    objs.append(id)
                } else if case .sequence(let nodes) = node {
                    objs.append(contentsOf: objIdsInSequence(nodes))
                }
            }

            return objs
        }

        /// Decodes an ASN1 formatted Public Key into it's raw DER representation
        /// - Parameters:
        ///   - pem: The ASN1 encoded Public Key representation
        ///   - expectedObjectIdentifier: The expected objectIdentifier for the particular key type
        /// - Returns: The raw bitString data (Public Key DER)
        ///
        /// ```
        /// 0:d=0  hl=4 l= 546 cons: SEQUENCE
        /// 4:d=1  hl=2 l=  13 cons:  SEQUENCE
        /// 6:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
        /// 17:d=2  hl=2 l=   0 prim:   NULL
        /// 19:d=1  hl=4 l= 527 prim:  BIT STRING
        /// ```
        internal static func decodePublicKeyPEM(
            _ pem: Data,
            expectedPrimaryObjectIdentifier: [UInt8],
            expectedSecondaryObjectIdentifier: [UInt8]?
        ) throws -> [UInt8] {
            let asn = try ASN1.Decoder.decode(data: pem)

            //print("PublicKey")
            //print(asn)

            // Enforce the above ASN1 Structure
            guard case .sequence(let sequence) = asn else {
                throw Error.invalidPEMFormat("PublicKey::No top level sequence for PublicKey PEM")
            }
            guard sequence.count == 2 else {
                throw Error.invalidPEMFormat(
                    "PublicKey::Top level sequnce should contain two nodes but we got \(sequence.count) isntead"
                )
            }
            guard case .sequence(let params) = sequence.first else {
                throw Error.invalidPEMFormat(
                    "PublicKey::Expected the first node of the top level to be a sequence node, but we got \(sequence.first?.description ?? "NIL") instead"
                )
            }
            guard params.count >= 1 else {
                throw Error.invalidPEMFormat("PublicKey::Expected at least one param within the secondary sequence")
            }
            guard case .objectIdentifier(let objectID) = params.first else {
                throw Error.invalidPEMFormat(
                    "PublicKey::Expected first param of secondary sequence to be an objectIndentifier"
                )
            }

            // Ensure the ObjectID specified in the PEM matches that of the Key.Type we're attempting to instantiate
            guard objectID.byteArray == expectedPrimaryObjectIdentifier else {
                throw Error.objectIdentifierMismatch(got: objectID.byteArray, expected: expectedPrimaryObjectIdentifier)
            }

            // If the key supports a secondary objectIdentifier (ensure one is present and that they match)
            if let expectedSecondaryObjectIdentifier = expectedSecondaryObjectIdentifier {
                guard params.count >= 2 else { throw Error.invalidPEMFormat("PrivateKey::") }
                guard case .objectIdentifier(let objectIDSecondary) = params[1] else {
                    throw Error.invalidPEMFormat("PrivateKey::")
                }
                guard objectIDSecondary.byteArray == expectedSecondaryObjectIdentifier else {
                    throw Error.objectIdentifierMismatch(
                        got: objectIDSecondary.byteArray,
                        expected: expectedSecondaryObjectIdentifier
                    )
                }
            }

            guard case .bitString(let bits) = sequence.last else {
                throw Error.invalidPEMFormat("Expected the last element of the top level sequence to be a bitString")
            }

            return bits.byteArray
        }

        /// Decodes an ASN1 formatted Private Key into it's raw DER representation
        /// - Parameters:
        ///   - pem: The ASN1 encoded Private Key representation
        ///   - expectedObjectIdentifier: The expected objectIdentifier for the particular key type
        /// - Returns: The raw octetString data (Private Key DER)
        internal static func decodePrivateKeyPEM(
            _ pem: Data,
            expectedPrimaryObjectIdentifier: [UInt8],
            expectedSecondaryObjectIdentifier: [UInt8]?
        ) throws -> [UInt8] {
            let asn = try ASN1.Decoder.decode(data: pem)

            //print("PrivateKey")
            //print(asn)

            // Enforce the above ASN1 Structure
            guard case .sequence(let sequence) = asn else {
                throw Error.invalidPEMFormat("PrivateKey::Top level node is not a sequence")
            }
            // Enforce the integer/version param as the first param in our top level sequence
            guard case .integer(let integer) = sequence.first else {
                throw Error.invalidPEMFormat("PrivateKey::First item in top level sequence wasn't an integer")
            }
            //print("PEM Version: \(integer.bytes)")
            switch integer {
            case Data(hex: "0x00"):
                //Proceed with standard pkcs1 private key format
                return try decodePrivateKey(
                    sequence,
                    expectedPrimaryObjectIdentifier: expectedPrimaryObjectIdentifier,
                    expectedSecondaryObjectIdentifier: expectedSecondaryObjectIdentifier
                )
            case Data(hex: "0x01"):
                //Proceed with EC private key format
                return try decodePrivateECKey(
                    sequence,
                    expectedPrimaryObjectIdentifier: expectedPrimaryObjectIdentifier
                )
            default:
                throw Error.invalidPEMFormat("Unknown version identifier")
            }
        }

        /// Decodes a standard (RSA) Private Key PEM file
        /// - Parameters:
        ///   - sequence: The contents of the top level ASN1 Sequence node
        ///   - expectedPrimaryObjectIdentifier: The expected primary object identifier key to compare the PEM contents against
        ///   - expectedSecondaryObjectIdentifier: The expected secondary object identifier key to compare the PEM contents against
        /// - Returns: The private key bytes
        ///
        /// [Private key format]()
        /// ```
        /// 0:d=0  hl=4 l= 630 cons: SEQUENCE
        /// 4:d=1  hl=2 l=   1 prim:  INTEGER           :00
        /// 7:d=1  hl=2 l=  13 cons:  SEQUENCE
        /// 9:d=2  hl=2 l=   9 prim:   OBJECT            :rsaEncryption
        /// 20:d=2  hl=2 l=   0 prim:   NULL
        /// 22:d=1  hl=4 l= 608 prim:  OCTET STRING      [HEX DUMP]:3082...AA50
        /// ```
        private static func decodePrivateKey(
            _ sequence: [ASN1.Node],
            expectedPrimaryObjectIdentifier: [UInt8],
            expectedSecondaryObjectIdentifier: [UInt8]?
        ) throws -> [UInt8] {
            guard sequence.count == 3 else {
                throw Error.invalidPEMFormat("PrivateKey::Top level sequence doesn't contain 3 items")
            }
            guard case .sequence(let params) = sequence[1] else {
                throw Error.invalidPEMFormat("PrivateKey::Second item wasn't a sequence")
            }
            guard params.count >= 1 else {
                throw Error.invalidPEMFormat("PrivateKey::Second sequence contained fewer than expected parameters")
            }
            guard case .objectIdentifier(let objectID) = params.first else {
                throw Error.invalidPEMFormat("PrivateKey::")
            }

            // Ensure the ObjectID specified in the PEM matches that of the Key.Type we're attempting to instantiate
            guard objectID.byteArray == expectedPrimaryObjectIdentifier else {
                throw Error.objectIdentifierMismatch(got: objectID.byteArray, expected: expectedPrimaryObjectIdentifier)
            }

            // If the key supports a secondary objectIdentifier (ensure one is present and that they match)
            if let expectedSecondaryObjectIdentifier = expectedSecondaryObjectIdentifier {
                guard params.count >= 2 else { throw Error.invalidPEMFormat("PrivateKey::") }
                guard case .objectIdentifier(let objectIDSecondary) = params[1] else {
                    throw Error.invalidPEMFormat("PrivateKey::")
                }
                guard objectIDSecondary.byteArray == expectedSecondaryObjectIdentifier else {
                    throw Error.objectIdentifierMismatch(
                        got: objectIDSecondary.byteArray,
                        expected: expectedSecondaryObjectIdentifier
                    )
                }
            }

            guard case .octetString(let octet) = sequence[2] else { throw Error.invalidPEMFormat("PrivateKey::") }

            return octet.byteArray
        }

        /// Decodes an Eliptic Curve Private Key PEM that conforms to the IETF RFC5915 structure
        /// - Parameters:
        ///   - node: The contents of the top level ASN1 Sequence node
        ///   - expectedPrimaryObjectIdentifier: The expected primary object identifier key to compare the PEM contents against
        /// - Returns: The EC private key bytes
        ///
        /// [EC private key format](https://datatracker.ietf.org/doc/html/rfc5915#section-3)
        /// ```
        /// ECPrivateKey ::= SEQUENCE {
        ///     version        INTEGER { ecPrivkeyVer1(1) } (ecPrivkeyVer1),
        ///     privateKey     OCTET STRING,
        ///     parameters [0] ECParameters {{ NamedCurve }} OPTIONAL,
        ///     publicKey  [1] BIT STRING OPTIONAL
        /// }
        /// ```
        private static func decodePrivateECKey(
            _ sequence: [ASN1.Node],
            expectedPrimaryObjectIdentifier: [UInt8]
        ) throws -> [UInt8] {
            guard sequence.count >= 2 else {
                throw Error.invalidPEMFormat("PrivateKey::EC::Top level sequence doesn't contain at least 2 items")
            }
            guard case .octetString(let octet) = sequence[1] else {
                throw Error.invalidPEMFormat("PrivateKey::EC::Second item wasn't an octetString")
            }

            // Remaining parameters are optional...
            if sequence.count > 2 {
                guard case .objectIdentifier(let objectID) = sequence[2] else {
                    throw Error.invalidPEMFormat("PrivateKey::EC::Missing objectIdentifier in top level sequence")
                }
                // Ensure the ObjectID specified in the PEM matches that of the Key.Type we're attempting to instantiate
                guard objectID.byteArray == expectedPrimaryObjectIdentifier else {
                    throw Error.objectIdentifierMismatch(
                        got: objectID.byteArray,
                        expected: expectedPrimaryObjectIdentifier
                    )
                }
            }

            //if sequence.count > 3 {
            //    // Optional Public Key
            //    guard case .bitString(let _) = sequence[3] else { throw Error.invalidPEMFormat("PrivateKey::EC::") }
            //}

            return octet.byteArray
        }
    }
}
