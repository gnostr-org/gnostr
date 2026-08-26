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

// MARK: Encrypted PEM Cipher Algorithms

extension LibP2PCrypto.PEM {
    // MARK: Add support for new Cipher Algorithms here...
    internal enum CipherAlgorithm {
        case aes_128_cbc(iv: [UInt8])
        case aes_256_cbc(iv: [UInt8])
        //case des3(iv: [UInt8])

        init(objID: [UInt8], iv: [UInt8]) throws {
            switch objID {
            case [0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x02]:  // aes-128-cbc
                self = .aes_128_cbc(iv: iv)
            case [0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x2a]:  // aes-256-cbc
                self = .aes_256_cbc(iv: iv)
            //case [42, 134, 72, 134, 247, 13, 3, 7]:
            //  self = .des3(iv: iv)
            default:
                throw Error.unsupportedCipherAlgorithm(objID)
            }
        }

        func decrypt(bytes: [UInt8], withKey key: [UInt8]) throws -> [UInt8] {
            switch self {
            case .aes_128_cbc(let iv):
                //print("128 IV: \(iv)")
                return try AES(key: key, blockMode: CBC(iv: iv), padding: .pkcs7).decrypt(bytes)
            case .aes_256_cbc(let iv):
                //print("256 IV: \(iv)")
                return try AES(key: key, blockMode: CBC(iv: iv), padding: .pkcs7).decrypt(bytes)
            //default:
            //throw Error.invalidPEMFormat
            }
        }

        func encrypt(bytes: [UInt8], withKey key: [UInt8]) throws -> [UInt8] {
            switch self {
            case .aes_128_cbc(let iv):
                return try AES(key: key, blockMode: CBC(iv: iv), padding: .pkcs7).encrypt(bytes)
            case .aes_256_cbc(let iv):
                return try AES(key: key, blockMode: CBC(iv: iv), padding: .pkcs7).encrypt(bytes)
            }
        }

        /// The key length used for this Cipher strategy
        /// - Note: we need this information when deriving the key using our PBKDF strategy
        var desiredKeyLength: Int {
            switch self {
            case .aes_128_cbc: return 16
            case .aes_256_cbc: return 32
            }
        }

        var objectIdentifier: [UInt8] {
            switch self {
            case .aes_128_cbc:
                return [0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x02]
            case .aes_256_cbc:
                return [0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x01, 0x2a]
            }
        }

        var iv: [UInt8] {
            switch self {
            case .aes_128_cbc(let iv):
                return iv
            case .aes_256_cbc(let iv):
                return iv
            }
        }

        func encodeCipher() throws -> ASN1.Node {
            .sequence(nodes: [
                .objectIdentifier(data: Data(self.objectIdentifier)),
                .octetString(data: Data(self.iv)),
            ])
        }
    }

    /// Decodes the Cipher ASN1 Block in an Encrypted Private Key PEM file
    /// - Parameter node: The ASN1 sequence node containing the cipher parameters
    /// - Returns: The CipherAlogrithm if supported
    ///
    /// Expects an ASN1.Node with the following structure
    /// ```
    /// ASN1.Parser.Node.sequence(nodes: [
    ///     ASN1.Parser.Node.objectIdentifier(data: 9 bytes),      //des-ede3-cbc
    ///     ASN1.Parser.Node.octetString(data: 16 bytes)           //IV
    /// ])
    /// ```
    internal static func decodeCipher(_ node: ASN1.Node) throws -> CipherAlgorithm {
        guard case .sequence(let params) = node else { throw Error.invalidPEMFormat("EncryptedPrivateKey::CIPHER") }
        guard params.count == 2 else { throw Error.invalidPEMFormat("EncryptedPrivateKey::CIPHER") }
        guard case .objectIdentifier(let objID) = params.first else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::CIPHER")
        }
        guard case .octetString(let initialVector) = params.last else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::CIPHER")
        }

        return try CipherAlgorithm(objID: objID.byteArray, iv: initialVector.byteArray)
    }
}
