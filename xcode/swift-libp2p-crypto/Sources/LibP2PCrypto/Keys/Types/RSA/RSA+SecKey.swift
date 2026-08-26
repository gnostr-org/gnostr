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

#if canImport(Security)
import Foundation
import Multibase
@preconcurrency import Security

struct RSAPublicKey: CommonPublicKey {
    static var keyType: LibP2PCrypto.Keys.GenericKeyType { .rsa }

    /// The underlying SecKey that backs this struct
    private let key: SecKey

    fileprivate init(_ secKey: SecKey) {
        self.key = secKey
    }

    init(rawRepresentation raw: Data) throws {
        let attributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeRSA,
            kSecAttrKeyClass as String: kSecAttrKeyClassPublic,
            kSecAttrKeySizeInBits as String: 1024,
            kSecAttrIsPermanent as String: false,
        ]

        var error: Unmanaged<CFError>? = nil
        guard let secKey = SecKeyCreateWithData(raw as CFData, attributes as CFDictionary, &error) else {
            throw NSError(
                domain: "Error constructing SecKey from raw key data: \(error.debugDescription)",
                code: 0,
                userInfo: nil
            )
        }

        self.key = secKey
    }

    init(marshaledData data: Data) throws {
        let asn = try ASN1.Decoder.decode(data: data)
        guard case .sequence(let nodes) = asn else {
            throw NSError(domain: "RSAPublicKey Invalid marshaled data", code: 0)
        }
        guard case .sequence(let subjectInfo) = nodes[0] else {
            throw NSError(domain: "RSAPublicKey Invalid marshaled data", code: 0)
        }
        guard case .objectIdentifier(let objID) = subjectInfo.first else {
            throw NSError(domain: "RSAPublicKey Invalid marshaled data", code: 0)
        }
        guard objID.byteArray == RSAPublicKey.primaryObjectIdentifier else {
            throw NSError(domain: "RSAPublicKey Invalid marshaled data", code: 0)
        }
        guard case .bitString(let bits) = nodes[1] else {
            throw NSError(domain: "RSAPublicKey Invalid marshaled data", code: 0)
        }
        try self.init(rawRepresentation: bits)
    }

    var rawRepresentation: Data {
        let asnNodes: ASN1.Node = try! .sequence(nodes: [
            .sequence(nodes: [
                .objectIdentifier(data: Data(RSAPublicKey.primaryObjectIdentifier)),
                .null,
            ]),
            .bitString(data: self.key.rawRepresentation()),
        ])

        return Data(ASN1.Encoder.encode(asnNodes))
    }

    func encrypt(data: Data) throws -> Data {
        var error: Unmanaged<CFError>?
        guard let encryptedData = SecKeyCreateEncryptedData(self.key, .rsaEncryptionPKCS1, data as CFData, &error)
        else {
            throw NSError(domain: "Error Encrypting Data: \(error.debugDescription)", code: 0, userInfo: nil)
        }
        return encryptedData as Data
    }

    func verify(signature: Data, for expectedData: Data) throws -> Bool {
        var error: Unmanaged<CFError>?

        // Perform the signature verification
        let result = SecKeyVerifySignature(
            self.key,
            .rsaSignatureMessagePKCS1v15SHA256,
            expectedData as CFData,
            signature as CFData,
            &error
        )

        // Throw the error if we encountered one...
        if let error = error { throw error.takeRetainedValue() as Error }

        // return the result of the verification
        return result
    }

    public func marshal() throws -> Data {
        var publicKey = PublicKey()
        publicKey.type = .rsa
        //RSAPublicKeyExporter().toSubjectPublicKeyInfo(self.rawRepresentation)
        publicKey.data = self.rawRepresentation
        return try publicKey.serializedData()
    }

}

struct RSAPrivateKey: CommonPrivateKey {
    static var keyType: LibP2PCrypto.Keys.GenericKeyType { .rsa }

    /// The underlying SecKey that backs this struct
    private let key: SecKey

    fileprivate init(_ secKey: SecKey) {
        self.key = secKey
    }

    /// Initializes a new RSA key (backed by SecKey) of the specified bit size
    init(keySize: Int) throws {
        guard keySize >= 1024 else { throw NSError(domain: "Invalid RSA Bit Size", code: 0) }

        let parameters: [CFString: Any] = [
            kSecAttrKeyType: kSecAttrKeyTypeRSA,
            kSecAttrKeySizeInBits: keySize,
        ]

        var error: Unmanaged<CFError>? = nil

        guard let privKey = SecKeyCreateRandomKey(parameters as CFDictionary, &error) else {
            print(error.debugDescription)
            throw NSError(domain: "Key Generation Error: \(error.debugDescription)", code: 0, userInfo: nil)
        }

        self.key = privKey
    }

