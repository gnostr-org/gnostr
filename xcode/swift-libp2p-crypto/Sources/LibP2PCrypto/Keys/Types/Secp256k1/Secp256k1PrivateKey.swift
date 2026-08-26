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

extension Array where Element == UInt8 {

    fileprivate var bigEndianUInt: UInt? {
        guard self.count <= MemoryLayout<UInt>.size else {
            return nil
        }
        var number: UInt = 0
        for i in (0..<self.count).reversed() {
            number = number | (UInt(self[self.count - i - 1]) << (i * 8))
        }

        return number
    }
}

// TODO: Move to P256K implementation and remove @unchecked Sendable

public final class Secp256k1PrivateKey: @unchecked Sendable {

    // MARK: - Properties

    /// The raw private key bytes
    public let rawPrivateKey: [UInt8]

    /// The public key associated with this private key
    public let publicKey: Secp256k1PublicKey

    /// Returns the ethereum address representing the public key associated with this private key.
    //public var address: EthereumAddress {
    //    return publicKey.address
    //}

    /// True iff ctx should not be freed on deinit
    private let ctxSelfManaged: Bool

    /// Internal context for secp256k1 library calls
    private let ctx: OpaquePointer

    // MARK: - Initialization

    /// Initializes a new cryptographically secure `EthereumPrivateKey` from random noise.
    ///
    /// The process of generating the new private key is as follows:
    ///
    /// - Generate a secure random number between 55 and 65.590. Call it `rand`.
    /// - Read `rand` bytes from `/dev/urandom` and call it `bytes`.
    /// - Create the keccak256 hash of `bytes` and initialize this private key with the generated hash.
    public convenience init() throws {
        guard var rand = try? LibP2PCrypto.randomBytes(length: 2).bigEndianUInt else {
            //guard var rand = [UInt8].secureRandom(count: 2)?.bigEndianUInt else {
            throw Error.internalError
        }
        rand += 55

        guard let bytes = try? LibP2PCrypto.randomBytes(length: Int(rand)) else {
            //guard let bytes = [UInt8].secureRandom(count: Int(rand)) else {
            throw Error.internalError
        }
        let bytesHash = SHA3(variant: .keccak256).calculate(for: bytes)

        try self.init(privateKey: bytesHash)
    }

    /// Convenience initializer for `init(privateKey:)`
    public required convenience init(_ bytes: [UInt8]) throws {
        try self.init(privateKey: bytes)
    }

    /// Initializes a new instance of `Secp256k1PrivateKey` with the given `privateKey` Bytes.
    ///
    /// - Parameters:
    ///   - privateKey: The private key bytes. Must be exactly a big endian 32 Byte array representing the private key.
    ///   - ctx: An optional self managed context. If you have specific requirements and
    ///          your app performs not as fast as you want it to, you can manage the
    ///          `secp256k1_context` yourself with the public methods
    ///          `secp256k1_default_ctx_create` and `secp256k1_default_ctx_destroy`.
    ///          If you do this, we will not be able to free memory automatically and you
    ///          __have__ to destroy the context yourself once your app is closed or
    ///          you are sure it will not be used any longer. Only use this optional
    ///          context management if you know exactly what you are doing and you really
    ///          need it.
    /// - throws: EthereumPrivateKey.Error.keyMalformed if the restrictions described above are not met.
    ///           EthereumPrivateKey.Error.internalError if a secp256k1 library call or another internal call fails.
    ///           EthereumPrivateKey.Error.pubKeyGenerationFailed if the public key extraction from the private key fails.
    /// - Note: `privateKey` must be in the secp256k1 range as described in: https://en.bitcoin.it/wiki/Private_key
    /// ```
    /// So any number between
    /// 0x0000000000000000000000000000000000000000000000000000000000000001
    /// and
    /// 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140
    /// is considered to be a valid secp256k1 private key.
    ///  ```
    public init(privateKey: [UInt8], ctx: OpaquePointer? = nil) throws {
        guard privateKey.count == 32 else {
            throw Error.keyMalformed
        }
        self.rawPrivateKey = privateKey

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

        // Generate public key
        guard let pubKey = malloc(MemoryLayout<secp256k1_pubkey>.size)?.assumingMemoryBound(to: secp256k1_pubkey.self)
        else {
            throw Error.internalError
        }
        // Cleanup
        defer {
            free(pubKey)
        }
        var secret = privateKey
        if secp256k1_ec_pubkey_create(finalCtx, pubKey, &secret) != 1 {
            throw Error.pubKeyGenerationFailed
        }

        var pubOut = [UInt8](repeating: 0, count: 65)
        var pubOutLen = 65
        _ = secp256k1_ec_pubkey_serialize(finalCtx, &pubOut, &pubOutLen, pubKey, UInt32(SECP256K1_EC_UNCOMPRESSED))
        guard pubOutLen == 65 else {
            throw Error.pubKeyGenerationFailed
        }

        // First byte is header byte 0x04
        pubOut.remove(at: 0)

        self.publicKey = try Secp256k1PublicKey(publicKey: pubOut, ctx: ctx)
        // End Generate public key

        // Verify private key
        try verifyPrivateKey()
    }

