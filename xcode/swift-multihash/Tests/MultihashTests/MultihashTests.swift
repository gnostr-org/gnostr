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
import Multibase
import Multicodec
import Testing
import VarInt

@testable import Multihash

@Suite("Multihash Tests")
struct MultihashTests {

    /// Multihash inits
    /// - from multihash
    ///     multihash string in any base
    ///     multihash data
    /// - generate multihash
    ///     raw string with encoding and hash
    ///     raw hex string
    ///     raw b58 string
    ///

    @Test func testHashFunctions() throws {
        for test in MultihashTests.TestFixtures {
            let mh = try Multihash(
                raw: test.input,
                hashedWith: try Codecs(test.algorithm),
                customByteLength: Int(test.bits)! / 8
            )
            //print(mh.hexString)
            #expect(mh.asString(base: .base16) == test.multihash)
            #expect(mh.name == test.algorithm)
            #expect(mh.length == Int(test.bits)! / 8)
            #expect(try mh.code == Int(Codecs(test.algorithm).code))
        }
    }

    @Test func testHashFunctionsManually() throws {
        for test in MultihashTests.TestFixtures {
            let data = Array(test.input.data(using: .utf8)!)
            //let mh = try encodeBuf(Array(hash.data), code: test.0)
            var hash: [UInt8]
            switch test.0 {
            case "sha1":
                hash = Array(data).sha1()
            case "sha2-256":
                hash = Array(data).sha2(.sha256)
            case "sha2-512":
                hash = Array(data).sha2(.sha512)
            case "sha3-512":
                hash = Array(data).sha3(.sha512)
            default:
                Issue.record("Unknown hash function \(test.0)")
                return
            }
            let mh = try encodeMultihashBuffer(Array(hash.prefix(Int(test.bits)! / 8)), asHashType: test.algorithm)
            //print(mh.asString(base: .base16))
            #expect(mh.asString(base: .base16) == test.multihash)
        }
    }

    // # sha1 - 0x11 - sha1("multihash")
    // 111488c2f11fb2ce392acb5b2986e640211c4690073e # sha1 in hex
    // CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A==== # sha1 in base32
    // 5dsgvJGnvAfiR3K6HCBc4hcokSfmjj # sha1 in base58
    // ERSIwvEfss45KstbKYbmQCEcRpAHPg== # sha1 in base64

    @Test func testSHA1() throws {
        let originalHash = "multihash".data(using: .utf8)!.sha1()

        let multihash = try Multihash(raw: "multihash", hashedWith: .sha1)
        #expect(
            multihash.description.trimmingCharacters(in: .whitespacesAndNewlines)
                == "Multihash: sha1 0x11 20 88c2f11fb2ce392acb5b2986e640211c4690073e"
        )
        #expect(multihash.asString(base: .base16) == "111488c2f11fb2ce392acb5b2986e640211c4690073e")
        #expect(multihash.asString(base: .base32PadUpper) == "CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A====")
        #expect(multihash.asString(base: .base58btc) == "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj")
        #expect(multihash.asString(base: .base64Pad) == "ERSIwvEfss45KstbKYbmQCEcRpAHPg==")

