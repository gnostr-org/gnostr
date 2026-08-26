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

/// libp2p PBKDF parameters
private struct PBKDF {
    static let algorithmTagLength = 16
    static let nonceLength = 12
    static let keyLength = 16
    static let digest = "sha256"
    static let saltLength = 16
    static let iterations = 32767
}

#if canImport(CommonCrypto)

import CommonCrypto

struct PBKDF2: Sendable {
    static func SHA1(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
        pbkdf2(
            hash: CCPBKDFAlgorithm(kCCPRFHmacAlgSHA1),
            password: password,
            salt: salt,
            keyByteCount: keyByteCount,
            rounds: rounds
        )
    }

    static func SHA256(
        password: String,
        salt: Data,
        keyByteCount: Int = PBKDF.keyLength,
        rounds: Int = PBKDF.iterations
    ) -> Data? {
        pbkdf2(
            hash: CCPBKDFAlgorithm(kCCPRFHmacAlgSHA256),
            password: password,
            salt: salt,
            keyByteCount: keyByteCount,
            rounds: rounds
        )
    }

    static func SHA384(
        password: String,
        salt: Data,
        keyByteCount: Int = PBKDF.keyLength,
        rounds: Int = PBKDF.iterations
    ) -> Data? {
        pbkdf2(
            hash: CCPBKDFAlgorithm(kCCPRFHmacAlgSHA384),
            password: password,
            salt: salt,
            keyByteCount: keyByteCount,
            rounds: rounds
        )
    }

    static func SHA512(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
        pbkdf2(
            hash: CCPBKDFAlgorithm(kCCPRFHmacAlgSHA512),
            password: password,
            salt: salt,
            keyByteCount: keyByteCount,
            rounds: rounds
        )
    }

    private static func pbkdf2(
        hash: CCPBKDFAlgorithm,
        password: String,
        salt: Data,
        keyByteCount: Int,
        rounds: Int
    ) -> Data? {
        guard let passwordData = password.data(using: .utf8) else { return nil }
        var derivedKeyData = Data(repeating: 0, count: keyByteCount)
        let derivedCount = derivedKeyData.count
        let derivationStatus: Int32 = derivedKeyData.withUnsafeMutableBytes { derivedKeyBytes in
            let keyBuffer: UnsafeMutablePointer<UInt8> =
                derivedKeyBytes.baseAddress!.assumingMemoryBound(to: UInt8.self)
            return salt.withUnsafeBytes { saltBytes -> Int32 in
                let saltBuffer: UnsafePointer<UInt8> = saltBytes.baseAddress!.assumingMemoryBound(to: UInt8.self)
                return CCKeyDerivationPBKDF(
                    CCPBKDFAlgorithm(kCCPBKDF2),
                    password,
                    passwordData.count,
                    saltBuffer,
                    salt.count,
                    hash,
                    UInt32(rounds),
                    keyBuffer,
                    derivedCount
                )
            }
        }
        return derivationStatus == kCCSuccess ? derivedKeyData : nil
    }
}

#else

import CryptoSwift

struct PBKDF2: Sendable {
    static func MD5(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
        try? Data(
            CryptoSwift.PKCS5.PBKDF2(
                password: password.bytes,
                salt: salt.byteArray,
                iterations: rounds,
                keyLength: keyByteCount,
                variant: .md5
            ).calculate()
        )
    }

    static func SHA1(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
        try? Data(
            CryptoSwift.PKCS5.PBKDF2(
                password: password.bytes,
                salt: salt.byteArray,
                iterations: rounds,
                keyLength: keyByteCount,
                variant: .sha1
            ).calculate()
        )
    }

    static func SHA256(
        password: String,
        salt: Data,
        keyByteCount: Int = PBKDF.keyLength,
        rounds: Int = PBKDF.iterations
    ) -> Data? {
        try? Data(
            CryptoSwift.PKCS5.PBKDF2(
                password: password.bytes,
                salt: salt.byteArray,
                iterations: rounds,
                keyLength: keyByteCount,
                variant: .sha2(.sha256)
            ).calculate()
        )
    }

    static func SHA384(
        password: String,
        salt: Data,
        keyByteCount: Int = PBKDF.keyLength,
        rounds: Int = PBKDF.iterations
    ) -> Data? {
        try? Data(
            CryptoSwift.PKCS5.PBKDF2(
                password: password.bytes,
                salt: salt.byteArray,
                iterations: rounds,
                keyLength: keyByteCount,
                variant: .sha2(.sha384)
            ).calculate()
        )
    }

    static func SHA512(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
        try? Data(
            CryptoSwift.PKCS5.PBKDF2(
                password: password.bytes,
                salt: salt.byteArray,
                iterations: rounds,
                keyLength: keyByteCount,
                variant: .sha2(.sha512)
            ).calculate()
        )
    }
}

#endif

//extension Data {
//    func pbkdf2SHA1(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
//        return pbkdf2(hash:CCPBKDFAlgorithm(kCCPRFHmacAlgSHA1), password: password, salt: salt, keyByteCount: keyByteCount, rounds: rounds)
//    }
//
//    func pbkdf2SHA256(password: String, salt: Data, keyByteCount: Int = 16, rounds: Int = 32767) -> Data? {
//        //let salt:Data = self.prefix(PBKDF.saltLength)
//        //let ciphertextAndNonce:Data = self.dropFirst(PBKDF.saltLength)
//
//        return pbkdf2(hash:CCPBKDFAlgorithm(kCCPRFHmacAlgSHA256), password: password, salt: salt, keyByteCount: keyByteCount, rounds: rounds)
//    }
//
//    func pbkdf2SHA512(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
//        return pbkdf2(hash:CCPBKDFAlgorithm(kCCPRFHmacAlgSHA512), password: password, salt: salt, keyByteCount: keyByteCount, rounds: rounds)
//    }
//
//    private func pbkdf2(hash: CCPBKDFAlgorithm, password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data? {
//        guard let passwordData = password.data(using: .utf8) else { return nil }
//        var derivedKeyData = Data(repeating: 0, count: keyByteCount)
//        let derivedCount = derivedKeyData.count
//        let derivationStatus: Int32 = derivedKeyData.withUnsafeMutableBytes { derivedKeyBytes in
//            let keyBuffer: UnsafeMutablePointer<UInt8> =
//                derivedKeyBytes.baseAddress!.assumingMemoryBound(to: UInt8.self)
//            return salt.withUnsafeBytes { saltBytes -> Int32 in
//                let saltBuffer: UnsafePointer<UInt8> = saltBytes.baseAddress!.assumingMemoryBound(to: UInt8.self)
//                return CCKeyDerivationPBKDF(
//                    CCPBKDFAlgorithm(kCCPBKDF2),
//                    password,
//                    passwordData.count,
//                    saltBuffer,
//                    salt.count,
//                    hash,
//                    UInt32(rounds),
//                    keyBuffer,
//                    derivedCount)
//            }
//        }
//        return derivationStatus == kCCSuccess ? derivedKeyData : nil
//    }
//}
