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
import Multibase

public protocol CommonPrivateKey: DERCodable, Sendable {
    static var keyType: LibP2PCrypto.Keys.GenericKeyType { get }

    /// Init from raw representation
    init(rawRepresentation: Data) throws

    /// Raw Representation
    var rawRepresentation: Data { get }

    /// Derivation
    func derivePublicKey() throws -> CommonPublicKey

    /// Decryption
    func decrypt(data: Data) throws -> Data

    /// Signatures
    func sign(message: Data) throws -> Data

    /// Imports
    //init(fromMarshaledData:Data) throws
    //init(pemRSA:String) throws
    //init(pemEC:String) throws

    /// Exports
    //func exportPEM() throws -> Data
    //func exportJWK() throws -> Data
    //func exportCID() throws -> Data
    func marshal() throws -> Data

    /// Misc
    //func stretch() throws -> Data
    //func id() throws -> String
}

extension CommonPrivateKey {
    var keyType: LibP2PCrypto.Keys.GenericKeyType { Self.keyType }
}

extension CommonPrivateKey {
    /// The keys `rawID` is the SHA-256 multihash of its public key
    /// The public key is a protobuf encoding containing a type and the DER encoding
    /// of the PKCS SubjectPublicKeyInfo.
    public func rawID() throws -> [UInt8] {
        try self.derivePublicKey().rawID()
    }

    /// The key id is the base58 encoding of the SHA-256 multihash of its public key.
    /// The public key is a protobuf encoding containing a type and the DER encoding
    /// of the PKCS SubjectPublicKeyInfo.
    public func id(withMultibasePrefix: Bool = true) throws -> String {
        try self.derivePublicKey().id(withMultibasePrefix: withMultibasePrefix)
    }

    public func asString(base: BaseEncoding, withMultibasePrefix: Bool = false) -> String {
        self.data.asString(base: base, withMultibasePrefix: withMultibasePrefix)
    }

    public var data: Data {
        self.rawRepresentation
    }
}
