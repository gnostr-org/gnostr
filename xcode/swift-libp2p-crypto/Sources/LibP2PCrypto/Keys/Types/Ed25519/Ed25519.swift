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

import Crypto
import Foundation

extension Curve25519.Signing.PublicKey: CommonPublicKey, @unchecked Sendable {
    public static var keyType: LibP2PCrypto.Keys.GenericKeyType { .ed25519 }

    init(marshaledData data: Data) throws {
        try self.init(rawRepresentation: data)
    }

    public func encrypt(data: Data) throws -> Data {
        throw NSError(domain: "Ed25519 Keys don't support encryption", code: 0)
    }

    public func verify(signature: Data, for expectedData: Data) throws -> Bool {
        self.isValidSignature(signature, for: expectedData)
    }

    public func marshal() throws -> Data {
        var publicKey = PublicKey()
        publicKey.type = .ed25519
        publicKey.data = self.rawRepresentation
        return try publicKey.serializedData()
    }
}

extension Curve25519.Signing.PrivateKey: CommonPrivateKey, @unchecked Sendable {
    public static var keyType: LibP2PCrypto.Keys.GenericKeyType { .ed25519 }

    init(marshaledData data: Data) throws {
        try self.init(rawRepresentation: data)
    }

    public func derivePublicKey() throws -> CommonPublicKey {
        self.publicKey
    }

    public func decrypt(data: Data) throws -> Data {
        throw NSError(domain: "ED25519 keys don't support decryption", code: 0)
    }

    public func sign(message data: Data) throws -> Data {
        try self.signature(for: data)
    }

    public func marshal() throws -> Data {
        var privateKey = PrivateKey()
        privateKey.type = .ed25519
        privateKey.data = self.rawRepresentation
        return try privateKey.serializedData()
    }
}

extension Curve25519.Signing.PublicKey: @retroactive Equatable {
    public static func == (lhs: Curve25519.Signing.PublicKey, rhs: Curve25519.Signing.PublicKey) -> Bool {
        lhs.rawRepresentation == rhs.rawRepresentation
    }
}

extension Curve25519.Signing.PrivateKey: @retroactive Equatable {
    public static func == (lhs: Curve25519.Signing.PrivateKey, rhs: Curve25519.Signing.PrivateKey) -> Bool {
        lhs.rawRepresentation == rhs.rawRepresentation
    }
}

extension Curve25519.Signing.PublicKey: DERCodable {
    public static var primaryObjectIdentifier: [UInt8] { [0x2B, 0x65, 0x70] }
    public static var secondaryObjectIdentifier: [UInt8]? { nil }

    public init(publicDER: [UInt8]) throws {
        try self.init(rawRepresentation: publicDER)
    }

    public init(privateDER: [UInt8]) throws {
        throw NSError(domain: "Can't instantiate private key from public DER representation", code: 0)
    }

    public func publicKeyDER() throws -> [UInt8] {
        self.rawRepresentation.byteArray
    }

    public func privateKeyDER() throws -> [UInt8] {
        throw NSError(domain: "Public Key doesn't have private DER representation", code: 0)
    }

    public func exportPublicKeyPEM(withHeaderAndFooter: Bool) throws -> [UInt8] {
        let publicDER = try self.publicKeyDER()

        let asnNodes: ASN1.Node = .sequence(nodes: [
            .sequence(nodes: [
                .objectIdentifier(data: Data(Self.primaryObjectIdentifier))
            ]),
            .bitString(data: Data(publicDER)),
        ])

        let base64String = ASN1.Encoder.encode(asnNodes).toBase64()
        let bodyString = base64String.chunks(ofCount: 64).joined(separator: "\n")
        let bodyUTF8Bytes = bodyString.bytes

        if withHeaderAndFooter {
            let header = LibP2PCrypto.PEM.PEMType.publicKey.headerBytes + [0x0a]
            let footer = [0x0a] + LibP2PCrypto.PEM.PEMType.publicKey.footerBytes

            return header + bodyUTF8Bytes + footer
        } else {
            return bodyUTF8Bytes
        }
    }
}

extension Curve25519.Signing.PrivateKey: DERCodable {
    public static var primaryObjectIdentifier: [UInt8] { [0x2B, 0x65, 0x70] }
    public static var secondaryObjectIdentifier: [UInt8]? { nil }

    public init(publicDER: [UInt8]) throws {
        throw NSError(domain: "Can't instantiate private key from public DER representation", code: 0)
    }

    public init(privateDER: [UInt8]) throws {
        guard case .octetString(let rawData) = try ASN1.Decoder.decode(data: Data(privateDER)) else {
            throw LibP2PCrypto.PEM.Error.invalidParameters
        }
        try self.init(rawRepresentation: rawData)
    }

    public func publicKeyDER() throws -> [UInt8] {
        try self.publicKey.publicKeyDER()
    }

    public func privateKeyDER() throws -> [UInt8] {
        ASN1.Encoder.encode(
            ASN1.Node.octetString(data: Data(self.rawRepresentation))
        )
    }

    public func exportPrivateKeyPEMRaw() throws -> [UInt8] {
        let privKey = try privateKeyDER()

        let asnNodes: ASN1.Node = .sequence(nodes: [
            .integer(data: Data(hex: "0x00")),
            .sequence(nodes: [
                .objectIdentifier(data: Data(Self.primaryObjectIdentifier))
            ]),
            .octetString(data: Data(privKey)),
        ])

        return ASN1.Encoder.encode(asnNodes)
    }

    public func exportPrivateKeyPEM(withHeaderAndFooter: Bool) throws -> [UInt8] {
        let base64String = try self.exportPrivateKeyPEMRaw().toBase64()
        let bodyString = base64String.chunks(ofCount: 64).joined(separator: "\n")
        let bodyUTF8Bytes = bodyString.bytes

        if withHeaderAndFooter {
            let header = LibP2PCrypto.PEM.PEMType.privateKey.headerBytes + [0x0a]
            let footer = [0x0a] + LibP2PCrypto.PEM.PEMType.privateKey.footerBytes

            return header + bodyUTF8Bytes + footer
        } else {
            return bodyUTF8Bytes
        }
    }
}
