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

import CID
import Foundation
import LibP2PCrypto
import Multihash

/// - Reference: https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md#how-keys-are-encoded-and-messages-signed
public struct PeerID: Sendable {

    /// General PeerID Errors
    public enum Errors: Error {
        /// Unable to extract digest from Multihash
        case invalidMultihashDigest
        /// Invalid CID Codec for PeerID Instantiation
        case invalidCIDCodec(expected: String, received: String)
    }

    /// The keys ID is the SHA-256 multihash of its public key
    /// - Note: The public key is a protobuf encoding containing a type and the DER encoding of the PKCS SubjectPublicKeyInfo.
    public let multihash: Multihash
    public let keyPair: LibP2PCrypto.Keys.KeyPair?

    /// Returns the ID of this PeerID as bytes
    public var id: [UInt8] { multihash.value }

    /// Returns the ID of this PeerID as bytes
    @available(*, deprecated, renamed: "id")
    public var bytes: [UInt8] { id }

    /// Returns the PeerID's id as a base58 string (multihash/CIDv0).
    public var b58String: String {
        self.id.asString(base: .base58btc)
    }

    /// Returns the PeerID's id as a hex string.
    public var hexString: String {
        self.id.asString(base: .base16)
    }

    /// A base32 encoded, version 1 CID, representing this PeerID
    public var cidString: String {
        (try? CID(version: .v1, codec: .libp2p_key, hash: self.id).toBaseEncodedString(.base32)) ?? ""
    }

    public enum PeerType {
        case idOnly
        case isPublic
        case isPrivate
    }

    /// A simple way of checking a PeerID's type (id only, public key & id, or private key, public key and id)
    public var type: PeerType {
        if self.keyPair?.privateKey != nil {
            return .isPrivate
        } else if self.keyPair?.publicKey != nil {
            return .isPublic
        } else {
            return .idOnly
        }
    }

    /// Generates a Public/Private `KeyPair` of the specified type and initializes a `PeerID` with it (defaults to RSA 2048 bits)
    public init(_ keyType: LibP2PCrypto.Keys.KeyPairType = .RSA(bits: .B2048)) throws {
        let keyPair = try LibP2PCrypto.Keys.generateKeyPair(keyType)

        self.multihash = try keyPair.multihash()
        self.keyPair = keyPair
    }

    /// Initializes a `PeerID` using an existing Public/Private `KeyPair`
    public init(keyPair: LibP2PCrypto.Keys.KeyPair) throws {
        self.multihash = try keyPair.multihash()
        self.keyPair = keyPair
    }

    //init(publicKey:RawPublicKey) throws {
    //    let kp = LibP2PCrypto.Keys.KeyPair(
    //}

    //init(privateKey:RawPrivateKey) throws {
    //    let kp = LibP2PCrypto.Keys.KeyPair(
    //}

    /// Inits a `PeerID` based solely on an ID value with no underlying `KeyPair`
    public init(fromHexID hex: String) throws {
        self.multihash = try Multihash(hexString: hex)
        self.keyPair = nil
    }

    /// Inits a `PeerID` based solely on an ID value with no underlying `KeyPair`
    /// - Supports embedded ED25519 Public Keys
    public init(fromBytesID bytes: [UInt8]) throws {
        if let mh = try? Multihash(bytes), mh.algorithm == .identity {
            guard let digest = mh.digest else { throw Errors.invalidMultihashDigest }
            try self.init(marshaledPublicKey: Data(digest))
        } else {
            try self.init(fromBytesIDInternal: bytes)
        }
    }

    /// Inits a `PeerID` based solely on an ID value with no underlying `KeyPair`
    internal init(fromBytesIDInternal bytes: [UInt8]) throws {
        self.multihash = try Multihash(bytes)
        self.keyPair = nil
    }

    /// Inits a `PeerID` from a v0 dag-pb or v1 libp2p-key CID complient string
    /// - Supports embedded ED25519 Public Keys
    public init(cid: String) throws {
        try self.init(cid: CID(cid))
    }

