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
import Multibase
import secp256k1

public func secp256k1_default_ctx_create(errorThrowable: Error) throws -> OpaquePointer {
    let c = secp256k1_context_create(UInt32(SECP256K1_CONTEXT_SIGN) | UInt32(SECP256K1_CONTEXT_VERIFY))
    guard let ctx = c else {
        throw errorThrowable
    }

    guard var rand = try? LibP2PCrypto.randomBytes(length: 32) else {
        throw errorThrowable
    }

    guard secp256k1_context_randomize(ctx, &rand) == 1 else {
        throw errorThrowable
    }

    return ctx
}

public func secp256k1_default_ctx_destroy(ctx: OpaquePointer) {
    secp256k1_context_destroy(ctx)
}

// TODO: Move to P256K implementation and remove @unchecked Sendable

public final class Secp256k1PublicKey: @unchecked Sendable {

    static let UNCOMPRESSED_LENGTH = 64
    static let UNCOMPRESSED_LENGTH_WITH_HEADER = 65
    static let COMPRESSED_LENGTH = 32
    static let COMPRESSED_LENGTH_WITH_HEADER = 33

    public enum KeyFormat: UInt8 {
        case EVEN = 0x02
        case ODD = 0x03
        case UNCOMPRESSED = 0x04
        case HYBRID_EVEN = 0x06
        case HYBRID_ODD = 0x07
    }

    // MARK: - Properties

    /// The raw uncompressed public key bytes (without the 0x04 header prefix)
    public let rawPublicKey: [UInt8]

    /// True iff ctx should not be freed on deinit
    private let ctxSelfManaged: Bool

    /// Internal context for secp256k1 library calls
    private let ctx: OpaquePointer

    /// Internal context for secp256k1 library calls
    private let key: secp256k1_pubkey

    // MARK: - Initialization

    /// Convenient initializer for `init(publicKey:)`
    public required convenience init(_ bytes: [UInt8]) throws {
        try self.init(publicKey: bytes)
    }

    /// Initializes a new instance of `Secp256k1PublicKey` with the given raw public key Bytes.
    /// - Parameters:
    ///   - rawPublicKeyData: The public key, either compressed or uncompressed, with the proper key type header prefix (0x04 in the case of standard uncompressed key).
    ///   - ctx: An optional self managed context. If you have specific requirements and your app performs not as fast as you want it to, you can manage the `secp256k1_context` yourself with the public methods `secp256k1_default_ctx_create` and `secp256k1_default_ctx_destroy`. If you do this, we will not be able to free memory automatically and you __have__ to destroy the context yourself once your app is closed or you are sure it will not be used any longer. Only use this optional context management if you know exactly what you are doing and you really need it.
    /// - Throws:
    ///    Secp256k1PublicKey.Error.keyMalformed if the given `publicKey` does not fulfill the requirements from above. Secp256k1PublicKey.Error.internalError if a secp256k1 library call or another internal call fails.
    public init(publicKey rawPublicKeyData: [UInt8], ctx: OpaquePointer? = nil) throws {
        // Create a mutable copy of the raw bytes
        var rawPublicKeyData = rawPublicKeyData

        // WARNING:
        // We assume if we're provided a 64 byte key its the standard uncompressed key without the 0x04 header
        // This is a bad assumption because it would also be a hybrid key
        if rawPublicKeyData.count == Secp256k1PublicKey.UNCOMPRESSED_LENGTH {
            rawPublicKeyData.insert(KeyFormat.UNCOMPRESSED.rawValue, at: 0)
        }

        // Create context
        let finalCtx: OpaquePointer
        if let ctx = ctx {
            finalCtx = ctx
            self.ctxSelfManaged = true
        } else {
            let ctx = try secp256k1_default_ctx_create(errorThrowable: Error.internalError)
            finalCtx = ctx
            self.ctxSelfManaged = false
        }
        self.ctx = finalCtx

        var pubKey = secp256k1_pubkey()
        // Attempt to parse the public key data
        let res = secp256k1_ec_pubkey_parse(finalCtx, &pubKey, &rawPublicKeyData, rawPublicKeyData.count)
        // Check for parsing errors
        guard res == 1 else {
            throw NSError(domain: "Secp256k1::ParsePublicKey::Unable to parse public key", code: 0)
        }

        // Attempt to serialize the pubkey in it's uncompressed form
        var uncompressedPubKey = [UInt8](repeating: 0, count: Secp256k1PublicKey.UNCOMPRESSED_LENGTH_WITH_HEADER)
        var pubKeyLength = Secp256k1PublicKey.UNCOMPRESSED_LENGTH_WITH_HEADER
        let res2 = secp256k1_ec_pubkey_serialize(
            finalCtx,
            &uncompressedPubKey,
            &pubKeyLength,
            &pubKey,
            UInt32(SECP256K1_EC_UNCOMPRESSED)
        )
        // Check for serialization errors
        guard res2 == 1 else {
            throw NSError(domain: "Secp256k1::ParsePublicKey::Unable to serialize uncompressed public key", code: 0)
        }

        // Store the uncompressed public key in our rawPublicKey field
        self.rawPublicKey = Array(uncompressedPubKey.dropFirst())
        self.key = pubKey
    }

