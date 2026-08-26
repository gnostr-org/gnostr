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

import CID
import Foundation
import LibP2PCrypto
import Multibase
import Multihash
import Testing

@testable import PeerID

/// - Note: Linux Users
/// Make sure to compile the package for release before running these tests.
/// ```swift test -c release -Xswiftc -enable-testing```
/// Otherwise the RSA key generation can take an extremely long time...
@Suite("PeerID Tests")
struct PeerIDTests {

    struct goPeerID {
        static let id = "QmRLoXS3E73psYaUsma1VSbboTa2J8Z9kso1tpiGLk9WQ4"
        static let privKey =
            "CAASpwkwggSjAgEAAoIBAQDWBEbO8kc6a5kEks09CKPQargY3p0DCmCczoCT52/RYFqxvH9dI+s+u4ZAvF9aLWOBvFomL7jHZODPxKDrbiNCmyEbViNgZYK+PNbwh0V3ZGbB27X3q8yZtLvYA8dhcNkz/2SHBarSoC4QLA5MXUuSWtVaYMY3MzMnzBF57Jc9Ase7NvHOIUI90M7aN5izP7hxPXpZ+shiN+TyjM8mFxYONG7ZSsY3IxUhtrU5MRzFX+tp1o/gb/aa51mHf7AL3N02j5ABiYbCK97Rbwr03hsBcwgMxoDPJmP3WZ+D5yyPcOIIF1Vd7+4/f7FQJnIw3xr9/jvaFbPyDCVbBOhr9oyxAgMBAAECggEALlrgx2Q8v0+c5hux7p1XdgYXd/OHyKfPw0cLHH4NfylCm6q7X34vLvhJHO5wLMUV/3y/ffPqLu4Pr5DkVfoWExAsvJIMuY1jIzdkStbR2glaJHUlVc7VUxmNcj1nSxi5QwT3TjORC2v8bi5Mroeqnbmk6p15cW1akC0oP+NZ4rG48+WFHRqsBaBusdSOVfA+IiZUqSd1ILysJ1w7aVN3EC7jLjDG43i+P/2BcEHy8TVClGOknJL341bHe3UPdEpmeu6k6aHGlDI4blUMXahCIUh0IdZuj+Vi/TxQME9+3bKIOjQb8RCNm3U3j/uz5gs9SyTjBuYIib9Scj/jDbLh0QKBgQDfLr3go3Q/AR0jb12QjGALJz1lc9ZRX2RQJkqqmYkZwOlHHyl+YJgqOZiO80fUkN0sJ29CmKecXU4gXuHir913Fdceei1ScBSsvZpWtBLhEZXKrRJYq8U0atKUFQADDMGutyB/uGCNeNwR6VcJezHPICvHxQfmWlWHA5VIOEtRPQKBgQD1fID76SkIpF/EaJMnN2alXWWnzKhUBUPGpQtbpwgSfaCBiZ4vr3NQwKBntOOB5QwHmifNZMoqaFQLzC4B/uyTNUcQMQQ6arYav7WQXqXTmW6poTsjUSuSOPx1swsHlYX09SmUwWDfd94XF9UOU0KUfA2/c85ixzNlV5ejkFA4hQKBgEvP3uQN4hD82d8Nl2TgqkdfnvV1cdnWY4buWvK0kOPUqelk5n1tZoMBaZc1gLLuOpMjGiIvJNByyXUpheWxA7POEXLi4b5dIEjFZ0YIiVk21gEw5UiFoMl7d+ihcY2Xqbslrb507SdhZLAY6V3pITRQo06K2XIgQWlJiE4uATepAoGBALZ2vEiBnYZW5vfN4tKbUyhGq3B1pggNgbr8odyV4mIcDlk6OOGov0WeZ5ut0AyUesSLyFnaOIoc0ZuTP/8rxBwG1bMrO8FP39sx83pDX25P9PkQZixyALjGsp+pXOFeOhtAvo9azO5M4j638Bydtjc3neBX62dwOLtyx7tDYN0hAoGAVLmr3w7XMVHTfEuCSzKHyRrOaN2PAuSX31QAji1PwlwVKMylVrb8rRvBOpTicA/wXPX9Q5O/yjegqhqLT/LXAm9ziFzy5b9/9SzXPukKebXXbvc0FOmcsrcxtijlPyUzf9fKM1ShiwqqsgM9eNyZ9GWUJw2GFATCWW7pl7rtnWk="
    }

