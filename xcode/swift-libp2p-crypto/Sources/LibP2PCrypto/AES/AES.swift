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

#if canImport(CommonCrypto)
import CommonCrypto

extension LibP2PCrypto {
    public enum AES {
        static func createKey(key: String) throws -> AESKey {
            try AESKey(key: key)
        }

        static func createKey(key: String, iv: String) throws -> AESKey {
            try AESKey(key: key, iv: iv)
        }

        static func createKey(key: Data, iv: Data) throws -> AESKey {
            try AESKey(key: key, iv: iv)
        }

        public func decrypt(_ data: Data, withPassword password: String) throws -> Data {
            let key = try LibP2PCrypto.AES.createKey(key: password)

            return try key.decrypt(data)
        }

        public struct AESKey: Encryptable, Decryptable, Sendable {
            private let key: Data
            private let iv: Data

            public init(key: Data, iv: Data) throws {
                guard key.count == kCCKeySizeAES128 || key.count == kCCKeySizeAES256 else {
                    throw NSError(domain: "Error: Failed to set a key.", code: 0, userInfo: nil)
                }

                guard iv.count == kCCBlockSizeAES128 else {
                    throw NSError(domain: "Error: Failed to set an initial vector.", code: 0, userInfo: nil)
                }
                self.key = key
                self.iv = iv
            }

            /// Initializes an AES Key with the specified key and Initial Vector
            /// - Parameters:
            ///   - key: Either a 16 or 32 byte secret key
            ///   - iv: A 16 byte initial vector
            /// - Throws: An error if one is encountered along the way
            public init(key: String, iv: String) throws {
                guard let keyData = key.data(using: .utf8), let ivData = iv.data(using: .utf8) else {
                    throw NSError(domain: "Error: Failed to set a key.", code: 0, userInfo: nil)
                }

                self.key = keyData
                self.iv = ivData
            }

            /// Initializes an AES Key with the specified key and a randomly generated Initial Vector
            /// - Parameter key: Either a 16 or 32 byte secret key
            /// - Throws: An error if one is encountered along the way
            public init(key: String) throws {
                guard key.count == kCCKeySizeAES128 || key.count == kCCKeySizeAES256,
                    let keyData = key.data(using: .utf8)
                else {
                    throw NSError(domain: "Error: Failed to set a key.", code: 0, userInfo: nil)
                }
                try self.init(key: keyData, iv: Data(LibP2PCrypto.randomBytes(length: 16)))
            }

            //            public func encrypt(data: Data) throws -> Data {
            //                return try crypt(data: data, operation: CCOperation(kCCEncrypt))
            //            }
            //
            //            public func encrypt(string: String, using encoding:String.Encoding = .utf8) throws -> Data {
            //                guard let d = string.data(using: encoding) else {
            //                    throw NSError(domain: "Error: Failed to encode string using \(encoding).", code: 0, userInfo: nil)
            //                }
            //                return try encrypt(data: d)
            //            }

            public func encrypt(_ data: Data) throws -> Data {
                try crypt(data: data, operation: CCOperation(kCCEncrypt))
            }

            //            /// Converts a String to data using .utf8 encoding and then attempts to encrypt it
            //            public func encrypt(_ message:String) throws -> Data {
            //                guard let d = message.data(using: .utf8) else {
            //                    throw NSError(domain: "Error: Failed to encode string using \(String.Encoding.utf8).", code: 0, userInfo: nil)
            //                }
            //                return try encrypt(d)
            //            }

            public func decrypt(_ data: Data) throws -> Data {
                try crypt(data: data, operation: CCOperation(kCCDecrypt))

            }

            //            public func decrypt(data: Data, using encoding:String.Encoding = .utf8) throws -> String {
            //                let decryptedData = try decrypt(data: data)
            //                guard let str = String(bytes: decryptedData, encoding: encoding) else {
            //                    throw NSError(domain: "Error: Failed to convert data into string via encoding: \(encoding)", code: 0, userInfo: nil)
            //                }
            //                return str
            //            }