    /// Initializes a new instance of `EthereumPrivateKey` with the given `hexPrivateKey` hex string.
    ///
    /// - Parameters:
    ///   - hexPrivateKey: must be either 64 characters long or 66 characters (with the hex prefix 0x).
    ///   - ctx: An optional self managed context. If you have specific requirements and
    ///          your app performs not as fast as you want it to, you can manage the
    ///          `secp256k1_context` yourself with the public methods
    ///          `secp256k1_default_ctx_create` and `secp256k1_default_ctx_destroy`.
    ///          If you do this, we will not be able to free memory automatically and you
    ///          __have__ to destroy the context yourself once your app is closed or
    ///          you are sure it will not be used any longer. Only use this optional
    ///          context management if you know exactly what you are doing and you really
    ///          need it.
    /// - throws: EthereumPrivateKey.Error.keyMalformed if the restrictions described above are not met.
    ///           EthereumPrivateKey.Error.internalError if a secp256k1 library call or another internal call fails.
    ///           EthereumPrivateKey.Error.pubKeyGenerationFailed if the public key extraction from the private key fails.
    /// - Note: `privateKey` must be in the secp256k1 range as described in: https://en.bitcoin.it/wiki/Private_key
    /// ```
    /// So any number between
    /// 0x0000000000000000000000000000000000000000000000000000000000000001
    /// and
    /// 0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140
    /// is considered to be a valid secp256k1 private key.
    ///  ```
    public convenience init(hexPrivateKey: String, ctx: OpaquePointer? = nil) throws {
        guard hexPrivateKey.count == 64 || hexPrivateKey.count == 66 else {
            throw Error.keyMalformed
        }

        var hexPrivateKey = hexPrivateKey

        if hexPrivateKey.count == 66 {
            let s = hexPrivateKey.index(hexPrivateKey.startIndex, offsetBy: 0)
            let e = hexPrivateKey.index(hexPrivateKey.startIndex, offsetBy: 2)
            let prefix = String(hexPrivateKey[s..<e])

            guard prefix == "0x" else {
                throw Error.keyMalformed
            }

            // Remove prefix
            hexPrivateKey = String(hexPrivateKey[e...])
        }

        //guard let raw = try? BaseEncoding.decode(hexPrivateKey, as: .base16) else {
        //    throw Error.keyMalformed
        //}
        var raw = [UInt8]()
        for i in stride(from: 0, to: hexPrivateKey.count, by: 2) {
            let s = hexPrivateKey.index(hexPrivateKey.startIndex, offsetBy: i)
            let e = hexPrivateKey.index(hexPrivateKey.startIndex, offsetBy: i + 2)

            guard let b = UInt8(String(hexPrivateKey[s..<e]), radix: 16) else {
                throw Error.keyMalformed
            }
            raw.append(b)
        }

        try self.init(privateKey: raw, ctx: ctx)
    }

    // MARK: - Convenient functions

    public func sign(message: [UInt8]) throws -> (v: UInt, r: [UInt8], s: [UInt8]) {
        let hash = SHA3(variant: .keccak256).calculate(for: message)
        return try sign(hash: hash)
    }

    public func sign(hash _hash: [UInt8]) throws -> (v: UInt, r: [UInt8], s: [UInt8]) {
        var hash = _hash
        guard hash.count == 32 else {
            throw Error.internalError
        }
        guard
            let sig = malloc(MemoryLayout<secp256k1_ecdsa_recoverable_signature>.size)?.assumingMemoryBound(
                to: secp256k1_ecdsa_recoverable_signature.self
            )
        else {
            throw Error.internalError
        }
        defer {
            free(sig)
        }

        var seckey = rawPrivateKey

        guard secp256k1_ecdsa_sign_recoverable(ctx, sig, &hash, &seckey, nil, nil) == 1 else {
            throw Error.internalError
        }

        var output64 = [UInt8](repeating: 0, count: 64)
        var recid: Int32 = 0
        secp256k1_ecdsa_recoverable_signature_serialize_compact(ctx, &output64, &recid, sig)

        guard recid == 0 || recid == 1 else {
            // Well I guess this one should never happen but to avoid bigger problems...
            throw Error.internalError
        }

        return (v: UInt(recid), r: Array(output64[0..<32]), s: Array(output64[32..<64]))
    }

    /// Returns this private key serialized as a hex string.
    public func hex() -> String {
        rawPrivateKey.asString(base: .base16)
        //        var h = "0x"
        //        for b in rawPrivateKey {
        //            h += String(format: "%02x", b)
        //        }
        //
        //        return h
    }

    // MARK: - Helper functions

    private func verifyPrivateKey() throws {
        var secret = rawPrivateKey
        guard secp256k1_ec_seckey_verify(ctx, &secret) == 1 else {
            throw Error.keyMalformed
        }
    }

    // MARK: - Errors

    public enum Error: Swift.Error {

        case internalError
        case keyMalformed
        case pubKeyGenerationFailed
    }

    // MARK: - Deinitialization

    deinit {
        if !ctxSelfManaged {
            secp256k1_context_destroy(ctx)
        }
    }
}

// MARK: - Equatable

extension Secp256k1PrivateKey: Equatable {

    public static func == (_ lhs: Secp256k1PrivateKey, _ rhs: Secp256k1PrivateKey) -> Bool {
        lhs.rawPrivateKey == rhs.rawPrivateKey
    }
}

// MARK: - BytesConvertible

extension Secp256k1PrivateKey {

    public func makeBytes() -> [UInt8] {
        rawPrivateKey
    }
}

// MARK: - Hashable

extension Secp256k1PrivateKey: Hashable {

    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawPrivateKey)
    }
}