    struct samplePeerID {
        static let id = "122019318b6e5e0cf93a2314bf01269a2cc23cd3dcd452d742cdb9379d8646f6e4a9"
        static let privKey =
            "CAASpgkwggSiAgEAAoIBAQC2SKo/HMFZeBml1AF3XijzrxrfQXdJzjePBZAbdxqKR1Mc6juRHXij6HXYPjlAk01BhF1S3Ll4Lwi0cAHhggf457sMg55UWyeGKeUv0ucgvCpBwlR5cQ020i0MgzjPWOLWq1rtvSbNcAi2ZEVn6+Q2EcHo3wUvWRtLeKz+DZSZfw2PEDC+DGPJPl7f8g7zl56YymmmzH9liZLNrzg/qidokUv5u1pdGrcpLuPNeTODk0cqKB+OUbuKj9GShYECCEjaybJDl9276oalL9ghBtSeEv20kugatTvYy590wFlJkkvyl+nPxIH0EEYMKK9XRWlu9XYnoSfboiwcv8M3SlsjAgMBAAECggEAZtju/bcKvKFPz0mkHiaJcpycy9STKphorpCT83srBVQi59CdFU6Mj+aL/xt0kCPMVigJw8P3/YCEJ9J+rS8BsoWE+xWUEsJvtXoT7vzPHaAtM3ci1HZd302Mz1+GgS8Epdx+7F5p80XAFLDUnELzOzKftvWGZmWfSeDnslwVONkL/1VAzwKy7Ce6hk4SxRE7l2NE2OklSHOzCGU1f78ZzVYKSnS5Ag9YrGjOAmTOXDbKNKN/qIorAQ1bovzGoCwx3iGIatQKFOxyVCyO1PsJYT7JO+kZbhBWRRE+L7l+ppPER9bdLFxs1t5CrKc078h+wuUr05S1P1JjXk68pk3+kQKBgQDeK8AR11373Mzib6uzpjGzgNRMzdYNuExWjxyxAzz53NAR7zrPHvXvfIqjDScLJ4NcRO2TddhXAfZoOPVH5k4PJHKLBPKuXZpWlookCAyENY7+Pd55S8r+a+MusrMagYNljb5WbVTgN8cgdpim9lbbIFlpN6SZaVjLQL3J8TWH6wKBgQDSChzItkqWX11CNstJ9zJyUE20I7LrpyBJNgG1gtvz3ZMUQCn3PxxHtQzN9n1P0mSSYs+jBKPuoSyYLt1wwe10/lpgL4rkKWU3/m1Myt0tveJ9WcqHh6tzcAbb/fXpUFT/o4SWDimWkPkuCb+8j//2yiXk0a/T2f36zKMuZvujqQKBgC6B7BAQDG2H2B/ijofp12ejJU36nL98gAZyqOfpLJ+FeMz4TlBDQ+phIMhnHXA5UkdDapQ+zA3SrFk+6yGk9Vw4Hf46B+82SvOrSbmnMa+PYqKYIvUzR4gg34rL/7AhwnbEyD5hXq4dHwMNsIDq+l2elPjwm/U9V0gdAl2+r50HAoGALtsKqMvhv8HucAMBPrLikhXP/8um8mMKFMrzfqZ+otxfHzlhI0L08Bo3jQrb0Z7ByNY6M8epOmbCKADsbWcVre/AAY0ZkuSZK/CaOXNX/AhMKmKJh8qAOPRY02LIJRBCpfS4czEdnfUhYV/TYiFNnKRj57PPYZdTzUsxa/yVTmECgYBr7slQEjb5Onn5mZnGDh+72BxLNdgwBkhO0OCdpdISqk0F0Pxby22DFOKXZEpiyI9XYP1C8wPiJsShGm2yEwBPWXnrrZNWczaVuCbXHrZkWQogBDG3HGXNdU4MAWCyiYlyinIBpPpoAJZSzpGLmWbMWh28+RJS6AQX6KHrK1o2uw=="
        static let pubKey =
            "CAASpgIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQC2SKo/HMFZeBml1AF3XijzrxrfQXdJzjePBZAbdxqKR1Mc6juRHXij6HXYPjlAk01BhF1S3Ll4Lwi0cAHhggf457sMg55UWyeGKeUv0ucgvCpBwlR5cQ020i0MgzjPWOLWq1rtvSbNcAi2ZEVn6+Q2EcHo3wUvWRtLeKz+DZSZfw2PEDC+DGPJPl7f8g7zl56YymmmzH9liZLNrzg/qidokUv5u1pdGrcpLuPNeTODk0cqKB+OUbuKj9GShYECCEjaybJDl9276oalL9ghBtSeEv20kugatTvYy590wFlJkkvyl+nPxIH0EEYMKK9XRWlu9XYnoSfboiwcv8M3SlsjAgMBAAE="
        static let marshaled =
            "0a22122019318b6e5e0cf93a2314bf01269a2cc23cd3dcd452d742cdb9379d8646f6e4a912ab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100b648aa3f1cc1597819a5d401775e28f3af1adf417749ce378f05901b771a8a47531cea3b911d78a3e875d83e3940934d41845d52dcb9782f08b47001e18207f8e7bb0c839e545b278629e52fd2e720bc2a41c25479710d36d22d0c8338cf58e2d6ab5aedbd26cd7008b6644567ebe43611c1e8df052f591b4b78acfe0d94997f0d8f1030be0c63c93e5edff20ef3979e98ca69a6cc7f658992cdaf383faa2768914bf9bb5a5d1ab7292ee3cd79338393472a281f8e51bb8a8fd1928581020848dac9b24397ddbbea86a52fd82106d49e12fdb492e81ab53bd8cb9f74c05949924bf297e9cfc481f410460c28af5745696ef57627a127dba22c1cbfc3374a5b2302030100011aab09080012a609308204a20201000282010100b648aa3f1cc1597819a5d401775e28f3af1adf417749ce378f05901b771a8a47531cea3b911d78a3e875d83e3940934d41845d52dcb9782f08b47001e18207f8e7bb0c839e545b278629e52fd2e720bc2a41c25479710d36d22d0c8338cf58e2d6ab5aedbd26cd7008b6644567ebe43611c1e8df052f591b4b78acfe0d94997f0d8f1030be0c63c93e5edff20ef3979e98ca69a6cc7f658992cdaf383faa2768914bf9bb5a5d1ab7292ee3cd79338393472a281f8e51bb8a8fd1928581020848dac9b24397ddbbea86a52fd82106d49e12fdb492e81ab53bd8cb9f74c05949924bf297e9cfc481f410460c28af5745696ef57627a127dba22c1cbfc3374a5b2302030100010282010066d8eefdb70abca14fcf49a41e2689729c9ccbd4932a9868ae9093f37b2b055422e7d09d154e8c8fe68bff1b749023cc562809c3c3f7fd808427d27ead2f01b28584fb159412c26fb57a13eefccf1da02d337722d4765ddf4d8ccf5f86812f04a5dc7eec5e69f345c014b0d49c42f33b329fb6f58666659f49e0e7b25c1538d90bff5540cf02b2ec27ba864e12c5113b976344d8e9254873b30865357fbf19cd560a4a74b9020f58ac68ce0264ce5c36ca34a37fa88a2b010d5ba2fcc6a02c31de21886ad40a14ec72542c8ed4fb09613ec93be9196e105645113e2fb97ea693c447d6dd2c5c6cd6de42aca734efc87ec2e52bd394b53f52635e4ebca64dfe9102818100de2bc011d75dfbdccce26fabb3a631b380d44ccdd60db84c568f1cb1033cf9dcd011ef3acf1ef5ef7c8aa30d270b27835c44ed9375d85701f66838f547e64e0f24728b04f2ae5d9a56968a24080c84358efe3dde794bcafe6be32eb2b31a8183658dbe566d54e037c7207698a6f656db20596937a4996958cb40bdc9f13587eb02818100d20a1cc8b64a965f5d4236cb49f73272504db423b2eba720493601b582dbf3dd93144029f73f1c47b50ccdf67d4fd2649262cfa304a3eea12c982edd70c1ed74fe5a602f8ae4296537fe6d4ccadd2dbde27d59ca8787ab737006dbfdf5e95054ffa384960e299690f92e09bfbc8ffff6ca25e4d1afd3d9fdfacca32e66fba3a90281802e81ec10100c6d87d81fe28e87e9d767a3254dfa9cbf7c800672a8e7e92c9f8578ccf84e504343ea6120c8671d70395247436a943ecc0dd2ac593eeb21a4f55c381dfe3a07ef364af3ab49b9a731af8f62a29822f533478820df8acbffb021c276c4c83e615eae1d1f030db080eafa5d9e94f8f09bf53d57481d025dbeaf9d070281802edb0aa8cbe1bfc1ee7003013eb2e29215cfffcba6f2630a14caf37ea67ea2dc5f1f39612342f4f01a378d0adbd19ec1c8d63a33c7a93a66c22800ec6d6715adefc0018d1992e4992bf09a397357fc084c2a628987ca8038f458d362c8251042a5f4b873311d9df521615fd362214d9ca463e7b3cf619753cd4b316bfc954e610281806beec9501236f93a79f99999c60e1fbbd81c4b35d83006484ed0e09da5d212aa4d05d0fc5bcb6d8314e297644a62c88f5760fd42f303e226c4a11a6db213004f5979ebad9356733695b826d71eb664590a200431b71c65cd754e0c0160b28989728a7201a4fa68009652ce918b9966cc5a1dbcf91252e80417e8a1eb2b5a36bb"
    }