    init(rawRepresentation raw: Data) throws {
        let attributes: [String: Any] = [
            kSecAttrKeyType as String: kSecAttrKeyTypeRSA,
            kSecAttrKeyClass as String: kSecAttrKeyClassPrivate,
            kSecAttrKeySizeInBits as String: 1024,
            kSecAttrIsPermanent as String: false,
        ]

        var error: Unmanaged<CFError>? = nil
        guard let secKey = SecKeyCreateWithData(raw as CFData, attributes as CFDictionary, &error) else {
            throw NSError(
                domain: "Error constructing SecKey from raw key data: \(error.debugDescription)",
                code: 0,
                userInfo: nil
            )
        }

        self.key = secKey
    }

    init(marshaledData data: Data) throws {
        try self.init(rawRepresentation: data)
    }

    var rawRepresentation: Data {
        var error: Unmanaged<CFError>?
        if let cfdata = SecKeyCopyExternalRepresentation(self.key, &error) {
            return cfdata as Data
        } else {
            //throw NSError(domain: "RawKeyError: \(error.debugDescription)", code: 0, userInfo: nil)
            return Data()
        }
    }

    func derivePublicKey() throws -> CommonPublicKey {
        guard let pubKey = SecKeyCopyPublicKey(self.key) else {
            throw NSError(domain: "Public Key Extraction Error", code: 0, userInfo: nil)
        }
        return RSAPublicKey(pubKey)
    }

    func decrypt(data: Data) throws -> Data {
        var error: Unmanaged<CFError>?
        guard let decryptedData = SecKeyCreateDecryptedData(self.key, .rsaEncryptionPKCS1, data as CFData, &error)
        else {
            throw NSError(domain: "Error Decrypting Data: \(error.debugDescription)", code: 0, userInfo: nil)
        }
        return decryptedData as Data
    }

    func sign(message: Data) throws -> Data {
        var error: Unmanaged<CFError>?

        // Sign the data
        guard
            let signature = SecKeyCreateSignature(
                self.key,
                .rsaSignatureMessagePKCS1v15SHA256,
                message as CFData,
                &error
            ) as Data?
        else { throw NSError(domain: "Encountered NIL Signature Value", code: 0) }

        // Throw the error if we encountered one
        if let error = error { throw error.takeRetainedValue() as Error }

        // Return the signature
        return signature
    }

    public func marshal() throws -> Data {
        var privateKey = PrivateKey()
        privateKey.type = .rsa
        privateKey.data = self.rawRepresentation
        return try privateKey.serializedData()
    }

}

extension RSAPublicKey: Equatable {
    static func == (lhs: RSAPublicKey, rhs: RSAPublicKey) -> Bool {
        lhs.rawRepresentation == rhs.rawRepresentation
    }
}

extension RSAPrivateKey: Equatable {
    static func == (lhs: RSAPrivateKey, rhs: RSAPrivateKey) -> Bool {
        lhs.rawRepresentation == rhs.rawRepresentation
    }
}

extension SecKey {
    func asString(base: BaseEncoding) throws -> String {
        try self.rawRepresentation().asString(base: base)
    }

    func extractPubKey() throws -> SecKey {
        guard let pubKey = SecKeyCopyPublicKey(self) else {
            throw NSError(domain: "Public Key Extraction Error", code: 0, userInfo: nil)
        }
        return pubKey
    }

    /// Returns the DER Encoded representation of the SecKey ( this does not include ASN.1 Headers for SubjectKeyInfo format)
    /// - Note: The method returns data in the PKCS #1 format for an RSA key. For an elliptic curve public key, the format follows the ASN.1 X9.63 standard using a byte string of 04 || X || Y. For an elliptic curve private key, the output is formatted as the public key concatenated with the big endian encoding of the secret scalar, or 04 || X || Y || K. All of these representations use constant size integers, including leading zeros as needed.
    func rawRepresentation() throws -> Data {
        var error: Unmanaged<CFError>?
        if let cfdata = SecKeyCopyExternalRepresentation(self, &error) {
            return cfdata as Data
        } else {
            throw NSError(domain: "RawKeyError: \(error.debugDescription)", code: 0, userInfo: nil)
        }
    }

    var attributes: CFDictionary? {
        SecKeyCopyAttributes(self)
    }
}

#endif
