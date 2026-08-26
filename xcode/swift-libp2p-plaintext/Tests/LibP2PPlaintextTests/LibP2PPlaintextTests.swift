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

import LibP2P
import Testing

@testable import LibP2PPlaintext

@Suite("Libp2p Plaintext Tests")
struct LibP2PPlaintextTests {

    /// Our Marshaled PeerID can be decoded as an Exchange Protobuf or directly into a PeerID, same for GO
    /// JS has some extra bytes in their Exchange Protobuf that throws off the PeerID(marshaled: ) initialization.
    @Test func testExchangeProtobufDecodingSwift() throws {
        let hexMsg =
            "0a22122093b3c869bb577e6845ccb6fd26f4b5ad345eb7d1cda27f1bcf7f9ed92e19c36e12ab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100bd2a4fb3cf4784db8f15ef53e77ba83da234fbc80e3fc6c8212a1bea8310b5524db4495336d509c52c0af5e21fee94ed6618e3fd4d5c88d586f181701d95611bfeaf0d97c22585ce7a7f20be10556f4d849544574d4b7817f2874da9a3f2df3469c11a435f1b23435d0b2379d5273262d9748f7acead319c2c2dc7a1d654f7d0158728e45770bc4582af079a31cc4d6b2ac2575a97d27d3c49fc7ad5aa36bb31a1f54ff007333ebb9f63eb51d5a6097b9b59b953663ae7ba9a664f7fc8d6e32ff4ea31c20fc818e8235c309a780c128de10c90207b7921629b5c64a0e29bac56d4f02ad4b1e58a9546554e808732a45c5edb0b4ff554537b8623cbd4670103c70203010001"

        let data = [UInt8](hex: hexMsg)

        let exchangeMsg = try Exchange(serializedBytes: data)

        #expect(exchangeMsg.hasPubkey)
        #expect(exchangeMsg.pubkey.type == .rsa)

        let peerID = try PeerID(marshaledPeerID: Data(data))

        #expect(peerID.type == .isPublic)
        #expect(peerID.keyPair?.keyType == .rsa)
    }

    /// The Pubkey exchange that Go sends us is a Marshaled PeerID
    /// - Note: This succeeds (shows that GO's marshaled public key can be decoded into an Exchange Protobuf or directly into a PeerID)
    @Test func testExchangeProtobufDecodingGO() throws {
        let hexMsg =
            "0a22122061e066c40af81bf4f6f8712f4bf444b5fdef44588859297b04465ce0f6c748b912ab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100b14f3a4c567128172768c95130582d0f20a29f65311a4b7ab26ab2eda870e37b7fd2e3032981f9b874ab1ffb89c6257b5cbb83ad7fcfc66b7eda26e664ffee9ef4107742807ecdd983d8486460aaca130eb1818d070d241f400ed6b21e1f88b88ce28eb1e75d22d8b0d656bd5b5abf0873f0864bbb5080af894cbc022322ef56a6479f65fecf1036debca6fa9b98d4e820e49e3cf0294027754b934d1112bc45bca6be5c95f7903ed1fd58a839b3484aaa8f3ac3047241d3b707bcb5df3da6b20f599d97280fa8e8c34f524490d0453077eb501fa56c12637ef5cd122c0c86f4a2896f0247760ba02681afe8ee3ad3cc0b180a67d97f40a6f0aba46f2d2b2da70203010001"

        let data = [UInt8](hex: hexMsg)

        let exchangeMsg = try Exchange(serializedBytes: data)

        #expect(exchangeMsg.hasPubkey)
        #expect(exchangeMsg.pubkey.type == .rsa)

        let peerID = try PeerID(marshaledPeerID: Data(data))

        #expect(peerID.type == .isPublic)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.b58String == "QmUviibYYK9PuVJqxqiTuSuZPgUPur5Ncy1YEvKmj3osr8")
    }

    /// The Pubkey that JS sends us is an Exchange Protobuf
    /// - Note: This fails (shows that JS's marshaled public key can be decoded into an Exchange Protobuf, but not directly into a PeerID)
    @Test func testExchangeProtobufDecodingJS() throws {
        let hexMsg =
            "0a221220d7a3092350d15eb2885ec0946a7ab5d55f23b743783df66a36a691b2df7704be12b002080012ab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100cb65970671b7b8bab30156570780dd1fa41ca663ec5caf5e1ec2f6a61bbd1064e0c9d61f6c963944c59edbc8b8aa441dd28c66dfa59d45c1ece7c0dc56b56a631e44234d55dea0ce041200d234efc283b28d954d0426f80f2577843f2700657f6a0126d527a104090e34551ffbf92b92aac74bd61ae61b5a61a488eef83eb880bc96a70f561002dfccca47e48fcabdd6035b737883acce0f6999369e72d601de5c83475b3305e7de8eef6ffa731c32897575789d34d24d4d567532acf8d7b6a59d54b464e485352c5a684285d95395f584a8f56b6aadfe565f1f978eac5feb6a61dcc21ad8630d6dc5ff091d758efdb6cd9f27a64c702fbad6694cdaec5804df0203010001"

        let data = [UInt8](hex: hexMsg)

        let exchangeMsg = try Exchange(serializedBytes: data)

        #expect(exchangeMsg.hasPubkey)
        #expect(exchangeMsg.pubkey.type == .rsa)

        /// JS sends a marshalled PublicKey instead of a marshalled PeerID
        let peerID = try PeerID(marshaledPublicKey: exchangeMsg.pubkey.data)
        #expect(peerID.type == .isPublic)
        #expect(peerID.keyPair?.keyType == .rsa)
        #expect(peerID.b58String == "QmcrQZ6RJdpYuGvZqD5QEHAv6qX4BrQLJLQPQUrTrzdcgm")
    }