            /// AES Encrypt / Decrypt data
            /// - Parameters:
            ///   - data: The data that should be worked on
            ///   - operation: Either Encryption or Decryption
            /// - Throws: An error if the data couldn't be processed
            /// - Returns: The AES Encrypted / Decrypted data
            private func crypt(data: Data, operation: CCOperation) throws -> Data {
                if data.isEmpty { return data }

                let cryptLength = data.count + kCCBlockSizeAES128
                var cryptData = Data(count: cryptLength)

                let keyLength = key.count
                let options = CCOptions(kCCOptionPKCS7Padding)

                var bytesLength = Int(0)

                let status = cryptData.withUnsafeMutableBytes { cryptBytes in
                    data.withUnsafeBytes { dataBytes in
                        iv.withUnsafeBytes { ivBytes in
                            key.withUnsafeBytes { keyBytes in
                                CCCrypt(
                                    operation,  //CCOperation
                                    CCAlgorithm(kCCAlgorithmAES),  //CCAlgorithm
                                    options,  //CCOptions (PKCS7 Padding, etc...)
                                    keyBytes.baseAddress,  //Key Pointer
                                    keyLength,  //Key Length
                                    ivBytes.baseAddress,  //IV Pointer
                                    dataBytes.baseAddress,  //Pointer to Data to encrypt
                                    data.count,  //Length of Data to encrypt
                                    cryptBytes.baseAddress,  //Pointer to encrypted data out
                                    cryptLength,  //Length of encrypted data out
                                    &bytesLength  //The number of bytes written
                                )
                            }
                        }
                    }
                }

                guard UInt32(status) == UInt32(kCCSuccess) else {
                    throw NSError(domain: "Error: Failed to crypt data. Status \(status)", code: 0, userInfo: nil)
                }

                cryptData.removeSubrange(bytesLength..<cryptData.count)
                return cryptData
            }
        }
    }
}

#else

import CryptoSwift

extension LibP2PCrypto {
    public enum AES {
        static func createKey(key: String) throws -> AESKey {
            try AESKey(key: key)
        }

        static func createKey(key: String, iv: String) throws -> AESKey {
            try AESKey(key: key, iv: iv)
        }

        static func createKey(key: Data, iv: Data) throws -> AESKey {
            try AESKey(key: key, iv: iv)
        }

        public func decrypt(_ data: Data, withPassword password: String) throws -> Data {
            let key = try LibP2PCrypto.AES.createKey(key: password)

            return try key.decrypt(data)
        }

        public struct AESKey: Encryptable, Decryptable {
            private let aes: CryptoSwift.AES

            public init(key: Data, iv: Data) throws {

                guard key.count == 16 || key.count == 32 else {
                    throw NSError(domain: "Error: Failed to set a key.", code: 0, userInfo: nil)
                }

                guard iv.count == 16 else {
                    throw NSError(domain: "Error: Failed to set an initial vector.", code: 0, userInfo: nil)
                }

                self.aes = try CryptoSwift.AES(key: key.byteArray, blockMode: CBC(iv: iv.byteArray), padding: .pkcs5)
            }

            /// Initializes an AES Key with the specified key and Initial Vector
            /// - Parameters:
            ///   - key: Either a 16 or 32 byte secret key
            ///   - iv: A 16 byte initial vector
            /// - Throws: An error if one is encountered along the way
            public init(key: String, iv: String) throws {
                guard let keyData = key.data(using: .utf8), let ivData = iv.data(using: .utf8) else {
                    throw NSError(domain: "Error: Failed to set a key.", code: 0, userInfo: nil)
                }

                try self.init(key: keyData, iv: ivData)
            }

            /// Initializes an AES Key with the specified key and a randomly generated Initial Vector
            /// - Parameter key: Either a 16 or 32 byte secret key
            /// - Throws: An error if one is encountered along the way
            public init(key: String) throws {
                guard key.count == 16 || key.count == 32, let keyData = key.data(using: .utf8) else {
                    throw NSError(domain: "Error: Failed to set a key.", code: 0, userInfo: nil)
                }

                try self.init(key: keyData, iv: Data(LibP2PCrypto.randomBytes(length: 16)))
            }

            public func encrypt(_ data: Data) throws -> Data {
                try Data(aes.encrypt(data.byteArray))
            }

            public func decrypt(_ data: Data) throws -> Data {
                try Data(aes.decrypt(data.byteArray))
            }
        }
    }
}

#endif