    /// Inits a `PeerID` from a v0 dag-pb or v1 libp2p-key CID
    /// - Supports embedded ED25519 Public Keys
    public init(cid: CID) throws {
        guard cid.codec == .libp2p_key || cid.codec == .dag_pb else {
            throw Errors.invalidCIDCodec(expected: "'v0 dag-pb' or 'v1 libp2p-key'", received: "\(cid.codec)")
        }
        if cid.multihash.algorithm == .identity {
            // Check to see if we can instantiate an ED25519 pubkey from the id
            guard let digest = cid.multihash.digest else { throw Errors.invalidMultihashDigest }
            try self.init(marshaledPublicKey: Data(digest))
        } else {
            try self.init(fromCIDInternal: cid)
        }
    }

    /// Inits a `PeerID` based solely on a CID value with no underlying `KeyPair`
    internal init(fromCIDInternal cid: CID) throws {
        guard cid.codec == .libp2p_key || cid.codec == .dag_pb else {
            throw Errors.invalidCIDCodec(expected: "'v0 dag-pb' or 'v1 libp2p-key'", received: "\(cid.codec)")
        }
        self.multihash = cid.multihash
        self.keyPair = nil
    }

    /// Returns the PeerID's id as a self-describing CIDv1 in Base32 (RFC 0001)
    /// return self-describing String representation
    /// in default format from RFC 0001: https://github.com/libp2p/specs/pull/209
    public func toString() -> String {
        self.cidString
    }

    /// This method checks each PeerID's multihash for embedded public keys (via the use of the identity protocol) and converts them to the traditional SHA256 versions before comparing the underlying digests.
    /// - Allows for comparisons between Traditional PeerIDs and Embedded Public Key PeerIDs.
    internal func isEquivalent(to other: PeerID) -> Bool {
        var lhs = self.multihash
        var rhs = other.multihash

        if lhs.algorithm == .identity {
            guard let digest = lhs.digest else { return false }
            guard let mh = try? Multihash(raw: digest, hashedWith: .sha2_256) else { return false }
            lhs = mh
        } else if rhs.algorithm == .identity {
            guard let digest = rhs.digest else { return false }
            guard let mh = try? Multihash(raw: digest, hashedWith: .sha2_256) else { return false }
            rhs = mh
        }

        return lhs == rhs
    }

    /// Returns the PeerID as a SHA256 Base58 Encoding
    /// - This is equivalent to calling `.b58String` for all keys except embedded ED25519 Public Keys
    /// - For Embedded Public Keys, this method will strip the public key from the ID and return the traditional SHA256 encoded value (Qm prefix style)
    public func traditionalB58String() throws -> String {
        if multihash.algorithm == .identity {
            guard let digest = self.multihash.digest else { throw Errors.invalidMultihashDigest }
            let mh = try Multihash(raw: digest, hashedWith: .sha2_256)
            return mh.b58String
        } else {
            return self.b58String
        }
    }

    /// Returns the Peer ID as a printable string without the Qm prefix.
    ///
    /// Example: <peer.ID xxxxxx>
    public func toPrint() -> String {
        self.description
    }

    private func toBase64Pad(_ buf: [UInt8]) -> String {
        buf.asString(base: .base64Pad)
    }
}

extension PeerID: CustomStringConvertible {
    public var description: String {
        let pid = self.b58String
        return "<peer.ID \(dropPrefix(pid).prefix(6))>"
    }

    public var shortDescription: String {
        let pid = self.b58String
        return String(dropPrefix(pid).prefix(6))
    }

    public var debugDescription: String {
        """
            Peer ID
            id: \(id.asString(base: .base58btc))
            pubKey: \(keyPair?.publicKey.asString(base: .base64Pad) ?? "NIL")
            privKey: \(keyPair?.privateKey?.asString(base: .base64Pad) ?? "NIL")
        """
    }

    private func dropPrefix(_ pid: String) -> String.SubSequence {
        var skip = 0
        if pid.hasPrefix("Qm") {
            skip = 2
        } else if pid.hasPrefix("12D3KooW") {
            skip = 8
        }
        return pid.dropFirst(skip)
    }
}

extension Array where Element == UInt8 {
    fileprivate var base64Pad: String {
        self.asString(base: .base64Pad)
    }
}
