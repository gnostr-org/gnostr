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
//
//  - TODO: Support JWK https://tools.ietf.org/html/rfc7517
//  - TODO: Support PEM format

import Crypto
import Foundation
import Multibase

public enum LibP2PCrypto {

    public static func randomBytes(length: Int) throws -> [UInt8] {
        #if (os(macOS) || os(iOS) || os(watchOS) || os(tvOS)) || os(Linux) || os(Android) || os(Windows)
        var rng = SystemRandomNumberGenerator()
        return (0..<length).map { _ in rng.next() }
        #else
        fatalError("No secure random number generator on this platform.")
        #endif
    }

}

extension String {

    /// Encrypts a string using a plaintext password with the following AES GCM cipher
    ///
    /// * algorithmTagLength = 16,
    /// * nonceLength = 12,
    /// * keyLength = 16,
    /// * digest = 'sha256',
    /// * saltLength = 16,
    /// * iterations = 32767
    /// * algorithm = 'aes-128-gcm'
    public func encryptGCM(password: String) throws -> Data {
        guard let data = self.data(using: .utf8) else {
            throw NSError(domain: "Failed to decode string into data", code: 0, userInfo: nil)
        }
        return try data.encryptGCM(password: password)
    }

    /// Decryptes a BaseEncoded string via AES-GCM password encrypted data and attempts to return the plaintext message...
    public func decryptGCM(password: String, base: BaseEncoding) throws -> String? {
        try String(data: BaseEncoding.decode(self, as: base).data.decryptGCM(password: password), encoding: .utf8)
    }

    public func encrypt(withKey key: Encryptable, encodedUsing encoding: String.Encoding = .utf8) throws -> Data {
        try key.encrypt(self, encodedUsing: encoding)
    }

    /// Attempts to decode the string via the specified base encoding then decrypts it
    public func decrypt(withKey key: Decryptable, baseEncoded base: BaseEncoding) throws -> Data {
        try key.decrypt(baseEncoded: self, base: base)
    }

    /// If the string is multibase encoded compliant (includes multibase prefix), we'll automatically decode it and attempt to decrypt the data
    public func decrypt(withKey key: Decryptable) throws -> Data {
        try key.decrypt(multibaseEncoded: self)
    }

    //    func encrypt(withKeyPair key:LibP2PCrypto.Keys.KeyPair, using encoding:String.Encoding = .utf8) throws -> Data {
    //        guard let d = self.data(using: .utf8) else { throw NSError(domain: "Error during string encoding", code: 0, userInfo: nil) }
    //        return try LibP2PCrypto.Keys.encrypt(d, publicKey: key.publicKey)
    //    }

    //    func decrypt(withKeyPair key:LibP2PCrypto.Keys.KeyPair, using encoding:String.Encoding = .utf8) throws -> Data {
    //        guard let d = self.data(using: .utf8) else { throw NSError(domain: "Error during string encoding", code: 0, userInfo: nil) }
    //        return try LibP2PCrypto.Keys.decrypt(d, privateKey: key.privateKey)
    //    }

    //    func encrypt(withAESKey key:LibP2PCrypto.AES.AESKey, using encoding:String.Encoding = .utf8) throws -> Data {
    //        try key.encrypt(string: self, using: encoding)
    //    }

    //    func encrypt(withHmacKey key:LibP2PCrypto.HMAC.HMACKey) -> Data {
    //        key.encrypt(self)
    //    }
}

extension Data {

    /// Returns the encrypted data in the format [ { salt }  { nonce}  { ciphertext }  { GCM algorithm tag } ]
    public func encryptGCM(password: String) throws -> Data {
        try Data(self.byteArray.encryptGCM(password: password))
    }

    /// Returns  decrypted data that was previously encrypted with `encryptGCM(password:)`
    public func decryptGCM(password: String) throws -> Data {
        try Data(self.byteArray.decryptGCM(password: password))
    }

    public func encrypt(withKey key: Encryptable) throws -> Data {
        try key.encrypt(self)
    }

    public func decrypt(withKey key: Decryptable) throws -> Data {
        try key.decrypt(self)
    }
}

extension Array where Element == UInt8 {
    public func encrypt(withKey key: Encryptable) throws -> Data {
        try key.encrypt(self)
    }

    public func decrypt(withKey key: Decryptable) throws -> Data {
        try key.decrypt(self)
    }

    /// Returns the encrypted data in the format [ { salt }  { nonce}  { ciphertext }  { GCM algorithm tag } ]
    public func encryptGCM(password: String) throws -> [UInt8] {
        // Generate a 128-bit salt using a CSPRNG.
        let salt = try LibP2PCrypto.randomBytes(length: 16)

        // Attempt to derive the aes encryption key from the password and salt
        // PBKDF2-SHA256
        guard let key = PBKDF2.SHA256(password: password, salt: Data(salt), keyByteCount: 16, rounds: 32767) else {
            throw NSError(
                domain: "Failed to derive AESGCM encryption key from plaintext password",
                code: 0,
                userInfo: nil
            )
        }

        // Return the salt prepended to the encrypted data
        return try salt + encryptGCM(data: self, withKey: key)
    }

    /// Returns  decrypted data that was previously encrypted with `encryptGCM(password:)`
    public func decryptGCM(password: String) throws -> [UInt8] {
        var data = self

        // Generate a 128-bit salt using a CSPRNG.
        let salt = data.prefix(16)

        // Strip the salt
        data.removeFirst(16)

        // Attempt to derive the aes encryption key from the password and salt
        // PBKDF2-SHA256
        guard let key = PBKDF2.SHA256(password: password, salt: Data(salt), keyByteCount: 16, rounds: 32767) else {
            throw NSError(
                domain: "Failed to derive AESGCM encryption key from plaintext password",
                code: 0,
                userInfo: nil
            )
        }

        return try decryptGCM(data: data, withKey: key)
    }

    private func encryptGCM(data: [UInt8], withKey key: Data) throws -> [UInt8] {
        let nonce = try LibP2PCrypto.randomBytes(length: 12)

        // AES - GCM (CryptoSwift)
        //let aesGCM = try AES(key: key.bytes, blockMode: GCM(iv: nonce, mode: .combined), padding: .noPadding)
        // Encrypt and prepend nonce.
        //let ciphertext = try aesGCM.encrypt(data)

        // AES - GCM (swift-crypto)
        let aesGCM = try AES.GCM.seal(data, using: SymmetricKey(data: key), nonce: AES.GCM.Nonce(data: nonce))

        //let ciphertext = aesGCM

        return aesGCM.combined?.byteArray ?? []  //nonce + ciphertext
    }

    private func decryptGCM(data: [UInt8], withKey key: Data) throws -> [UInt8] {
        //Strip the nonce off the front of the data
        //let nonce = Array(data.prefix(12))

        // AES - GCM (CryptoSwift)
        //        let aesGCM = try AES(key: key.bytes, blockMode: GCM(iv: nonce, mode: .combined), padding: .noPadding)
        // Decrypt the ciphertext
        //        return try aesGCM.decrypt(data.dropFirst(12))

        try AES.GCM.open(AES.GCM.SealedBox(combined: data), using: SymmetricKey(data: key)).byteArray

    }
}
