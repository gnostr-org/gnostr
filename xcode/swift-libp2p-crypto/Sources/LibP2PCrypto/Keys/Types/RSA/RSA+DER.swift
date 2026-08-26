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

extension RSAPublicKey: DERCodable {
    /// RSA Object Identifier Bytes
    public static var primaryObjectIdentifier: [UInt8] { [42, 134, 72, 134, 247, 13, 1, 1, 1] }

    public static var secondaryObjectIdentifier: [UInt8]? { nil }

    public func publicKeyDER() throws -> [UInt8] {
        self.rawRepresentation.byteArray
    }

    public func privateKeyDER() throws -> [UInt8] {
        throw NSError(domain: "Public Key doesn't have private DER representation", code: 0)
    }

    init(publicDER: [UInt8]) throws {
        try self.init(rawRepresentation: Data(publicDER))
    }

    init(privateDER: [UInt8]) throws {
        throw NSError(domain: "Can't instantiate private key from public DER representation", code: 0)
    }

    public func exportPublicKeyPEM(withHeaderAndFooter: Bool) throws -> [UInt8] {
        let publicDER = try self.publicKeyDER()

        let base64String = publicDER.toBase64()
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

extension RSAPrivateKey: DERCodable {
    /// RSA Object Identifier Bytes
    public static var primaryObjectIdentifier: [UInt8] { [42, 134, 72, 134, 247, 13, 1, 1, 1] }

    static var secondaryObjectIdentifier: [UInt8]? { nil }

    func publicKeyDER() throws -> [UInt8] {
        try self.derivePublicKey().rawRepresentation.byteArray
    }

    func privateKeyDER() throws -> [UInt8] {
        self.rawRepresentation.byteArray
    }

    init(publicDER: [UInt8]) throws {
        throw NSError(domain: "Can't instantiate private key from public DER representation", code: 0)
    }

    init(privateDER: [UInt8]) throws {
        try self.init(rawRepresentation: Data(privateDER))
    }

    public func exportPrivateKeyPEMRaw() throws -> [UInt8] {
        let privateDER = try self.privateKeyDER()
        let asnNodes: ASN1.Node = .sequence(nodes: [
            .integer(data: Data(hex: "0x00")),
            .sequence(nodes: [
                .objectIdentifier(data: Data(Self.primaryObjectIdentifier)),
                .null,
            ]),
            .octetString(data: Data(privateDER)),
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