        #expect(multihash.name == "sha1")
        #expect(multihash.code == Int(Codecs.sha1.code))
        #expect(Data(multihash.digest!) == originalHash)
    }

    @Test func testSHA1Tests1() throws {
        let mh = try #require("multihash".data(using: .utf8))
        let originalHash = mh.sha1()
        let sha1 = try encodeMultihashBuffer(Array(originalHash), asHashType: Codecs.sha1)
        //print(sha1.asString(base: .base16))
        //print(sha1.asString(base: .base32))
        //print(sha1.asString(base: .base58btc))
        //print(sha1.asString(base: .base64))
        #expect(sha1.asString(base: .base16) == "111488c2f11fb2ce392acb5b2986e640211c4690073e")
        #expect(sha1.asString(base: .base32PadUpper) == "CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A====")
        #expect(sha1.asString(base: .base58btc) == "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj")
        #expect(sha1.asString(base: .base64Pad) == "ERSIwvEfss45KstbKYbmQCEcRpAHPg==")

        let oh = try decodeMultihashBuffer(sha1)
        #expect(oh.name == "sha1")
        #expect(oh.code == Int(Codecs.sha1.code))
        #expect(Data(oh.digest) == originalHash)
    }

    @Test func testSHA1Tests2() throws {
        let sha = "multihash".data(using: .utf8)!.sha1()
        //print(sha.asString(base: .base16))
        //print(sha.asString(base: .base32))
        //print(sha.asString(base: .base58btc))
        //print(sha.asString(base: .base64))

        let mh = try Multihash(raw: "multihash", hashedWith: .sha1)
        //let originalHash = mh.sha1()
        //let sha1 = try encodeBuf(Array(originalHash), code: SHA1)
        //print(mh.asString(base: .base16))
        //print(mh.asString(base: .base32))
        //print(mh.asString(base: .base58btc))
        //print(mh.asString(base: .base64))
        #expect(mh.asString(base: .base16) == "111488c2f11fb2ce392acb5b2986e640211c4690073e")
        #expect(mh.asString(base: .base32PadUpper) == "CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A====")
        #expect(mh.asString(base: .base58btc) == "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj")
        #expect(mh.asString(base: .base64Pad) == "ERSIwvEfss45KstbKYbmQCEcRpAHPg==")

        //let oh = try decodeBuf(sha1)
        #expect(mh.name == "sha1")
        #expect(mh.code == Int(Codecs.sha1.code))
        #expect(Data(mh.digest!) == sha)

        //'multihash' as a base16 hex string with the multibase 'f' prefix
        let bData = try #require("multihash".data(using: .utf8))
        let b = bData.sha1().asString(base: .base16, withMultibasePrefix: true)
        //print(b)
        let mh2 = try Multihash(multibase: b, codec: .sha1)
        #expect(mh2.asString(base: .base16) == "111488c2f11fb2ce392acb5b2986e640211c4690073e")
        #expect(mh2.asString(base: .base32PadUpper) == "CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A====")
        #expect(mh2.asString(base: .base58btc) == "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj")
        #expect(mh2.asString(base: .base64Pad) == "ERSIwvEfss45KstbKYbmQCEcRpAHPg==")

        //let oh = try decodeBuf(sha1)
        #expect(mh2.name == "sha1")
        #expect(mh2.code == Int(Codecs.sha1.code))
        #expect(Data(mh2.digest!) == sha)

        //<base encoding> <hash type> <hash length> <hash>
        //       f             11           14        ...
        //   hex lower        sha1       20 bytes
        let mh3 = try Multihash(multihash: "f111488c2f11fb2ce392acb5b2986e640211c4690073e")
        #expect(mh3.asString(base: .base32PadUpper) == "CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A====")
        #expect(mh3.asString(base: .base58btc) == "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj")
        #expect(mh3.asString(base: .base64Pad) == "ERSIwvEfss45KstbKYbmQCEcRpAHPg==")

        #expect(mh3.name == "sha1")
        #expect(mh3.code == Int(Codecs.sha1.code))
        #expect(Data(mh3.digest!) == sha)

        let mh4 = try Multihash(multihash: "CCEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A====")
        #expect(mh4.asString(base: .base16) == "111488c2f11fb2ce392acb5b2986e640211c4690073e")
        #expect(mh4.asString(base: .base58btc) == "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj")
        #expect(mh4.asString(base: .base64Pad) == "ERSIwvEfss45KstbKYbmQCEcRpAHPg==")

        #expect(mh4.name == "sha1")
        #expect(mh4.code == Int(Codecs.sha1.code))
        #expect(Data(mh4.digest!) == sha)

    }

    // # sha2-256 0x12 - sha2-256("multihash")
    // 12209cbc07c3f991725836a3aa2a581ca2029198aa420b9d99bc0e131d9f3e2cbe47 # sha2-256 in hex
    // CIQJZPAHYP4ZC4SYG2R2UKSYDSRAFEMYVJBAXHMZXQHBGHM7HYWL4RY= # sha256 in base32
    // QmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk # sha256 in base58
    // EiCcvAfD+ZFyWDajqipYHKICkZiqQgudmbwOEx2fPiy+Rw== # sha256 in base64

    @Test func testSHA2_256() throws {
        let originalHash = "multihash".data(using: .utf8)!.sha256()

        let multihash = try Multihash(raw: "multihash", hashedWith: .sha2_256)
        #expect(
            multihash.asString(base: .base16) == "12209cbc07c3f991725836a3aa2a581ca2029198aa420b9d99bc0e131d9f3e2cbe47"
        )
        #expect(
            multihash.asString(base: .base32PadUpper) == "CIQJZPAHYP4ZC4SYG2R2UKSYDSRAFEMYVJBAXHMZXQHBGHM7HYWL4RY="
        )
        #expect(multihash.asString(base: .base58btc) == "QmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk")
        #expect(multihash.asString(base: .base64Pad) == "EiCcvAfD+ZFyWDajqipYHKICkZiqQgudmbwOEx2fPiy+Rw==")

        #expect(multihash.name == "sha2-256")
        #expect(multihash.code == Int(Codecs.sha2_256.code))
        #expect(Data(multihash.digest!) == originalHash)
    }

    @Test func testSHA2_256Tests1() throws {
        let mh = try #require("multihash".data(using: .utf8))
        let originalHash = mh.sha256()
        let sha256 = try encodeMultihashBuffer(Array(originalHash), code: Int(Codecs.sha2_256.code))
        //print(sha256.asString(base: .base16))
        //print(sha256.asString(base: .base32))
        //print(sha256.asString(base: .base58btc))
        //print(sha256.asString(base: .base64))
        #expect(
            sha256.asString(base: .base16) == "12209cbc07c3f991725836a3aa2a581ca2029198aa420b9d99bc0e131d9f3e2cbe47"
        )
        #expect(
            sha256.asString(base: .base32PadUpper) == "CIQJZPAHYP4ZC4SYG2R2UKSYDSRAFEMYVJBAXHMZXQHBGHM7HYWL4RY="
        )
        #expect(sha256.asString(base: .base58btc) == "QmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk")
        #expect(sha256.asString(base: .base64Pad) == "EiCcvAfD+ZFyWDajqipYHKICkZiqQgudmbwOEx2fPiy+Rw==")

        let oh = try decodeMultihashBuffer(sha256)
        #expect(oh.name == "sha2-256")
        #expect(oh.code == Int(Codecs.sha2_256.code))
        #expect(Data(oh.digest) == originalHash)

    }

    @Test func testSHA2_256Tests2() throws {
        let sha = "multihash".data(using: .utf8)!.sha256()
        //print(sha.asString(base: .base16))
        //print(sha.asString(base: .base32))
        //print(sha.asString(base: .base58btc))
        //print(sha.asString(base: .base64))

        //print()
        let mh = try Multihash(raw: "multihash", hashedWith: .sha2_256, using: .utf8)
        //let originalHash = mh.sha256()
        //let sha256 = try encodeBuf(Array(originalHash), code: SHA2_256)
        //print(mh.asString(base: .base16))
        //print(mh.asString(base: .base32))
        //print(mh.asString(base: .base58btc))
        //print(mh.asString(base: .base64))
        //print(mh.asMultibase(.base16Upper))
        //print(mh.asMultibase(.base32))
        //print(mh.asMultibase(.base58btc))
        //print(mh.asMultibase(.base64))
        #expect(
            mh.asString(base: .base16) == "12209cbc07c3f991725836a3aa2a581ca2029198aa420b9d99bc0e131d9f3e2cbe47"
        )
        #expect(
            mh.asString(base: .base32PadUpper) == "CIQJZPAHYP4ZC4SYG2R2UKSYDSRAFEMYVJBAXHMZXQHBGHM7HYWL4RY="
        )
        #expect(mh.asString(base: .base58btc) == "QmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk")
        #expect(mh.asString(base: .base64Pad) == "EiCcvAfD+ZFyWDajqipYHKICkZiqQgudmbwOEx2fPiy+Rw==")

        //let oh = try decodeBuf(sha256)
        #expect(mh.name == "sha2-256")
        #expect(mh.code == Int(Codecs.sha2_256.code))
        #expect(Data(mh.digest!) == sha)
    }
}
