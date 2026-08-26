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
    public enum Keys {
        public enum ElipticCurveType {
            case P256
            case P384
            case P521

            var bits: Int {
                switch self {
                case .P256:
                    return 256
                case .P384:
                    return 384
                case .P521:
                    return 521
                }
            }

            var description: String {
                "\(bits) Curve"
            }
        }

        public enum RSABitLength: Sendable {
            /// - Warning: RSA Keys with less than 2048 bits are considered insecure
            case B1024
            case B2048
            case B3072
            case B4096
            case custom(bits: Int)

            var bits: Int {
                switch self {
                case .B1024:
                    return 1024
                case .B2048:
                    return 2048
                case .B3072:
                    return 3072
                case .B4096:
                    return 4096
                case .custom(let bits):
                    if bits < 2048 { print("‼️ WARNING: RSA Keys less than 2048 are considered insecure! ‼️") }
                    return bits
                }
            }

            var description: String {
                "\(self.bits) Bit"
            }
        }

        public enum KeyPairType: Sendable {
            case RSA(bits: RSABitLength = .B2048)
            case Ed25519
            case Secp256k1

            //case EC(curve:ElipticCurveType = .P256)
            //case ECDSA(curve:ElipticCurveType = .P256)
            //case ECSECPrimeRandom(curve:ElipticCurveType = .P256)
            //case DSA(bits:Int)
            //case AES(bits:Int)
            //case DES(bits:Int)
            //case CAST
            //case RC2(bits:Int)
            //case RC4(bits:Int)
            //case ThreeDES

            var toProtoType: KeyType {
                switch self {
                case .RSA:
                    return .rsa
                case .Ed25519:
                    return .ed25519
                case .Secp256k1:
                    return .secp256K1
                }
            }

            var toGenericType: GenericKeyType {
                .init(self.toProtoType)
            }

            var name: String {
                switch self {
                case .RSA:
                    return "RSA"
                case .Ed25519:
                    return "ED25519"
                case .Secp256k1:
                    return "Secp256k1"
                }
            }

            var description: String {
                switch self {
                case .RSA(let bits):
                    return "\(bits.description) RSA"
                case .Ed25519:
                    return "ED25519 Curve"
                case .Secp256k1:
                    return "Secp256k1"
                }
            }
        }

        public static func generateKeyPair(_ type: KeyPairType) throws -> KeyPair {
            try LibP2PCrypto.Keys.KeyPair(type)
        }

        /// Converts a protobuf serialized public key into its representative object.
        public static func unmarshalPublicKey(buf: [UInt8], into base: BaseEncoding = .base16) throws -> String {
            let pubKeyProto = try PublicKey(serializedBytes: buf)

            guard !pubKeyProto.data.isEmpty else {
                throw NSError(domain: "Unable to Unmarshal PublicKey", code: 0, userInfo: nil)
            }
            switch pubKeyProto.type {
            case .rsa:
                //let data = try RSAPublicKeyImporter().fromSubjectPublicKeyInfo( pubKeyProto.data )
                //return data.asString(base: base)
                return pubKeyProto.data.asString(base: base)

            case .ed25519:
                return pubKeyProto.data.asString(base: base)
            case .secp256K1:
                return pubKeyProto.data.asString(base: base)
            }

        }

        /// Converts a raw private key string into a protobuf serialized private key.
        public static func marshalPrivateKey(
            raw: String,
            asKeyType: KeyPairType,
            fromBase base: BaseEncoding? = nil
        ) throws -> [UInt8] {
            do {
                let decoded: (base: BaseEncoding, data: Data)
                if let b = base {
                    decoded = try BaseEncoding.decode(raw, as: b)
                } else {
                    decoded = try BaseEncoding.decode(raw)
                }
                return try self.marshalPrivateKey(raw: decoded.data, keyType: asKeyType)
            } catch {
                print(error)
                throw NSError(
                    domain: "Failed to decode raw private key, unknown base encoding.",
                    code: 0,
                    userInfo: nil
                )
            }
        }

        public static func marshalPrivateKey(raw: Data, keyType: KeyPairType) throws -> [UInt8] {
            var privKeyProto = PrivateKey()
            privKeyProto.data = raw
            privKeyProto.type = keyType.toProtoType
            return Array(try privKeyProto.serializedData())
        }

        /// Converts a protobuf serialized private key into its representative object.
        public static func unmarshalPrivateKey(buf: [UInt8], into base: BaseEncoding = .base16) throws -> String {
            let privKeyProto = try PrivateKey(serializedBytes: buf)

            let data = privKeyProto.data
            guard !data.isEmpty else { throw NSError(domain: "Unable to Unmarshal PrivateKey", code: 0, userInfo: nil) }

            return data.asString(base: base)
        }

        /// Converts a public key object into a protobuf serialized public key.
        /// - TODO: PEM Format
        /// - [PEM](https://developer.apple.com/forums/thread/104753)
        /// - [Example of PEM Format](https://github.com/TakeScoop/SwiftyRSA/blob/master/Source/SwiftyRSA.swift#L44)
        /// - [Stackoverflow Question](https://stackoverflow.com/questions/21322182/how-to-store-ecdsa-private-key-in-go)
        ///
        /// Here is a code sample that demonstrates encoding and decoding of keys in Go. It helps to know that you need to connect couple of steps. Crypto algorithm is the fist step, in this case ECDSA key. Then you need standard encoding, x509 is most commontly used standard. Finally you need a file format, PEM is again commonly used one. This is currently most commonly used combination, but feel free to substitute any other algoriths or encoding.
        /// ```
        /// func encode(privateKey *ecdsa.PrivateKey, publicKey *ecdsa.PublicKey) (string, string) {
        ///     x509Encoded, _ := x509.MarshalECPrivateKey(privateKey)
        ///     pemEncoded := pem.EncodeToMemory(&pem.Block{Type: "PRIVATE KEY", Bytes: x509Encoded})

        ///     x509EncodedPub, _ := x509.MarshalPKIXPublicKey(publicKey)
        ///     pemEncodedPub := pem.EncodeToMemory(&pem.Block{Type: "PUBLIC KEY", Bytes: x509EncodedPub})

        ///     return string(pemEncoded), string(pemEncodedPub)
        /// }

        /// func decode(pemEncoded string, pemEncodedPub string) (*ecdsa.PrivateKey, *ecdsa.PublicKey) {
        ///     block, _ := pem.Decode([]byte(pemEncoded))
        ///     x509Encoded := block.Bytes
        ///     privateKey, _ := x509.ParseECPrivateKey(x509Encoded)

        ///     blockPub, _ := pem.Decode([]byte(pemEncodedPub))
        ///     x509EncodedPub := blockPub.Bytes
        ///     genericPublicKey, _ := x509.ParsePKIXPublicKey(x509EncodedPub)
        ///     publicKey := genericPublicKey.(*ecdsa.PublicKey)

        ///     return privateKey, publicKey
        /// }

        /// func test() {
        ///     privateKey, _ := ecdsa.GenerateKey(elliptic.P384(), rand.Reader)
        ///     publicKey := &privateKey.PublicKey

        ///     encPriv, encPub := encode(privateKey, publicKey)

        ///     fmt.Println(encPriv)
        ///     fmt.Println(encPub)

        ///     priv2, pub2 := decode(encPriv, encPub)

        ///     if !reflect.DeepEqual(privateKey, priv2) {
        ///         fmt.Println("Private keys do not match.")
        ///     }
        ///     if !reflect.DeepEqual(publicKey, pub2) {
        ///         fmt.Println("Public keys do not match.")
        ///     }
        /// }
        /// ```
        ///
        public enum ExportedKeyType {
            case PEM
            case JWK
        }

        func exportKey(key: String, password: String, format: ExportedKeyType) {}

        func keyStretcher(cipherType: String, hashType: String, secret: String) {}

        func importKey(encryptedKey: String, password: String) {}

    }
}