    @Test func testExchangeProtobufEncodingJSEchoExampleDialersKey() throws {
        let expectedHexDialer =
            "0a221220add8ac59b1fe19f20dfaa2228c239e3c131d73e0a7848ac869d5eb959a27ec6c12b002080012ab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a02820101009a3520ce8cfcfa4fc1d9b1fecb0e9c6241188dd8e8dec881d44b4e69f1058eaf710550216c0b7d51e26a22844e4737e17135a954cb215953fff28dfd6976794c26aad507225231afb2db2e31d85b9ca680803bded3c7e896cf0959d945c451733563cd6684f6de597cbec0fdb11254e02044744ec9ffb61a00d120f6bbdc09b95bccedd07b701707626a95e891fe29609e7514ee9ba3b506cb2a3ffe0b6e6dbeae4adb678fa8551a14d8344ba0584aab0a8bb7296b6ee8f85ce2375f290c5d5e7eb905f3d49cee6cd381f65b1ce8af9e442fb0f218610ee14c833919e2aa260a7c77ba13baeca68809df32aaa05ae8f27ff9f04ce57938863c91e346f071fe350203010001"

        let jsDialer = """
            {
              "id": "Qma3GsJmB47xYuyahPZPSadh1avvxfyYQwk8R3UnFrQ6aP",
              "privKey": "CAASpwkwggSjAgEAAoIBAQCaNSDOjPz6T8HZsf7LDpxiQRiN2OjeyIHUS05p8QWOr3EFUCFsC31R4moihE5HN+FxNalUyyFZU//yjf1pdnlMJqrVByJSMa+y2y4x2FucpoCAO97Tx+iWzwlZ2UXEUXM1Y81mhPbeWXy+wP2xElTgIER0Tsn/thoA0SD2u9wJuVvM7dB7cBcHYmqV6JH+KWCedRTum6O1BssqP/4Lbm2+rkrbZ4+oVRoU2DRLoFhKqwqLtylrbuj4XOI3XykMXV5+uQXz1JzubNOB9lsc6K+eRC+w8hhhDuFMgzkZ4qomCnx3uhO67KaICd8yqqBa6PJ/+fBM5Xk4hjyR40bwcf41AgMBAAECggEAZnrCJ6IYiLyyRdr9SbKXCNDb4YByGYPEi/HT1aHgIJfFE1PSMjxcdytxfyjP4JJpVtPjiT9JFVU2ddoYu5qJN6tGwjVwgJEWg1UXmPaAw1T/drjS94kVsAs82qICtFmwp52Apg3dBZ0Qwq/8qE1XbG7lLyohIbfCBiL0tiPYMfkcsN9gnFT/kFCX0LVs2pa9fHCRMY9rqCc4/rWJa1w8sMuQ23y4lDaxKF9OZVvOHFQkbBDrkquWHE4r55fchCz/rJklkPJUNENuncBRu0/2X+p4IKFD1DnttXNwb8j4LPiSlLro1T0hiUr5gO2QmdYwXFF63Q3mjQy0+5I4eNbjjQKBgQDZvZy3gUKS/nQNkYfq9za80uLbIj/cWbO+ZZjXCsj0fNIcQFJcKMBoA7DjJvu2S/lf86/41YHkPdmrLAEQAkJ+5BBNOycjYK9minTEjIMMmZDTXXugZ62wnU6F46uLkgEChTqEP57Y6xwwV+JaEDFEsW5N1eE9lEVX9nGIr4phMwKBgQC1TazLuEt1WBx/iUT83ita7obXqoKNzwsS/MWfY2innzYZKDOqeSYZzLtt9uTtp4X4uLyPbYs0qFYhXLsUYMoGHNN8+NdjoyxCjQRJRBkMtaNR0lc5lVDWl3bTuJovjFCgAr9uqJrmI5OHcCIk/cDpdWb3nWaMihVlePmiTcTy9wKBgQCU0u7c1jKkudqks4XM6a+2HAYGdUBk4cLjLhnrUWnNAcuyl5wzdX8dGPi8KZb+IKuQE8WBNJ2VXVj7kBYh1QmSJVunDflQSvNYCOaKuOeRoxzD+y9Wkca74qkbBmPn/6FFEb7PSZTO+tPHjyodGNgz9XpJJRjQuBk1aDJtlF3m1QKBgE5SAr5ym65SZOU3UGUIOKRsfDW4Q/OsqDUImvpywCgBICaX9lHDShFFHwau7FA52ScL7vDquoMB4UtCOtLfyQYA9995w9oYCCurrVlVIJkb8jSLcADBHw3EmqF1kq3NqJqm9TmBfoDCh52vdCCUufxgKh33kfBOSlXuf7B8dgMbAoGAZ3r0/mBQX6S+s5+xCETMTSNv7TQzxgtURIpVs+ZVr2cMhWhiv+n0Omab9X9Z50se8cWl5lkvx8vn3D/XHHIPrMF6qk7RAXtvReb+PeitNvm0odqjFv0J2qki6fDs0HKwq4kojAXI1Md8Th0eobNjsy21fEEJT7uKMJdovI/SErI=",
              "pubKey": "CAASpgIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQCaNSDOjPz6T8HZsf7LDpxiQRiN2OjeyIHUS05p8QWOr3EFUCFsC31R4moihE5HN+FxNalUyyFZU//yjf1pdnlMJqrVByJSMa+y2y4x2FucpoCAO97Tx+iWzwlZ2UXEUXM1Y81mhPbeWXy+wP2xElTgIER0Tsn/thoA0SD2u9wJuVvM7dB7cBcHYmqV6JH+KWCedRTum6O1BssqP/4Lbm2+rkrbZ4+oVRoU2DRLoFhKqwqLtylrbuj4XOI3XykMXV5+uQXz1JzubNOB9lsc6K+eRC+w8hhhDuFMgzkZ4qomCnx3uhO67KaICd8yqqBa6PJ/+fBM5Xk4hjyR40bwcf41AgMBAAE="
            }
            """

        let peerID = try PeerID(fromJSON: jsDialer.data(using: .utf8)!)

        let keyType: Exchange.KeyType
        switch peerID.keyPair!.keyType {
        case .rsa:
            keyType = .rsa
        case .ed25519:
            keyType = .ed25519
        case .secp256k1:
            keyType = .secp256K1
        }

        var pubkey = Exchange.PublicKey()
        pubkey.type = keyType
        pubkey.data = try Data(peerID.marshalPublicKey())

        var exch = Exchange()
        exch.id = Data(peerID.id)
        exch.pubkey = pubkey

        let encoded = try exch.serializedData()
        //print(encoded.asString(base: .base16))

        #expect(encoded.asString(base: .base16) == expectedHexDialer)
    }

