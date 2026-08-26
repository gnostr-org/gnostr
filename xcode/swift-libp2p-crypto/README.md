# LibP2PCrypto

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-crypto/actions/workflows/build+test.yml/badge.svg)

> Core LibP2P cryptography API for interacting with various Keys, Hashes and Ciphers 

## Table of Contents

- [Overview](#overview)
- [Disclaimer](#disclaimer)
- [Install](#install)
- [Usage](#usage)
  - [Example](#example)
  - [API](#api)
- [Contributing](#contributing)
- [Credits](#credits)
- [Resources](#resources)
  - [Links](#links) 
- [License](#license)

## Overview
LibP2PCrypto is an API / abstraction layer for commonly used cryptography within the LibP2P ecosystem. 
This library...
- Wraps Public Private Key Pairs such as RSA, ED25519, Secp256k1 and EC keys in a common KeyPair class that can perform signing and signature verification without having to worry about the nuances of each algorithm. 
- Tries to make importing PEM, CERTIFICATES and DER files a little easier. 
- Provides methods for Marshaling Public and Private Keys for use in LibP2P PeerIDs and CIDs. 
- Makes various HMAC and AES Ciphers available under a common API as well.

#### Note:
This package currently doesn't support Blake2b, Blake2s & Blake3. If you're up for the challenge, please feel free to add support!

## Disclaimer
‼️ This is a work in progress ‼️ 

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-crypto.git", .upToNextMajor(from: "0.0.1"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PCrypto", package: "swift-libp2p-crypto"),
            ]),
    ...
)
```

## Usage

### Example 
check out the [tests](https://github.com/SwiftEthereum/libp2p-crypto/blob/main/Tests/libp2p-cryptoTests/libp2p_cryptoTests.swift) for more examples

```Swift

import LibP2PCrypto

/// Generate a new Public / Private Key Pair
let rsaKeyPair = try LibP2PCrypto.Keys.generateKeyPair(.RSA(bits: .B2048))
rsaKeyPair.keyType          // -> .rsa
rsaKeyPair.publicKey        // -> PublicKey
rsaKeyPair.publicKey.data   // -> raw public key data
rsaKeyPair.privateKey       // -> optional(PrivateKey)

/// Key Types
try LibP2PCrypto.Keys.generateKeyPair(.RSA(bits: .B2048))   // RSA w/ Bit Options... 1024, 2048, 3072, 4096 
try LibP2PCrypto.Keys.generateKeyPair(.Ed25519)             // Ed25519 Elliptic Key
try LibP2PCrypto.Keys.generateKeyPair(.Secp256k1)           // Secp256k1 Key


/// Importing Marshalled Keys 
let pubKey = try LibP2PCrypto.Keys.importMarshaledPublicKey(marshaledPublicKeyData)
/// or using a marshaled private key...
let privateKey = try LibP2PCrypto.Keys.importMarshaledPrivateKey(marshaledPrivateKeyData)
/// or instantiate a KeyPair instead... (if you pass in a public key, the private key will be nil, if you pass in a private key, both public and private keys will be imported/extracted)
let keyPair = try LibP2PCrypto.Keys.KeyPair(marshaledPrivateKey: marshaledPrivateKeyData)
let keyPair = try LibP2PCrypto.Keys.KeyPair(marshaledPrivateKey: marshaledPrivateKeyAsBase64String, base: .base64Pad) // Or you can use a String encoding of the key data


/// Importing PEM files (PEM, Encrypted PEM, DER and CERTIFICATES are supported. See the tests for more examples...)
let pem = """
-----BEGIN PUBLIC KEY-----
MCowBQYDK2VwAyEACM3Nzttt7KmXG9qDEYys++oQ9G749jqrbRRs92BUzpA=
-----END PUBLIC KEY-----
"""

let keyPair = try LibP2PCrypto.Keys.KeyPair(pem: pem)

keyPair.keyType     // -> .ed25519
keyPair.publicKey   // -> Public Key (can be used for verifying signed messages...)
keyPair.privateKey  // -> nil (private key is nil when importing public keys...)

/// You can import encrypted PEM files as well...
let keyPair = try LibP2PCrypto.Keys.KeyPair(pem: encryptedPem, password: "mypassword")


/// Once you have a KeyPair, you can use it to.... 

/// Sign & Verify Messages 
let keyPair = try LibP2PCrypto.Keys.generateKeyPair(.Ed25519)
let message = "Hello, world!".data(using: .utf8)!
let signedData = try keyPair.privateKey!.sign(message: message)

if try keyPair.publicKey.verfiy(signedData, for: message) {
 // The signature is valid! It means that the public key used to verify the signed message was derived from the same private key that signed the message
} else { 
 // Invalid signature. This public key does not belong to the private key that signed the message. 
}

/// If you want to share the public key, you can export it by marshaling it 
try keyPair.publicKey.marshal()

/// If you want to safely store the Private key between sessions you can export it as an encrypted PEM String
try keyPair.exportEncryptedPrivatePEMString(withPassword "mypassword")



/// AES Cipher (streaming encryption cipher)
let message = "Hello World!"
let key256  = "12345678901234561234567890123456"   // 32 bytes for AES256
let iv      = "abcdefghijklmnop"                   // 16 bytes for AES128

let aes256Key = try LibP2PCrypto.AES.createKey(key: key256, iv: iv)

let encrypted = try aes256Key.encrypt(message)
let decrypted:String = try aes256Key.decrypt(encrypted)

/// AES GCM 
let message = "Hello World!"
let encrypted = try message.encryptGCM(password: "mypassword")
let decrypted = try encrypted.decryptGCM(password: "mypassword")



/// HMAC (Hashed Message Authentication Codes)
let message = "Hello World!"
let key = "secret"
let hmacKeyLocal = LibP2PCrypto.HMAC.create(algorithm: .SHA256, secret: key)
let hmacKeyRemote = LibP2PCrypto.HMAC.create(algorithm: .SHA256, secret: key)

let encrypted = hmacKeyLocal.encrypt(message)

hmacKeyRemote.verify(message, hash: encrypted)          // -> Returns true. Correct data + correct HMAC == true 
hmacKeyRemote.verify("HellØ world!", hash: encrypted)   // -> Returns false. Corrupted data + correct HMAC == false 


/// PBKDF2 (password based key derivation function 2)
let key = PBKDF2.SHA1(password: "mypassword", salt: salt, keyByteCount: 16, rounds: itterations)


```

### API
```Swift

/// Generate Keys
LibP2PCrypto.Keys.generateKeyPair(_ type:KeyPairType) throws -> KeyPair

/// Import Marshaled Keys
LibP2PCrypto.Keys.KeyPair(marshaledPublicKey str:String, base:BaseEncoding) throws
LibP2PCrypto.Keys.KeyPair(marshaledPublicKey data:Data) throws
LibP2PCrypto.Keys.KeyPair(marshaledPrivateKey str:String, base:BaseEncoding) throws
LibP2PCrypto.Keys.KeyPair(marshaledPrivateKey data:Data) throws

/// Import PEM formatted Keys
LibP2PCrypto.Keys.KeyPair(pem:String, password:String? = nil) throws -> KeyPair
LibP2PCrypto.Keys.KeyPair(pem:Data, password:String? = nil) throws -> KeyPair

/// Sign & Verify
RawPrivateKey.sign(message: Data) throws -> Data
RawPublicKey.verfiy(_ signature: Data, for expectedData:Data) throws -> Bool

/// AES CIPHER
LibP2PCrypto.AES.createKey(key:String, iv:String) throws -> AESKey
LibP2PCrypto.AES.AESKey.encrypt(_ data:Data) throws -> Data
LibP2PCrypto.AES.AESKey.decrypt(_ data: Data) throws -> Data

/// HMAC
LibP2PCrypto.HMAC.create(algorithm:CryptoAlgorithm, secret:String) -> LibP2PCrypto.HMAC.HMACKey
LibP2PCrypto.HMAC.HMACKey.encrypt(_ message:Data) -> Data
LibP2PCrypto.HMAC.HMACKey.verify(_ data:Data, hash:Data) -> Bool

/// PBKDF2
LibP2PCrypto.PBKDF2.SHA1(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data?
LibP2PCrypto.PBKDF2.SHA256(password: String, salt: Data, keyByteCount: Int = 16, rounds: Int = 32767) -> Data?
LibP2PCrypto.PBKDF2.SHA512(password: String, salt: Data, keyByteCount: Int, rounds: Int) -> Data?

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [krzyzanowskim - CryptoSwift](https://github.com/krzyzanowskim/CryptoSwift)
- [Koray Koska - Secp256k1](https://github.com/bitcoin-core/secp256k1)
- [siemensikkema - JWT-Kit](https://github.com/vapor/jwt-kit.git) 
- [RSA Import/Export](https://github.com/nextincrement/rsa-public-key-importer-exporter.git)
- [CommonCrypto, Crypto-Kit and swift-crypto libraries]

## Resources
### Links
- [JWK to PEM](https://github.com/ibm-cloud-security/Swift-JWK-to-PEM)
- [OpenSSL](https://stackoverflow.com/questions/31380713/how-to-add-openssl-to-a-swift-project)
- [SecItemExport](https://developer.apple.com/documentation/security/1394828-secitemexport?language=objc)
- [SwiftyRSA](https://github.com/TakeScoop/SwiftyRSA)
- [Swift Sodium (Blake2B)](https://github.com/jedisct1/swift-sodium#hashing)
- [Create a SecKey from a Cert](https://stackoverflow.com/questions/10579985/how-can-i-get-seckeyref-from-der-pem-file)
- [PEM Thread](https://developer.apple.com/forums/thread/96278)
- [SECKey from PEM](https://developer.apple.com/forums/thread/104753)
- [How SwiftyRSA does it](https://github.com/TakeScoop/SwiftyRSA/blob/master/Source/SwiftyRSA.swift#L44)
- [More PEM Discussion](https://developer.apple.com/forums/thread/85915?answerId=256298022#256298022)
- [PEM and DER Format](https://tls.mbed.org/kb/cryptography/asn1-key-structures-in-der-and-pem)
- [ASN.1 Overview](https://www.obj-sys.com/asn1tutorial/node4.html)

## License

[MIT](LICENSE) © 2022 Breth Inc.