    public func compressPublicKey() throws -> [UInt8] {
        var compressedPubKey = [UInt8](repeating: 0, count: Secp256k1PublicKey.COMPRESSED_LENGTH_WITH_HEADER)
        var pubKeyLength = Secp256k1PublicKey.COMPRESSED_LENGTH_WITH_HEADER
        var pubkey = self.key
        let res = secp256k1_ec_pubkey_serialize(
            self.ctx,
            &compressedPubKey,
            &pubKeyLength,
            &pubkey,
            UInt32(SECP256K1_EC_COMPRESSED)
        )
        guard res == 1 else {
            throw NSError(domain: "Unable to uncompress pubkey", code: 0)
        }
        guard compressedPubKey.count == pubKeyLength,
            compressedPubKey.count == Secp256k1PublicKey.COMPRESSED_LENGTH_WITH_HEADER
        else {
            throw NSError(
                domain: "Uncompressed Key Length Mismatch \(compressedPubKey.count) != \(pubKeyLength)",
                code: 0
            )
        }
        return compressedPubKey
    }

    /// Initializes a new instance of `SecP256k1PublicKey` with the given a hex string.
    /// - Parameter hexPublicKey: The uncompressed (or compressed) hex public key either with the hex prefix `0x` or without.
    /// - throws: SecP256k1PublicKey.Error.keyMalformed if the given `hexPublicKey` does not fulfill the requirements from above. Or a SecP256k1PublicKey.Error.internalError if a secp256k1 library fails to parse / validate the provided key.
    public convenience init(hexPublicKey: String) throws {
        let byteCount = hexPublicKey.count
        guard byteCount == 128 || byteCount == 130 || byteCount == 64 || byteCount == 66 else {
            throw Error.keyMalformed
        }

        try self.init(publicKey: Array(try BaseEncoding.decode(hexPublicKey, as: .base16).data))
    }

    // MARK: - Signatures

    public func verifySignature(message: [UInt8], v: [UInt8], r: [UInt8], s: [UInt8]) throws -> Bool {
        // Get public key
        var rawpubKey = rawPublicKey
        rawpubKey.insert(KeyFormat.UNCOMPRESSED.rawValue, at: 0)
        guard let pubkey = malloc(MemoryLayout<secp256k1_pubkey>.size)?.assumingMemoryBound(to: secp256k1_pubkey.self)
        else {
            throw Error.internalError
        }
        defer {
            free(pubkey)
        }
        guard
            secp256k1_ec_pubkey_parse(ctx, pubkey, &rawpubKey, Secp256k1PublicKey.UNCOMPRESSED_LENGTH_WITH_HEADER) == 1
        else {
            throw Error.keyMalformed
        }

        // Create raw signature array
        var rawSig: [UInt8] = []

        // Ensure the provided R and S values are the correct length
        guard r.count <= 32 && s.count <= 32 else {
            throw Error.signatureMalformed
        }

        // Ensure the provided V value is valid
        guard let vInt = Int32(v.asString(base: .base16), radix: 16), vInt >= 0, vInt <= 3 else {
            print("Invalid v param")
            throw Error.signatureMalformed
        }

        // Prepare the signature bytes
        rawSig.append(contentsOf: r)
        rawSig.append(contentsOf: s)

        // Parse recoverable signature
        guard
            let recsig = malloc(MemoryLayout<secp256k1_ecdsa_recoverable_signature>.size)?.assumingMemoryBound(
                to: secp256k1_ecdsa_recoverable_signature.self
            )
        else {
            throw Error.internalError
        }
        defer {
            free(recsig)
        }
        guard secp256k1_ecdsa_recoverable_signature_parse_compact(ctx, recsig, &rawSig, vInt) == 1 else {
            throw Error.signatureMalformed
        }

        // Convert to normal signature
        guard
            let sig = malloc(MemoryLayout<secp256k1_ecdsa_signature>.size)?.assumingMemoryBound(
                to: secp256k1_ecdsa_signature.self
            )
        else {
            throw Error.internalError
        }
        defer {
            free(sig)
        }
        guard secp256k1_ecdsa_recoverable_signature_convert(ctx, sig, recsig) == 1 else {
            throw Error.internalError
        }

        // Check validity with signature
        var hash = SHA3(variant: .keccak256).calculate(for: message)
        guard hash.count == 32 else {
            throw Error.internalError
        }
        return secp256k1_ecdsa_verify(ctx, sig, &hash, pubkey) == 1
    }

    /// Returns this public key serialized as a hex string.
    /// - Uncompressed 64 byte public key without the header prefix (0x04)
    public func hex() -> String {
        rawPublicKey.asString(base: .base16)
    }

    // MARK: - Errors

    public enum Error: Swift.Error {

        case internalError
        case keyMalformed
        case signatureMalformed
    }

    // MARK: - Deinitialization

    deinit {
        if !ctxSelfManaged {
            secp256k1_context_destroy(ctx)
        }
    }
}

// MARK: - Equatable

extension Secp256k1PublicKey: Equatable {

    public static func == (_ lhs: Secp256k1PublicKey, _ rhs: Secp256k1PublicKey) -> Bool {
        lhs.rawPublicKey == rhs.rawPublicKey
    }
}

// MARK: - BytesConvertible

extension Secp256k1PublicKey {

    public func makeBytes() -> [UInt8] {
        rawPublicKey
    }
}

// MARK: - Hashable

extension Secp256k1PublicKey: Hashable {

    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawPublicKey)
    }
}