    static let testIdHex = samplePeerID.id
    nonisolated(unsafe) static let testIdBytes = try! Multihash(hexString: testIdHex)
    static let testIdB58String = testIdBytes.asString(base: .base58btc)
    nonisolated(unsafe) static let testIdCID = try! CID(version: .v1, codec: .libp2p_key, multihash: testIdBytes)
    static let testIdCIDString = try! testIdCID.toBaseEncodedString(.base32)

    /// Generate a new PeerID with default params (RSA 2048)
    @Test func testGeneratePeerID_Default_Params() throws {
        let peerID = try PeerID()
        print(peerID.debugDescription)
        #expect(peerID.b58String.count == 46)  //Example: QmYijSS4Tcz2i4LQbqJ7wkow1cy7sKqMbxChA5YJkav7VS
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 2048)
    }

    /// Creates a new PeerID with an underlying RSA 1024 bit key pair
    @Test func testGeneratePeerID_RSA_1024() throws {
        let peerID = try PeerID(.RSA(bits: .B1024))
        print(peerID.debugDescription)
        #expect(peerID.b58String.count == 46)  //Example: QmYijSS4Tcz2i4LQbqJ7wkow1cy7sKqMbxChA5YJkav7VS
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 1024)
    }

    /// Creates a new PeerID with an underlying RSA 2048 bit key pair
    @Test func testGeneratePeerID_RSA_2048() throws {
        let peerID = try PeerID(.RSA(bits: .B2048))
        print(peerID.debugDescription)
        #expect(peerID.b58String.count == 46)  //Example: QmYijSS4Tcz2i4LQbqJ7wkow1cy7sKqMbxChA5YJkav7VS
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 2048)
    }

    /// Creates a new PeerID with an underlying RSA 3072 bit key pair
    @Test(.canImportSecurity)
    func testGeneratePeerID_RSA_3072() throws {
        let peerID = try PeerID(.RSA(bits: .B3072))
        print(peerID.debugDescription)
        #expect(peerID.b58String.count == 46)  //Example: QmYijSS4Tcz2i4LQbqJ7wkow1cy7sKqMbxChA5YJkav7VS
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 3072)
    }

    /// Creates a new PeerID with an underlying RSA 4096 bit key pair
    @Test(.canImportSecurity)
    func testGeneratePeerID_RSA_4096() throws {
        let peerID = try PeerID(.RSA(bits: .B4096))
        print(peerID.debugDescription)
        #expect(peerID.b58String.count == 46)  //Example: QmYijSS4Tcz2i4LQbqJ7wkow1cy7sKqMbxChA5YJkav7VS
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 4096)
    }

    /// Creates a new PeerID with an underlying Secp256k1 key pair
    ///
    /// libp2p-js/peerID equivalent
    /// ```
    /// it('can be created for a Secp256k1 key', async () => {
    ///   const id = await PeerId.create({ keyType: 'secp256k1', bits: 256 })
    ///   const expB58 = mh.toB58String(mh.encode(id.pubKey.bytes, 'identity'))
    ///   expect(id.toB58String()).to.equal(expB58)
    /// })
    /// ```
    @Test func testGenerate_Secp256k1_PeerID() throws {
        let peerID = try PeerID(.Secp256k1)
        print(peerID.debugDescription)
        //let expB58 = try Multihash(raw: peerID.publicKey, hashedWith: .identity)
        //XCTAssertEqual(peerID.b58String, expB58.asString(base: .base58btc))
        #expect(peerID.b58String.count == 46)
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .secp256k1)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 64)
    }

    /// Creates a new PeerID with an underlying Ed25519 key pair
    @Test func testGenerate_Ed25519_PeerID() throws {
        let peerID = try PeerID(.Ed25519)
        print(peerID.debugDescription)
        //let expB58 = try Multihash(raw: peerID.keyPair!.publicKey.data, hashedWith: .identity)
        //XCTAssertEqual(peerID.b58String, expB58.asString(base: .base58btc))
        #expect(peerID.b58String.count == 52)
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.keyType == .ed25519)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.attributes()?.size == 32)
    }

    @Test func testFromHexString() throws {
        let peerID = try PeerID(fromHexID: PeerIDTests.testIdHex)
        print(peerID.id.asString(base: .base16))
        print(peerID.hexString)
        print(peerID)
        #expect(peerID.hexString == PeerIDTests.testIdHex)
        #expect(peerID.id.asString(base: .base16) == PeerIDTests.testIdHex)
        #expect(peerID.keyPair == nil)
    }

    @Test func testFromBytes() throws {
        let peerID = try PeerID(fromBytesID: PeerIDTests.testIdBytes.value)
        print(peerID)
        #expect(peerID.hexString == PeerIDTests.testIdHex)
        #expect(peerID.id.asString(base: .base16) == PeerIDTests.testIdHex)
        #expect(peerID.keyPair == nil)
    }

    @Test func testCIDRoundTripRSA() throws {
        let peerID = try PeerID(.RSA(bits: .B1024))

        let cid = peerID.cidString

        let peerID2 = try PeerID(cid: cid)

        #expect(peerID.id == peerID2.id)
    }

    @Test func testCIDRoundTripEd25519() throws {
        let peerID = try PeerID(.Ed25519)

        let cid = peerID.cidString

        let peerID2 = try PeerID(cid: cid)

        #expect(peerID.id == peerID2.id)
    }

    @Test func testCIDRoundTripSecp256k1() throws {
        let peerID = try PeerID(.Secp256k1)

        let cid = peerID.cidString

        let peerID2 = try PeerID(cid: cid)

        #expect(peerID.id == peerID2.id)
    }

    @Test func testHexDecoding() throws {
        /// - FIXME: Decoding base16 (hex) is super slow using the Multibase library
        let hex1 = try BaseEncoding.decode(PeerIDTests.samplePeerID.marshaled, as: .base16).data
        let hex2 = Data(hex: PeerIDTests.samplePeerID.marshaled)

        #expect(hex1 == hex2)
    }

    @Test func testEmbeddedEd25519PublicKeys() throws {
        // [Embedded:Traditional]
        let ed25519EmbeddedB58IDs = [
            "12D3KooWAfPDpPRRRBrmqy9is2zjU5srQ4hKuZitiGmh4NTTpS2d": "QmPoHmYtUt8BU9eiwMYdBfT6rooBnna5fdAZHUaZASGQY8",
            "12D3KooWF5Qbrbvhhha1AcqRULWAfYzFEnKvWVGBUjw489hpo5La": "Qmbp3SxL2SYcH6Ly4r5SGQwfxkDCJPuhJG35GCZimcTiBc",
            "12D3KooWBHBzTtotosqDcDYuTjLoReWsynXiM547f6JYKL9JSjjY": "QmbvwwFmC6gskHBPyqcrAp9cAiWu38Pn22gMkRXHmnRWLo",
            "12D3KooWJEfH2MB4RsUoaJPogDPRWbFTi8iehsxsqrQpiJwFNDrP": "QmSLYHye2CMeg3eTDLFk5k2b1QiCq8KYcL3qvHkFK3pUrQ",
            "12D3KooWNPQXNm9CJirv9uUKKZgWrTFYGuxRmXFzUjj3VXsoFr2H": "QmXa6eSg5waAp6FjHVaoeZmKPYANK8aRmyvzMRTqLKkbNv",
        ]

        // Ensure that we can extract the ED25519 Public Key when it's embedded in the PeerID
        for id in ed25519EmbeddedB58IDs {
            let embeddedKeyInBytes = try BaseEncoding.decode(id.0, as: .base58btc)

            let peerID = try PeerID(fromBytesID: embeddedKeyInBytes.data.byteArray)

            #expect(peerID.b58String == id.0)
            #expect(peerID.type == .isPublic)
            #expect(peerID.keyPair?.publicKey != nil)
            #expect(peerID.keyPair?.keyType == .ed25519)
        }

        // Ensure we can instantiate the KeyPair via the CID string initializer as well
        for id in ed25519EmbeddedB58IDs {
            let peerID = try PeerID(cid: id.0)

            #expect(peerID.b58String == id.0)
            #expect(peerID.type == .isPublic)
            #expect(peerID.keyPair?.publicKey != nil)
            #expect(peerID.keyPair?.keyType == .ed25519)
        }

        // Ensure we can instantiate a PeerID (id only) with the traditional b58 Multihash ED25519 bytes id
        for id in ed25519EmbeddedB58IDs {
            let edBytes = try BaseEncoding.decode(id.1, as: .base58btc)

            let peerID = try PeerID(fromBytesID: edBytes.data.byteArray)

            #expect(peerID.b58String == id.1)
            #expect(peerID.type == .idOnly)
            #expect(peerID.keyPair == nil)
        }

        // Ensure we can instantiate the KeyPair via the CID string initializer as well
        for id in ed25519EmbeddedB58IDs {
            let peerID = try PeerID(cid: id.1)

            #expect(peerID.b58String == id.1)
            #expect(peerID.type == .idOnly)
            #expect(peerID.keyPair == nil)
        }

        // Ensure we can compare them appropriately
        for id in ed25519EmbeddedB58IDs {
            let pid1 = try PeerID(cid: id.0)
            let pid2 = try PeerID(cid: id.1)

            #expect(pid1.isEquivalent(to: pid2))
            #expect(pid1 == pid2)
        }

        // Ensure we can force the traditional B58 String if necessary
        for id in ed25519EmbeddedB58IDs {
            let pid1 = try PeerID(cid: id.0)

            #expect(try pid1.traditionalB58String() == id.1)
        }
    }

    @Test func testEmbeddedEd25519PublicKeysPreference() throws {
        let pid = try PeerID(.Ed25519)
        // Ensure we default to embedding the pubkey in our PeerIDs b58 string
        #expect(pid.multihash.algorithm == .identity)
        #expect(pid.b58String.hasPrefix("12D3KooW"))
    }

    @Test func testRecoverPublicKeyFromPeerIDWithEmbeddedED25519Keys() throws {
        // RSA doesn't support embedding pub keys in their ID's
        let rsa = try PeerID(.RSA(bits: .B1024))
        let rsaID = rsa.id

        // So when we instantiate a PeerID from it's ID, it's public key should be nil
        let recoveredPeerID = try PeerID(fromBytesID: rsaID)
        #expect(recoveredPeerID.b58String == rsa.b58String)
        #expect(recoveredPeerID.type == .idOnly)
        #expect(recoveredPeerID.keyPair?.publicKey == nil)

        // ED25519 supports embedding it's public key in it's ID
        let ed25519 = try PeerID(.Ed25519)
        let edID = ed25519.id

        // So when we instantiate a PeerID from it's ID, it's public key should be populated
        let recoveredED = try PeerID(fromBytesID: edID)
        #expect(recoveredED.b58String == ed25519.b58String)
        #expect(recoveredED.type == .isPublic)
        #expect(recoveredED.keyPair?.publicKey != nil)
        #expect(recoveredED.keyPair?.keyType == .ed25519)
        #expect(recoveredED.keyPair?.publicKey.data == ed25519.keyPair?.publicKey.data)
    }

    @Test func testFromMarshaledStringSample() throws {
        let peerID = try PeerID(marshaledPeerID: PeerIDTests.samplePeerID.marshaled, base: .base16)
        //let peerID = try PeerID(marshaledPeerID: Data(hex: PeerIDTests.samplePeerID.marshaled))
        print(peerID)
        #expect(peerID.hexString == PeerIDTests.samplePeerID.id)
    }

    @Test func testFromMarshaledPublicKey() throws {
        //try Multihash(hexString: "f\(PeerIDTests.samplePeerID.marshaled)").value
        let marshaledPeerIDData = Data(hex: PeerIDTests.samplePeerID.marshaled)
        let protoPeerID = try PeerIdProto(serializedBytes: marshaledPeerIDData)

        print("ID: \(protoPeerID.id.asString(base: .base16))")
        print("pubKey: \(protoPeerID.pubKey.asString(base: .base64Pad))")
        print("privKey: \(protoPeerID.privKey.asString(base: .base64Pad))")

        #expect(protoPeerID.id.asString(base: .base16) == PeerIDTests.samplePeerID.id)
        #expect(protoPeerID.pubKey.asString(base: .base64Pad) == PeerIDTests.samplePeerID.pubKey)
        #expect(protoPeerID.privKey.asString(base: .base64Pad) == PeerIDTests.samplePeerID.privKey)

        guard protoPeerID.hasPubKey else {
            Issue.record("No Pub Key Found")
            return
        }

        let peerID = try PeerID(marshaledPublicKey: protoPeerID.pubKey)

        print(peerID)

        print("Multihashing Proto Pub Key")
        let id = try Multihash(raw: protoPeerID.pubKey, hashedWith: .sha2_256)
        print(id.hexString)
        #expect(id.hexString == PeerIDTests.testIdHex)
        print("--------------------------")

        let pid = peerID.id.asString(base: .base16)
        print(pid)
        #expect(pid == PeerIDTests.testIdHex)
    }

    @Test func testFromMarshaledPrivateKey() throws {
        //try Multihash(hexString: "f\(PeerIDTests.samplePeerID.marshaled)").value
        let marshaledPeerIDData = Data(hex: PeerIDTests.samplePeerID.marshaled)
        let protoPeerID = try PeerIdProto(serializedBytes: marshaledPeerIDData)

        #expect(protoPeerID.id.asString(base: .base16) == PeerIDTests.samplePeerID.id)
        #expect(protoPeerID.pubKey.asString(base: .base64Pad) == PeerIDTests.samplePeerID.pubKey)
        #expect(protoPeerID.privKey.asString(base: .base64Pad) == PeerIDTests.samplePeerID.privKey)

        guard protoPeerID.hasPrivKey else {
            Issue.record("No Private Key Found")
            return
        }

        let peerID = try PeerID(marshaledPrivateKey: protoPeerID.privKey)

        print(peerID)

        print("Multihashing Proto Pub Key")
        let id = try Multihash(raw: protoPeerID.pubKey, hashedWith: .sha2_256)
        print(id.hexString)
        #expect(id.hexString == PeerIDTests.testIdHex)
        print("--------------------------")

        let pid = peerID.id.asString(base: .base16)
        print(pid)
        #expect(pid == PeerIDTests.testIdHex)
    }

    /// Decodes a base64 encoded string
    /// Unmarshales a PrivateKey
    /// Imports a SecKey from the raw data
    /// Extracts/derives a Public Key from the Private Key
    @Test func testFromMarshaledPrivateKey_GO() throws {
        let marshaledPrivateKey = try BaseEncoding.decode(PeerIDTests.goPeerID.privKey, as: .base64Pad)

        let peerID = try PeerID(marshaledPrivateKey: marshaledPrivateKey.data)

        print(peerID)

        let pid = peerID.id.asString(base: .base58btc)
        print(pid)
        #expect(pid == PeerIDTests.goPeerID.id)
    }

    /// Marshaling Private RSA Keys
    @Test func testFromMarshaledPrivateKey_GO_2() throws {
        let peerID = try PeerID(marshaledPrivateKey: PeerIDTests.goPeerID.privKey, base: .base64Pad)

        #expect(peerID.b58String == PeerIDTests.goPeerID.id)

        guard let marshaledPrivKey = try peerID.keyPair?.privateKey?.marshal() else {
            Issue.record("Failed to import")
            return
        }

        print(marshaledPrivKey.asString(base: .base64Pad))

        #expect(marshaledPrivKey.asString(base: .base64Pad) == PeerIDTests.goPeerID.privKey)
    }

    @Test func testToJSONPublic() throws {
        let peerID = try PeerID(marshaledPeerID: PeerIDTests.samplePeerID.marshaled, base: .base16)

        let publicJSON = try peerID.toJSON(includingPrivateKey: false)

        //print("Full JSON")
        //print(String(data: fullJSON, encoding: .utf8))
        //print()
        //print("Public JSON")
        //print(String(data: publicJSON, encoding: .utf8))

        let pubID = try PeerID(fromJSON: publicJSON)
        // Ensure the ID matches the test fixture
        #expect(pubID.hexString == PeerIDTests.samplePeerID.id)
        // Ensure our Public Key was instantiated
        #expect(pubID.keyPair?.publicKey != nil)
        // Ensure that the Private Key did not get exported
        #expect(pubID.keyPair?.privateKey == nil)
    }

    @Test func testToJSONFull() throws {
        let peerID = try PeerID(marshaledPeerID: PeerIDTests.samplePeerID.marshaled, base: .base16)

        let fullJSON = try peerID.toJSON(includingPrivateKey: true)
        let publicJSON = try peerID.toJSON(includingPrivateKey: false)

        //print("Full JSON")
        //print(String(data: fullJSON, encoding: .utf8))
        //print()
        //print("Public JSON")
        //print(String(data: publicJSON, encoding: .utf8))

        let pubID = try PeerID(fromJSON: publicJSON)
        // Ensure the ID matches the test fixture
        #expect(pubID.hexString == PeerIDTests.samplePeerID.id)
        // Ensure our Public Key was instantiated
        #expect(pubID.keyPair?.publicKey != nil)
        // Ensure that the Private Key did not get exported
        #expect(pubID.keyPair?.privateKey == nil)

        let fullID = try PeerID(fromJSON: fullJSON)
        // Ensure the ID matches the test fixture
        #expect(fullID.hexString == PeerIDTests.samplePeerID.id)
        // Ensure that we derived the Public Key
        #expect(fullID.keyPair?.publicKey != nil)
        // Ensure that the Private Key was instantiated
        #expect(fullID.keyPair?.privateKey != nil)

        // PeerID <-> PeerID Equality
        #expect(pubID == fullID)
        // [UInt8] <-> PeerID Equality
        #expect(pubID.id == fullID)
        // Data <-> PeerID Equality
        #expect(Data(pubID.id) == fullID)

        #expect(pubID.id == fullID.id)
        #expect(
            pubID.keyPair?.publicKey.asString(base: .base64) == fullID.keyPair?.publicKey.asString(base: .base64)
        )
        #expect(
            pubID.keyPair?.privateKey?.asString(base: .base64) != fullID.keyPair?.privateKey?.asString(base: .base64)
        )
    }

    @Test func testImportExportEncryptedPEM() throws {
        /// The encrypted version of an RSA 1024 Private Key
        ///
        /// Encrypted with
        /// ```
        /// openssl pkcs8
        ///     -in foo.pem
        ///     -topk8
        ///     -v2 aes-128-cbc
        ///     -passout pass:mypassword
        /// ```
        let ENCRYPTED = """
            -----BEGIN ENCRYPTED PRIVATE KEY-----
            MIICzzBJBgkqhkiG9w0BBQ0wPDAbBgkqhkiG9w0BBQwwDgQI49PtP+7yJmgCAggA
            MB0GCWCGSAFlAwQBAgQQYz/oWtq4qhWPNrAQiO3i5wSCAoCjWvOSqAMdA4qDF8BB
            aaqGRnZ/Lvewsrs4keppFogFnYpeVkzEmeleQLIYkO2mnNvsjhfh2Vk1LW/qNPIl
            NvwjXyNbP1E6TlLmTNEAgIfyViHOCuk+17tkgAtK98huFTi0U+LbMcaxSnJ7CsNY
            9JODko7fLXMpEaGy5qcuXWsMHG1iKcggYs0J1kmWSVw9ZQP7Uh9hs31zz60kFe+T
            1I8EOjC06EcKY2HmOhzS+p378nWD3Lxi49FWkHslx1OtQwAXqMG5xWSo+kTWgmUx
            fB3Olmv7opDcQ5OtOSxRjM/6SCtrtIlPRjIS7Uu4foW2BpFS+mkkvaJR0lMiEFjA
            qMdLu3MZzT8U9lEDpd+ki+OjIC2bOXkv/OgHFmHjrTrGTVnK+HP5B0XkcaN0kmi5
            ypd8/XB4zDqO/eSSTKnDe5cvw9Ruj5vt9cesUGjckTlVlZ7Sip2nqtngEAh0k7gc
            p8p0LpNRyOM5edxNCsRLWj3Z9oskkbEFbL3INuVr6HZ5C1IpUHaxzdii1FBeLSqY
            RYCC7iOgfqRILkBN2dsnWhdLLvcVpeQqSccnNCYSrXgr40T8BqZKLnuhHT7/iZaw
            OiKp9MyygPf0wO5IFaSglpk02dohJpg/LYxFBZk+qJKPR9883NrtSPSzXxDogu2f
            /tc8OCoH919cB8WAsU1cvKYMxsr9HTfoxS7itrJX9d7tE3J2Ky7fQrPWt247BXSE
            FMUJ8BQpLL/2lNIxW9clLEuzr0RZKu3AhBU0V0o8KDucrsLPdbLvV9/J8+G8VJWB
            DZjkXrHO2Oob0rOBtz0gnIF4TSwMWlI28OFWLwN3ByGeT0KcDN7SghLtDSyEQKNW
            ZHiA
            -----END ENCRYPTED PRIVATE KEY-----
            """

        let peerID = try PeerID(pem: ENCRYPTED, password: "mypassword")
        #expect(peerID.keyPair != nil)
        #expect(peerID.keyPair?.publicKey != nil)
        #expect(peerID.keyPair?.privateKey != nil)
        #expect(peerID.type == .isPrivate)
        #expect(peerID.keyPair?.keyType == .rsa)

        /// Every time we export the encrypted PEM it should be unique (unless you manually set the IV using swift-libp2p-crypto)
        let export1 = try peerID.exportKeyPair(as: .privatePEMString(encryptedWithPassword: "mypassword"))
        let export2 = try peerID.exportKeyPair(as: .privatePEMString(encryptedWithPassword: "mypassword"))

        #expect(export1 != ENCRYPTED)
        #expect(export2 != ENCRYPTED)
        #expect(export1 != export2)
        #expect(export1.count == export2.count)
        #expect(export1.count == ENCRYPTED.count)

        /// Ensure we can't decrypt using the wrong passwords
        #expect(throws: Error.self) { try PeerID(pem: ENCRYPTED, password: "") }
        #expect(throws: Error.self) { try PeerID(pem: ENCRYPTED, password: "MyPassword") }
        #expect(throws: Error.self) { try PeerID(pem: ENCRYPTED, password: nil) }
    }

    @Test func testImportExportED25519PeerID() throws {
        let peerID = try PeerID(.Ed25519)
        let export = try peerID.exportKeyPair(as: .privatePEMString(encryptedWithPassword: "mypassword"))

        print(export)

        let imported = try PeerID(pem: export, password: "mypassword")

        #expect(imported.keyPair != nil)
        #expect(imported.keyPair?.privateKey != nil)
        #expect(imported.keyPair?.privateKey?.rawRepresentation == peerID.keyPair?.privateKey?.rawRepresentation)
        #expect(imported == peerID)

        /// Ensure the wrong passwords throw errors
        #expect(throws: Error.self) { try PeerID(pem: export, password: "") }
        #expect(throws: Error.self) { try PeerID(pem: export, password: "MyPassword") }
        #expect(throws: Error.self) { try PeerID(pem: export, password: nil) }
    }

    @Test func testImportExportSecp256k1PeerID() throws {
        let peerID = try PeerID(.Secp256k1)
        let export = try peerID.exportKeyPair(as: .privatePEMString(encryptedWithPassword: "mypassword"))

        print(export)

        let imported = try PeerID(pem: export, password: "mypassword")

        #expect(imported.keyPair != nil)
        #expect(imported.keyPair?.privateKey != nil)
        #expect(imported.keyPair?.privateKey?.rawRepresentation == peerID.keyPair?.privateKey?.rawRepresentation)
        #expect(imported == peerID)

        /// Ensure the wrong passwords throw errors
        #expect(throws: Error.self) { try PeerID(pem: export, password: "") }
        #expect(throws: Error.self) { try PeerID(pem: export, password: "MyPassword") }
        #expect(throws: Error.self) { try PeerID(pem: export, password: nil) }
    }
}

struct TestHelper {
    static var isLinux: Bool {
        #if os(Linux)
        return true
        #else
        return false
        #endif
    }

    static var canImportSecurity: Bool {
        #if canImport(Security)
        return true
        #else
        return false
        #endif
    }
}

extension Trait where Self == ConditionTrait {
    /// This test only runs on Linux systems
    public static var requiresLinux: Self {
        enabled(
            if: TestHelper.isLinux,
            "This test is only available for linux systems"
        )
    }

    /// A trait to determine if the Security module can be imported
    public static var canImportSecurity: Self {
        enabled(
            if: TestHelper.canImportSecurity,
            "This test is only available when we can import the Security module"
        )
    }
}
