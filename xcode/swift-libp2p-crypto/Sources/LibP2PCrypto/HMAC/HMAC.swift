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
import Multibase

extension LibP2PCrypto {
    public enum HMAC {
        public enum CryptoAlgorithm: Sendable {
            case MD5, SHA1, SHA256, SHA384, SHA512

            internal func encrypt(_ message: Data, key: String) -> Data {
                switch self {
                case .MD5:
                    var hmac = Crypto.HMAC<Insecure.MD5>(key: SymmetricKey(data: key.bytes))
                    hmac.update(data: message)
                    return Data(hmac.finalize())
                case .SHA1:
                    var hmac = Crypto.HMAC<Insecure.SHA1>(key: SymmetricKey(data: key.bytes))
                    hmac.update(data: message)
                    return Data(hmac.finalize())
                case .SHA256:
                    var hmac = Crypto.HMAC<SHA256>(key: SymmetricKey(data: key.bytes))
                    hmac.update(data: message)
                    return Data(hmac.finalize())
                case .SHA384:
                    var hmac = Crypto.HMAC<SHA384>(key: SymmetricKey(data: key.bytes))
                    hmac.update(data: message)
                    return Data(hmac.finalize())
                case .SHA512:
                    var hmac = Crypto.HMAC<SHA512>(key: SymmetricKey(data: key.bytes))
                    hmac.update(data: message)
                    return Data(hmac.finalize())
                }
            }
        }

        public struct HMACKey: Encryptable, Sendable {
            private let algorithm: CryptoAlgorithm
            private let secret: String

            /// Initializes a reusable HMAC Key by specifying the Hashing Algorithm and secret key to use
            /// - Parameters:
            ///   - algorithm: The Hashing Algorithm to use
            ///   - secret: The shared secret key
            public init(algorithm: CryptoAlgorithm, secret: String) {
                self.algorithm = algorithm
                self.secret = secret
            }

            public func encrypt(_ message: Data) -> Data {
                LibP2PCrypto.HMAC.encrypt(message, algorithm: self.algorithm, key: self.secret)
            }

            public func encrypt(_ message: String) -> Data {
                LibP2PCrypto.HMAC.encrypt(message: message, algorithm: self.algorithm, key: self.secret)
            }

            public func verify(_ str: String, hash: Data) -> Bool {
                self.encrypt(str) == hash
            }

            public func verify(_ data: Data, hash: Data) -> Bool {
                self.encrypt(data) == hash
            }
        }

        /// Creates an HMACKey that can be used to encrypt / hash multiple data sets...
        /// - Parameters:
        ///   - algorithm: The hashing algorithm to use
        ///   - secret: The shared secret key to hash the data against
        /// - Returns: HMACKey
        public static func create(algorithm: CryptoAlgorithm, secret: String) -> LibP2PCrypto.HMAC.HMACKey {
            HMACKey(algorithm: algorithm, secret: secret)
        }

        /// A one offf, stateless, HMAC hashing / encryption method
        /// - Parameters:
        ///   - message: The String message to encrypt / hash
        ///   - algorithm: The hashing algorithm to use
        ///   - key: The shared secret key to hash the data against
        /// - Returns: The encrypted / hashed HMAC data
        public static func encrypt(message: String, algorithm: CryptoAlgorithm, key: String) -> Data {
            let data = message.data(using: .utf8)
            return self.encrypt(data!, algorithm: algorithm, key: key)
        }

        /// A one offf, stateless, HMAC hashing / encryption method
        /// - Parameters:
        ///   - message: The data to encrypt / hash
        ///   - algorithm: The hashing algorithm to use
        ///   - key: The shared secret key to hash the data against
        /// - Returns: The encrypted / hashed HMAC data
        public static func encrypt(_ message: Data, algorithm: CryptoAlgorithm, key: String) -> Data {
            algorithm.encrypt(message, key: key)
        }
    }
}

public protocol Encryptable {
    func encrypt(_ message: Data) throws -> Data
}

// An Encryptable extension that gives us default implementations of Bytes and String Encrypting
extension Encryptable {

    public func encrypt(_ bytes: [UInt8]) throws -> Data {
        try self.encrypt(Data(bytes))
    }

    /// A default string -> data implementation using .utf8 encoding...
    public func encrypt(_ message: String, encodedUsing encoding: String.Encoding = .utf8) throws -> Data {
        guard let d = message.data(using: encoding) else {
            throw NSError(domain: "Error: Failed to encode string using \(encoding).", code: 0, userInfo: nil)
        }
        return try self.encrypt(d)
    }

}

public protocol Decryptable {
    func decrypt(_ message: Data) throws -> Data
}

// An Encryptable extension that gives us default implementations of Bytes and String Encrypting
extension Decryptable {

    public func decrypt(_ bytes: [UInt8]) throws -> Data {
        try self.decrypt(Data(bytes))
    }

    /// Attempts to decode a base encoded string into the encrypted data to be decrypted...
    public func decrypt(baseEncoded: String, base: BaseEncoding) throws -> Data {
        let d = try Multibase.BaseEncoding.decode(baseEncoded, as: base)
        return try self.decrypt(d.data)
    }

    /// Attempts to decode a multibase compliant string into the encrypted data to be decrypted...
    public func decrypt(multibaseEncoded: String) throws -> Data {
        let d = try Multibase.BaseEncoding.decode(multibaseEncoded)
        return try self.decrypt(d.data)
    }

    public func decrypt(_ data: Data, intoString encoding: String.Encoding = .utf8) throws -> String {
        let d = try self.decrypt(data)
        guard let str = String(data: d, encoding: encoding) else {
            throw NSError(domain: "Error: Failed to encode data using \(encoding).", code: 0, userInfo: nil)
        }
        return str
    }

    public func decrypt(_ bytes: [UInt8], intoString encoding: String.Encoding = .utf8) throws -> String {
        let d = try self.decrypt(bytes)
        guard let str = String(data: d, encoding: encoding) else {
            throw NSError(domain: "Error: Failed to encode data using \(encoding).", code: 0, userInfo: nil)
        }
        return str
    }

    public func decrypt(
        baseEncoded: String,
        base: BaseEncoding,
        intoString encoding: String.Encoding = .utf8
    ) throws -> String {
        let d = try self.decrypt(baseEncoded: baseEncoded, base: base)
        guard let str = String(data: d, encoding: encoding) else {
            throw NSError(domain: "Error: Failed to encode data using \(encoding).", code: 0, userInfo: nil)
        }
        return str
    }

    public func decrypt(multibaseEncoded: String, intoString encoding: String.Encoding = .utf8) throws -> String {
        let d = try self.decrypt(multibaseEncoded: multibaseEncoded)
        guard let str = String(data: d, encoding: encoding) else {
            throw NSError(domain: "Error: Failed to encode data using \(encoding).", code: 0, userInfo: nil)
        }
        return str
    }
}
