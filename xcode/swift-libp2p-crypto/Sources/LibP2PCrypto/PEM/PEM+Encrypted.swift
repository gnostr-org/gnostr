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

// MARK: Encrypted PEM

extension LibP2PCrypto.PEM {

    internal struct EncryptedPEM {
        let objectIdentifer: [UInt8]
        let ciphertext: [UInt8]
        let pbkdfAlgorithm: PBKDFAlgorithm
        let cipherAlgorithm: CipherAlgorithm
    }

    /// Attempts to decode an encrypted Private Key PEM, returning all of the information necessary to decrypt the encrypted PEM
    /// - Parameter encryptedPEM: The raw base64 decoded PEM data
    /// - Returns: An `EncryptedPEM` Struct containing the ciphertext, the pbkdf alogrithm for key derivation, the cipher algorithm for decrypting and the objectIdentifier describing the contents of this PEM data
    ///
    /// To decrypt an encrypted PEM Private Key...
    /// 1) Strip the headers of the PEM and base64 decode the data
    /// 2) Parse the data via ASN1 looking for both the pbkdf and cipher algorithms, their respective parameters (salt, iv and itterations) and the ciphertext (aka octet string))
    /// 3) Derive the encryption key using the appropriate pbkdf alogorithm, found in step 2
    /// 4) Use the encryption key to instantiate the appropriate cipher algorithm, also found in step 2
    /// 5) Decrypt the encrypted ciphertext (the contents of the octetString node)
    /// 6) The decrypted octet string can now be handled like any other Private Key PEM
    ///
    /// ```
    /// sequence(nodes: [
    ///   ASN1.Parser.Node.sequence(nodes: [
    ///       ASN1.Parser.Node.objectIdentifier(data: 9 bytes),              // PEM's ObjectIdentifier
    ///       ASN1.Parser.Node.sequence(nodes: [
    ///           ASN1.Parser.Node.sequence(nodes: [
    ///               ASN1.Parser.Node.objectIdentifier(data: 9 bytes),      // PBKDF Algorithm
    ///               ASN1.Parser.Node.sequence(nodes: [
    ///                   ASN1.Parser.Node.octetString(data: 8 bytes),       // SALT
    ///                   ASN1.Parser.Node.integer(data: 2 bytes)            // ITERATIONS
    ///               ])
    ///           ]),
    ///           ASN1.Parser.Node.sequence(nodes: [
    ///               ASN1.Parser.Node.objectIdentifier(data: 9 bytes),      // Cipher Algorithm (ex: des-ede3-cbc)
    ///               ASN1.Parser.Node.octetString(data: 16 bytes)           // Initial Vector (IV)
    ///           ])
    ///       ])
    ///   ]),
    ///   ASN1.Parser.Node.octetString(data: 640 bytes)
    /// ])
    /// ```
    internal static func decodeEncryptedPEM(_ encryptedPEM: Data) throws -> EncryptedPEM {
        let asn = try ASN1.Decoder.decode(data: encryptedPEM)

        guard case .sequence(let encryptedPEMWrapper) = asn else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::")
        }
        guard encryptedPEMWrapper.count == 2 else { throw Error.invalidPEMFormat("EncryptedPrivateKey::") }
        guard case .sequence(let encryptionInfoWrapper) = encryptedPEMWrapper.first else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::")
        }
        guard encryptionInfoWrapper.count == 2 else { throw Error.invalidPEMFormat("EncryptedPrivateKey::") }
        guard case .objectIdentifier(let objID) = encryptionInfoWrapper.first else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::")
        }
        guard case .sequence(let encryptionAlgorithmsWrapper) = encryptionInfoWrapper.last else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::")
        }
        guard encryptionAlgorithmsWrapper.count == 2 else { throw Error.invalidPEMFormat("EncryptedPrivateKey::") }
        let pbkdf = try decodePBKFD(encryptionAlgorithmsWrapper.first!)
        let cipher = try decodeCipher(encryptionAlgorithmsWrapper.last!)
        guard case .octetString(let octets) = encryptedPEMWrapper.last else {
            throw Error.invalidPEMFormat("EncryptedPrivateKey::")
        }

        return EncryptedPEM(
            objectIdentifer: objID.byteArray,
            ciphertext: octets.byteArray,
            pbkdfAlgorithm: pbkdf,
            cipherAlgorithm: cipher
        )
    }

    internal static func encryptPEM(
        _ pem: Data,
        withPassword password: String,
        usingPBKDF pbkdf: PBKDFAlgorithm = .pbkdf2(salt: try! LibP2PCrypto.randomBytes(length: 8), iterations: 2048),
        andCipher cipher: CipherAlgorithm = .aes_128_cbc(iv: try! LibP2PCrypto.randomBytes(length: 16))
    ) throws -> Data {

        // Generate Encryption Key from Password
        let key = try pbkdf.deriveKey(password: password, ofLength: cipher.desiredKeyLength)

        // Encrypt Plaintext
        let ciphertext = try cipher.encrypt(bytes: pem.byteArray, withKey: key)

        // Encode Encrypted PEM (including pbkdf and cipher algos used)
        let nodes: ASN1.Node = .sequence(nodes: [
            .sequence(nodes: [
                .objectIdentifier(data: Data(hex: "2a864886f70d01050d")),
                .sequence(nodes: [
                    try pbkdf.encodePBKDF(),
                    try cipher.encodeCipher(),
                ]),
            ]),
            .octetString(data: Data(ciphertext)),
        ])

        let encoded = ASN1.Encoder.encode(nodes)

        let base64 = "\n" + encoded.toBase64().split(intoChunksOfLength: 64).joined(separator: "\n") + "\n"

        return Data(
            LibP2PCrypto.PEM.PEMType.encryptedPrivateKey.headerBytes + base64.bytes
                + LibP2PCrypto.PEM.PEMType.encryptedPrivateKey.footerBytes
        )
    }

    internal static func encryptPEMString(
        _ pem: Data,
        withPassword password: String,
        usingPBKDF pbkdf: PBKDFAlgorithm = .pbkdf2(salt: try! LibP2PCrypto.randomBytes(length: 8), iterations: 2048),
        andCipher cipher: CipherAlgorithm = .aes_128_cbc(iv: try! LibP2PCrypto.randomBytes(length: 16))
    ) throws -> String {
        let data = try LibP2PCrypto.PEM.encryptPEM(pem, withPassword: password, usingPBKDF: pbkdf, andCipher: cipher)
        return String(data: data, encoding: .utf8)!
    }
}