    @Test func testExchangeProtobufEncodingJSEchoExampleListenersKey() throws {
        let expectedHexListener =
            "0a221220d7a3092350d15eb2885ec0946a7ab5d55f23b743783df66a36a691b2df7704be12b002080012ab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100cb65970671b7b8bab30156570780dd1fa41ca663ec5caf5e1ec2f6a61bbd1064e0c9d61f6c963944c59edbc8b8aa441dd28c66dfa59d45c1ece7c0dc56b56a631e44234d55dea0ce041200d234efc283b28d954d0426f80f2577843f2700657f6a0126d527a104090e34551ffbf92b92aac74bd61ae61b5a61a488eef83eb880bc96a70f561002dfccca47e48fcabdd6035b737883acce0f6999369e72d601de5c83475b3305e7de8eef6ffa731c32897575789d34d24d4d567532acf8d7b6a59d54b464e485352c5a684285d95395f584a8f56b6aadfe565f1f978eac5feb6a61dcc21ad8630d6dc5ff091d758efdb6cd9f27a64c702fbad6694cdaec5804df0203010001"

        let jsListener = """
            {
              "id": "QmcrQZ6RJdpYuGvZqD5QEHAv6qX4BrQLJLQPQUrTrzdcgm",
              "privKey": "CAASqAkwggSkAgEAAoIBAQDLZZcGcbe4urMBVlcHgN0fpBymY+xcr14ewvamG70QZODJ1h9sljlExZ7byLiqRB3SjGbfpZ1FweznwNxWtWpjHkQjTVXeoM4EEgDSNO/Cg7KNlU0EJvgPJXeEPycAZX9qASbVJ6EECQ40VR/7+SuSqsdL1hrmG1phpIju+D64gLyWpw9WEALfzMpH5I/KvdYDW3N4g6zOD2mZNp5y1gHeXINHWzMF596O72/6cxwyiXV1eJ000k1NVnUyrPjXtqWdVLRk5IU1LFpoQoXZU5X1hKj1a2qt/lZfH5eOrF/ramHcwhrYYw1txf8JHXWO/bbNnyemTHAvutZpTNrsWATfAgMBAAECggEAQj0obPnVyjxLFZFnsFLgMHDCv9Fk5V5bOYtmxfvcm50us6ye+T8HEYWGUa9RrGmYiLweuJD34gLgwyzE1RwptHPj3tdNsr4NubefOtXwixlWqdNIjKSgPlaGULQ8YF2tm/kaC2rnfifwz0w1qVqhPReO5fypL+0ShyANVD3WN0Fo2ugzrniCXHUpR2sHXSg6K+2+qWdveyjNWog34b7CgpV73Ln96BWae6ElU8PR5AWdMnRaA9ucA+/HWWJIWB3Fb4+6uwlxhu2L50Ckq1gwYZCtGw63q5L4CglmXMfIKnQAuEzazq9T4YxEkp+XDnVZAOgnQGUBYpetlgMmkkh9qQKBgQDvsEs0ThzFLgnhtC2Jy//ZOrOvIAKAZZf/mS08AqWH3L0/Rjm8ZYbLsRcoWU78sl8UFFwAQhMRDBP9G+RPojWVahBL/B7emdKKnFR1NfwKjFdDVaoX5uNvZEKSl9UubbC4WZJ65u/cd5jEnj+w3ir9G8n+P1gp/0yBz02nZXFgSwKBgQDZPQr4HBxZL7Kx7D49ormIlB7CCn2i7mT11Cppn5ifUTrp7DbFJ2t9e8UNk6tgvbENgCKXvXWsmflSo9gmMxeEOD40AgAkO8Pn2R4OYhrwd89dECiKM34HrVNBzGoB5+YsAno6zGvOzLKbNwMG++2iuNXqXTk4uV9GcI8OnU5ZPQKBgCZUGrKSiyc85XeiSGXwqUkjifhHNh8yH8xPwlwGUFIZimnD4RevZI7OEtXw8iCWpX2gg9XGuyXOuKORAkF5vvfVriV4e7c9Ad4Igbj8mQFWz92EpV6NHXGCpuKqRPzXrZrNOA9PPqwSs+s9IxI1dMpk1zhBCOguWx2m+NP79NVhAoGBAI6WSoTfrpu7ewbdkVzTWgQTdLzYNe6jmxDf2ZbKclrf7lNr/+cYIK2Ud5qZunsdBwFdgVcnu/02czeS42TvVBgs8mcgiQc/Uy7yi4/VROlhOnJTEMjlU2umkGc3zLzDgYiRd7jwRDLQmMrYKNyEr02HFKFn3w8kXSzW5I8rISnhAoGBANhchHVtJd3VMYvxNcQb909FiwTnT9kl9pkjhwivx+f8/K8pDfYCjYSBYCfPTM5Pskv5dXzOdnNuCj6Y2H/9m2SsObukBwF0z5Qijgu1DsxvADVIKZ4rzrGb4uSEmM6200qjJ/9U98fVM7rvOraakrhcf9gRwuspguJQnSO9cLj6",
              "pubKey": "CAASpgIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDLZZcGcbe4urMBVlcHgN0fpBymY+xcr14ewvamG70QZODJ1h9sljlExZ7byLiqRB3SjGbfpZ1FweznwNxWtWpjHkQjTVXeoM4EEgDSNO/Cg7KNlU0EJvgPJXeEPycAZX9qASbVJ6EECQ40VR/7+SuSqsdL1hrmG1phpIju+D64gLyWpw9WEALfzMpH5I/KvdYDW3N4g6zOD2mZNp5y1gHeXINHWzMF596O72/6cxwyiXV1eJ000k1NVnUyrPjXtqWdVLRk5IU1LFpoQoXZU5X1hKj1a2qt/lZfH5eOrF/ramHcwhrYYw1txf8JHXWO/bbNnyemTHAvutZpTNrsWATfAgMBAAE="
            }
            """

        let peerID = try PeerID(fromJSON: jsListener.data(using: .utf8)!)

        let keyType: Exchange.KeyType
        switch peerID.keyPair!.keyType {
        case .rsa:
            keyType = .rsa
        case .ed25519:
            keyType = .ed25519
        case .secp256k1:
            keyType = .secp256K1
        }

        var pubkey = Exchange.PublicKey()
        pubkey.type = keyType
        pubkey.data = try Data(peerID.marshalPublicKey())

        var exch = Exchange()
        exch.id = Data(peerID.id)
        exch.pubkey = pubkey

        let encoded = try exch.serializedData()
        //print(encoded.asString(base: .base16))

        #expect(encoded.asString(base: .base16) == expectedHexListener)
    }
}
