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
import LibP2PCrypto
import Multibase

/// - MARK: Marshaled Imports and Exports
extension PeerID {

    /// PeerID Marshalling (protobufs) Related Errors
    public enum MarshallingError: Error {
        /// Marshalled payload doesn't contain any valid keys
        case emptyMarshalledData
        /// This PeerID doesn't have a Private Key to Marshal
        case noPrivateKeyAvailable
        /// This PeerID doesn't have a Public Key to Marshal
        case noPublicKeyAvailable
    }

    /// Inits a `PeerID` from a marshaled `PeerID` string
    /// - Parameters:
    ///   - marshaledPeerID: The marshalled PeerID string
    ///   - base: The base in which the data is encoded, unless the string provided is a valid Multibase string
    ///
    /// - Note: `base` can be left `nil` if the marshaledPeerID String is `Multibase` compliant (includes the multibase prefix) otherwise, you must specify the ecoded base of the string...
    public init(marshaledPeerID: String, base: BaseEncoding? = nil) throws {
        let marshaledData: Data
        if let base = base {
            marshaledData = try BaseEncoding.decode(marshaledPeerID, as: base).data
        } else {
            marshaledData = try BaseEncoding.decode(marshaledPeerID).data
        }
        try self.init(marshaledPeerID: marshaledData)
    }

    /// Inits a `PeerID` from a marshaled `PeerID`
    /// - Parameter data: The marshalled PeerID (serialized protobuf)
    public init(marshaledPeerID data: Data) throws {
        // Attampt to instantiate a PeerIdProto with the raw, marshaled, data
        let protoPeerID = try PeerIdProto(serializedBytes: data)

        //print(protoPeerID.id.asString(base: .base64))
        //print("Has PubKey: \(protoPeerID.hasPubKey)")
        //print(protoPeerID.pubKey.asString(base: .base64))
        //print("Has PrivKey: \(protoPeerID.hasPrivKey)")
        //print(protoPeerID.privKey.asString(base: .base64))

        // Enusre the Marshaled data included at least a public key (is this necessary, would we ever need to unmarshal an ID only?)
        guard protoPeerID.hasPubKey || protoPeerID.hasPrivKey else {
            throw MarshallingError.emptyMarshalledData
        }

        // If we have a private key, attempt to instantiate the PeerID via the private key, otherwise, try the public key...
        if protoPeerID.hasPrivKey {
            try self.init(marshaledPrivateKey: protoPeerID.privKey)
        } else if protoPeerID.hasPubKey {
            try self.init(marshaledPublicKey: protoPeerID.pubKey)
        } else {
            throw MarshallingError.emptyMarshalledData
        }
    }

    /// Inits a `PeerID` from a marshaled public key string
    public init(marshaledPublicKey str: String, base: BaseEncoding) throws {
        try self.init(keyPair: LibP2PCrypto.Keys.KeyPair(marshaledPublicKey: str, base: base))
    }

    /// Inits a `PeerID` from a marshaled public key
    public init(marshaledPublicKey key: Data) throws {
        try self.init(keyPair: LibP2PCrypto.Keys.KeyPair(marshaledPublicKey: key))
    }

    /// Inits a `PeerID` from a marshaled private key string
    public init(marshaledPrivateKey str: String, base: BaseEncoding) throws {
        try self.init(keyPair: LibP2PCrypto.Keys.KeyPair(marshaledPrivateKey: str, base: base))
    }

    /// Inits a `PeerID` from a marshaled private key
    public init(marshaledPrivateKey data: Data) throws {
        try self.init(keyPair: LibP2PCrypto.Keys.KeyPair(marshaledPrivateKey: data))
    }

    //    private static func computeDigest(pubKey:SecKey) throws -> [UInt8] {
    //        let bytes = try pubKey.rawRepresentation()
    //        return try self.computeDigest(rawPubKey: bytes)
    //    }
    //
    //    /// - Note: We need to marshal the raw public key before multihashing it....
    //    private static func computeDigest(rawPubKey bytes:Data) throws -> [UInt8] {
    //        let marshaled = try LibP2PCrypto.Keys.marshalPublicKey(raw: bytes, keyType: .RSA(bits: .B1024))
    //        //print(marshaled.asString(base: .base64Pad))
    //        if marshaled.count <= 42 {
    //            return try Multihash(raw: marshaled, hashedWith: .identity).value
    //        } else {
    //            //let mh = try Multihash(raw: bytes, hashedWith: .sha2_256)
    //            //print("Value: \(mh.value.asString(base: .base16))")
    //            //print("Hex: \(mh.hexString)")
    //            //print("Digest: \(mh.digest?.asString(base: .base16) ?? "NIL")")
    //            return try Multihash(raw: marshaled, hashedWith: .sha2_256).value //pubKey.hash()
    //        }
    //    }

    /// Returns a protocol-buffers encoded version of the id, public key and, if `includingPrivateKey` is set to `true`, the private key.
    public func marshal(includingPrivateKey: Bool = false) throws -> [UInt8] {
        var pid = PeerIdProto()
        pid.id = Data(self.id)
        pid.pubKey = try self.keyPair?.publicKey.marshal() ?? Data()
        if includingPrivateKey, let privKey = self.keyPair?.privateKey {
            pid.privKey = try privKey.marshal()
        }
        return try pid.serializedData().byteArray
    }

    /// Returns a protobuf encoded version of the id and private key
    public func marshalPrivateKey() throws -> [UInt8] {
        guard let privKey = self.keyPair?.privateKey else {
            throw MarshallingError.noPrivateKeyAvailable
        }
        return try privKey.marshal().byteArray
    }

    /// Returns a protobuf encoded version of the id and public key
    public func marshalPublicKey() throws -> [UInt8] {
        guard let pubKey = self.keyPair?.publicKey else {
            throw MarshallingError.noPublicKeyAvailable
        }
        return try pubKey.marshal().byteArray
    }
}

extension Data {
    var byteArray: [UInt8] {
        Array(self)
    }
}
