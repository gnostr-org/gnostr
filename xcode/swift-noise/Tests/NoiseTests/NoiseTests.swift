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

// TODO: remove the @preconcurrency tag once we drop support for swift 6.0
@preconcurrency import Crypto
import Foundation
import Testing

@testable import Noise

@Suite("Noise Tests")
struct NoiseTests {

    let initiatorsStatic = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
    )

    let respondersStatic = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20")
    )

    let initiatorsEphemeral = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f")
    )

    let respondersEphemeral = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60")
    )

    @Test func testHMACChaining() throws {
        let alicePrivateKey = Curve25519.KeyAgreement.PrivateKey()
        let bobsPublicKeyData = Curve25519.KeyAgreement.PrivateKey().publicKey.rawRepresentation
        let protocolSalt = "SomePubliclySharedSalt".data(using: .utf8)!
        let bobsPublicKey = try! Curve25519.KeyAgreement.PublicKey(rawRepresentation: bobsPublicKeyData)
        let sharedSecret = try! alicePrivateKey.sharedSecretFromKeyAgreement(with: bobsPublicKey)
        let chainingKey = sharedSecret.hkdfDerivedSymmetricKey(
            using: SHA256.self,
            salt: protocolSalt,
            sharedInfo: Data(),
            outputByteCount: 32
        )

        let inputKeyMaterial: Data = Data()

        // Here we're testing HMAC Chaining with a fresh HMAC for every expansion
        var hmac = HMAC<SHA256>(key: chainingKey)
        hmac.update(data: inputKeyMaterial)
        let tempKey = SymmetricKey(data: Data(hmac.finalize()))

        var hmac1 = HMAC<SHA256>(key: tempKey)
        hmac1.update(data: Data([0x01]))
        let output1 = Data(hmac1.finalize())

        var hmac2 = HMAC<SHA256>(key: tempKey)
        hmac2.update(data: output1 + Data([0x02]))
        let output2 = Data(hmac2.finalize())

        // As long as the data being hashed and the key that's signing is the same, the output should stay the same.
        var hmac2Same = HMAC<SHA256>(key: tempKey)
        hmac2Same.update(data: output1 + Data([0x02]))
        let output2Same = Data(hmac2Same.finalize())

        var hmac3 = HMAC<SHA256>(key: tempKey)
        hmac3.update(data: output2 + Data([0x03]))
        let output3 = Data(hmac3.finalize())

        // As long as the data being hashed and the key that's signing the data is the same, the output should stay the same.
        #expect(output2 == output2Same)

        // Here we're testing HMAC Chaining with a singular HMAC instance
        // Calling update multiple times on a single HMAC instantiation is NOT the same as instantiating a new HMAC for each expansion...
        var hmac1U = HMAC<SHA256>(key: tempKey)
        hmac1U.update(data: Data([0x01]))
        let output1U = Data(hmac1U.finalize())

        hmac1U.update(data: output1U + Data([0x02]))
        let output2U = Data(hmac1U.finalize())

        hmac1U.update(data: output2U + Data([0x03]))
        let output3U = Data(hmac1U.finalize())

        // The first pass is equal
        #expect(output1U == output1)
        // The following passes are NOT equal
        #expect(output2U != output2)
        #expect(output3U != output3)
    }

    /// These test params come from the [js-libp2p-noise test suite](https://github.com/NodeFactoryIo/js-libp2p-noise/blob/f9d56d8c87635ec03b6d7aa50e594b57923f41df/test/handshakes/xx.spec.ts)
    @Test func testHMACChaining2_HKDF() throws {
        let ckBytes = Array(
            hex:
                "4e6f6973655f58585f32353531395f58436861436861506f6c795f53484132353600000000000000000000000000000000000000000000000000000000000000"
        )
        let chainingKey = SymmetricKey(data: Data(ckBytes.prefix(32)))
        let ikm = Array(
            hex:
                "a3eae50ea37a47e8a7aa0c7cd8e16528670536dcd538cebfd724fb68ce44f1910ad898860666227d4e8dd50d22a9a64d1c0a6f47ace092510161e9e442953da3"
        )

        let hf = Noise.NoiseHashFunction.sha256
        let keys = try hf.HKDF(chainingKey: chainingKey, inputKeyMaterial: Array(ikm), numOutputs: 3)

        #expect(keys.0.toHexString() == "cc5659adff12714982f806e2477a8d5ddd071def4c29bb38777b7e37046f6914")
        #expect(keys.1.toHexString() == "a16ada915e551ab623f38be674bb4ef15d428ae9d80688899c9ef9b62ef208fa")
        #expect(keys.2!.toHexString() == "ff67bf9727e31b06efc203907e6786667d2c7a74ac412b4d31a80ba3fd766f68")
    }

    /// The Noise_XX_25519_ChaChaPoly_SHA256 test vector
    /// ```
    /// handshake=Noise_XX_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4560a34e36ea82109f26cf2e5a5caf992b608d55c747f615e5a3425a7a19eefb8f
    /// msg_2_payload=
    /// msg_2_ciphertext=87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d97e5ea11b16f3968710b23a3be3202dc1b5e1ce3c963347491e74f5c0768a9b42
    /// msg_3_payload=79656c6c6f777375626d6172696e65 -> yellowsubmarine //Uses shared CipherState 1
    /// msg_3_ciphertext=a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6
    /// msg_4_payload=7375626d6172696e6579656c6c6f77 -> submarineyellow //Uses shared CipherState 2
    /// msg_4_ciphertext=2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521
    /// ```
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString())
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (decryptedMessage1Payload, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4560a34e36ea82109f26cf2e5a5caf992b608d55c747f615e5a3425a7a19eefb8f"
        )

        // Have initiator consume the message
        let (decryptedMessage2Payload, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d97e5ea11b16f3968710b23a3be3202dc1b5e1ce3c963347491e74f5c0768a9b42"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (decryptedMessage3Payload, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521")
    }

    /// handshake=Noise_XX_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=746573745f6d73675f30
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254746573745f6d73675f30
    /// msg_1_payload=746573745f6d73675f31
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4572e7a2ba5123ac30618b3d205f5c2d17f50cbca216483ac56bcc78e33bf520303278db641e5e731b2e3a
    /// msg_2_payload=746573745f6d73675f32
    /// msg_2_ciphertext=87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d9f27e318e43ba630594c4d08eeb3b36d97c7377a2f4f9144b2f0c8095ad92140505b2ab53eff244b14138
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_With_Payloads() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: Array(hex: "746573745f6d73675f30"))

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254746573745f6d73675f30"
        )

        // Have Responder consume the message
        let (decryptedMessage1Payload, _, _) = try responder.readMessage(msgInit1)

        #expect(decryptedMessage1Payload == Array(hex: "746573745f6d73675f30"))

        let (msgResp2, _, _) = try responder.writeMessage(payload: Array(hex: "746573745f6d73675f31"))

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4572e7a2ba5123ac30618b3d205f5c2d17f50cbca216483ac56bcc78e33bf520303278db641e5e731b2e3a"
        )

        // Have initiator consume the message
        let (decryptedMessage2Payload, _, _) = try initiator.readMessage(msgResp2)

        #expect(decryptedMessage2Payload == Array(hex: "746573745f6d73675f31"))

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: Array(hex: "746573745f6d73675f32"))

        #expect(
            msgInit3.toHexString()
                == "87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d9f27e318e43ba630594c4d08eeb3b36d97c7377a2f4f9144b2f0c8095ad92140505b2ab53eff244b14138"
        )

        // Assert that our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (decryptedMessage3Payload, respCS1, respCS2) = try responder.readMessage(msgInit3)

        #expect(decryptedMessage3Payload == Array(hex: "746573745f6d73675f32"))

        // Assert that our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521")
    }

    /// handshake=Noise_XX_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// prologue=6e6f74736563726574
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4588f043d1e49a3289b1beeab8f96b0551a48cddf9f38b1a12e46c6908644198f3
    /// msg_2_payload=
    /// msg_2_ciphertext=87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d95a04fa1f1c41fb3f00d496f242c1e44ce5b749b3d54bf74cea2dad086d601fb6
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_With_Prologue() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4588f043d1e49a3289b1beeab8f96b0551a48cddf9f38b1a12e46c6908644198f3"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d95a04fa1f1c41fb3f00d496f242c1e44ce5b749b3d54bf74cea2dad086d601fb6"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521")
    }

    /// handshake=Noise_XX_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// prologue=6e6f74736563726574
    /// msg_0_payload=746573745f6d73675f30
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254746573745f6d73675f30
    /// msg_1_payload=746573745f6d73675f31
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4545958c588d17d6373e0c1dcfa3755d37f50cbca216483ac56bcc98f5095870aa814ba40c08079c11f087
    /// msg_2_payload=746573745f6d73675f32
    /// msg_2_ciphertext=87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d9c1e9a1a313d02b78871cfd178a521a4c7c7377a2f4f9144b2f0ccedc84d379151b466741e4b266db6023
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_With_Payload_and_Prologue() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: Array(hex: "746573745f6d73675f30"))

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254746573745f6d73675f30"
        )

        // Have Responder consume the message
        let (decryptedMessage1Payload, _, _) = try responder.readMessage(msgInit1)

        #expect(decryptedMessage1Payload == Array(hex: "746573745f6d73675f30"))

        let (msgResp2, _, _) = try responder.writeMessage(payload: Array(hex: "746573745f6d73675f31"))

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4545958c588d17d6373e0c1dcfa3755d37f50cbca216483ac56bcc98f5095870aa814ba40c08079c11f087"
        )

        // Have initiator consume the message
        let (decryptedMessage2Payload, _, _) = try initiator.readMessage(msgResp2)

        #expect(decryptedMessage2Payload == Array(hex: "746573745f6d73675f31"))

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: Array(hex: "746573745f6d73675f32"))

        #expect(
            msgInit3.toHexString()
                == "87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d9c1e9a1a313d02b78871cfd178a521a4c7c7377a2f4f9144b2f0ccedc84d379151b466741e4b266db6023"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (decryptedMessage3Payload, respCS1, respCS2) = try responder.readMessage(msgInit3)

        #expect(decryptedMessage3Payload == Array(hex: "746573745f6d73675f32"))

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521")
    }

    /// handshake=Noise_XXpsk0_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// preshared_key=2176657279736563726574766572797365637265747665727973656372657421
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254555f63b2499511e3b299a5649351a5b7
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846622df46c0ac1e0fc71795a84e37cc0e963d131c8e84c02cd5cfcfe8def3fe128b324ab54fd3be59ac143dfdc68996211170078c960e051d6ed971da20bde0fcf1
    /// msg_2_payload=
    /// msg_2_ciphertext=462127adbe047db3d1fce0581b5447d99b606c591545a7719132e0c91fe93d123be3fe89ffd8af56e02cefda863080ecee5651dcecddde76b8237c66740d7c8f
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=75fff8afebd2f14da1cac9cc5b5201395cdf2ad65f3a97804e360c16f4e2ac
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=150cc85f79f9ca0f0730b8b4707805ed1969ff6b2770a5d466cd2754802805
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_PreSharedKey0() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 0
                    ),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 0
                    ),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254555f63b2499511e3b299a5649351a5b7"
        )

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846622df46c0ac1e0fc71795a84e37cc0e963d131c8e84c02cd5cfcfe8def3fe128b324ab54fd3be59ac143dfdc68996211170078c960e051d6ed971da20bde0fcf1"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "462127adbe047db3d1fce0581b5447d99b606c591545a7719132e0c91fe93d123be3fe89ffd8af56e02cefda863080ecee5651dcecddde76b8237c66740d7c8f"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "75fff8afebd2f14da1cac9cc5b5201395cdf2ad65f3a97804e360c16f4e2ac")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "150cc85f79f9ca0f0730b8b4707805ed1969ff6b2770a5d466cd2754802805")
    }

    /// handshake=Noise_XXpsk1_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// preshared_key=2176657279736563726574766572797365637265747665727973656372657421
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd1662549f27cade2b6f2db14f582e49cfbfc068
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484667f0f30704cd806d42849595f4e39d8ace7b1f7ab9c62c9ccaf7284b3d8ce0d88286de6a5c75efd3ada339fd7ba335dee5fe0151a61f7decdabe8fab42d807358
    /// msg_2_payload=
    /// msg_2_ciphertext=3709db3d2b87c711bdd3ef87e62edd8a2775482a4421a58fb5eeb106861e98d24c021634f68b6fa8a9c2f48161e190714c2a2d90a55d2dc32f33fcbaf5afc67c
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=61eaa2290029bcde241e90efb965beeb7837ec5441928800275670fdb058de
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=ee55ef942191e45cf5bcea014c4a0c71f0780ff6095ff93f467e7a746e264c
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_PreSharedKey1() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 1
                    ),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 1
                    ),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd1662549f27cade2b6f2db14f582e49cfbfc068"
        )

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484667f0f30704cd806d42849595f4e39d8ace7b1f7ab9c62c9ccaf7284b3d8ce0d88286de6a5c75efd3ada339fd7ba335dee5fe0151a61f7decdabe8fab42d807358"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "3709db3d2b87c711bdd3ef87e62edd8a2775482a4421a58fb5eeb106861e98d24c021634f68b6fa8a9c2f48161e190714c2a2d90a55d2dc32f33fcbaf5afc67c"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "61eaa2290029bcde241e90efb965beeb7837ec5441928800275670fdb058de")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "ee55ef942191e45cf5bcea014c4a0c71f0780ff6095ff93f467e7a746e264c")
    }

    /// handshake=Noise_XXpsk2_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// preshared_key=2176657279736563726574766572797365637265747665727973656372657421
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd16625462f2d89ffb750657573d23edc7c79728
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466c9996f0e38eded0281a0f505a4f2473b114924724e374c408b3ba103abc7ffbf72bf9b5f5f37f1a8ee33e4708c1f35d54d93fc2a553004be11b9a6ad56d03f30
    /// msg_2_payload=
    /// msg_2_ciphertext=4744625f46dfce9240a5b1927393fd862a2520366f4df66de4b75019d201de92f3bd1d11aef54b65374c268c0ec19d34ec1fff795f07ef7065932e5983ee2e84
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=70847317d915af289e3ff17e5d66e4b4b0020d1bd997b8bb17cfa15710db0b
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=ad071b14d700e2789c1251f57e3b1e455e3f3be012d7ab6abce986b536ba21
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_PreSharedKey2() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 2
                    ),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 2
                    ),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd16625462f2d89ffb750657573d23edc7c79728"
        )

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466c9996f0e38eded0281a0f505a4f2473b114924724e374c408b3ba103abc7ffbf72bf9b5f5f37f1a8ee33e4708c1f35d54d93fc2a553004be11b9a6ad56d03f30"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "4744625f46dfce9240a5b1927393fd862a2520366f4df66de4b75019d201de92f3bd1d11aef54b65374c268c0ec19d34ec1fff795f07ef7065932e5983ee2e84"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "70847317d915af289e3ff17e5d66e4b4b0020d1bd997b8bb17cfa15710db0b")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "ad071b14d700e2789c1251f57e3b1e455e3f3be012d7ab6abce986b536ba21")
    }

    /// handshake=Noise_XXpsk3_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// preshared_key=2176657279736563726574766572797365637265747665727973656372657421
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254648d756cb03de7ad06e87f9a577c00de
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846696a7a5454cc70bb4eec2a2f7c616c143564ff1ae149458f9e70afb3498be7a88c9feda8ece3bb7d846bd57a37fc9cf362b7d090998d862bd82fcf9a19cf154e1
    /// msg_2_payload=
    /// msg_2_ciphertext=f5b6224ea13577089dc14b20ca8e90d0cedede4faff50348d4d0a0f941182ad787d1e72132665f8402f660af90e07e671606bb5a4931d244dfa6590809fac237
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=5e80fec73b32f6ff466aa5addbc2b16e2cf062f09c36796ecb2efcc35cac99
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=df3c8983cb9f286df65e57d0010dc65eeca3bca44b6b240da8ebf92be581cd
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_PreSharedKey3() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 3
                    ),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 3
                    ),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254648d756cb03de7ad06e87f9a577c00de"
        )

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846696a7a5454cc70bb4eec2a2f7c616c143564ff1ae149458f9e70afb3498be7a88c9feda8ece3bb7d846bd57a37fc9cf362b7d090998d862bd82fcf9a19cf154e1"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "f5b6224ea13577089dc14b20ca8e90d0cedede4faff50348d4d0a0f941182ad787d1e72132665f8402f660af90e07e671606bb5a4931d244dfa6590809fac237"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "5e80fec73b32f6ff466aa5addbc2b16e2cf062f09c36796ecb2efcc35cac99")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "df3c8983cb9f286df65e57d0010dc65eeca3bca44b6b240da8ebf92be581cd")
    }

    // The Handshake Initialization should fail with an index out of bounds error for PSK placement
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_PreSharedKey4() throws {

        /// Key placement of 4 is invalid... this should throw an error
        #expect(throws: Noise.Errors.custom("Invalid presharedKey placement")) {
            try Noise.HandshakeState(
                config:
                    Noise.Config(
                        cipherSuite: Noise.CipherSuite(
                            keyCurve: .ed25519,
                            cipher: .ChaChaPoly1305,
                            hashFunction: .sha256
                        ),
                        handshakePattern: Noise.Handshakes.XX,
                        initiator: true,
                        prologue: [],
                        presharedKey: (
                            key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                            placement: 4
                        ),
                        staticKeypair: initiatorsStatic,
                        ephemeralKeypair: initiatorsEphemeral
                    )
            )
        }

        /// Key placement of 4 is invalid... this should throw an error
        #expect(throws: Noise.Errors.custom("Invalid presharedKey placement")) {
            try Noise.HandshakeState(
                config:
                    Noise.Config(
                        cipherSuite: Noise.CipherSuite(
                            keyCurve: .ed25519,
                            cipher: .ChaChaPoly1305,
                            hashFunction: .sha256
                        ),
                        handshakePattern: Noise.Handshakes.XX,
                        initiator: false,
                        prologue: [],
                        presharedKey: (
                            key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                            placement: 4
                        ),
                        staticKeypair: respondersStatic,
                        ephemeralKeypair: respondersEphemeral
                    )
            )
        }
    }

    /// Noise_XX_25519_ChaChaPoly_SHA512
    ///```
    /// handshake=Noise_XX_25519_ChaChaPoly_SHA512
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846692e5b8dda95b4ec55e42c2cbded11735474b3612a895298bcb02e8469353fe827273e4a7aadfc1aa32578b46bce2006fe1482f062e2f27e43ad23e67a304c030
    /// msg_2_payload=
    /// msg_2_ciphertext=ac3087e2342498dfa6606faf700dc5782b9612bdbc8bbb67a87181baac2d693d2f8df70600534bcbd389bbbf733550ae3e7e9a78f80aafa70d6211640223800d
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=2dcb8503b438910b2a2ffcf242ef705e6cce2d25bd30444402427981ee2064
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=56d2ce5c1e7e28b7406b99aff512114313b811e17c0af6497baa906165ba31
    /// ```
    @Test func testNoise_XX_25519_ChaChaPoly_SHA512() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha512
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha512
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846692e5b8dda95b4ec55e42c2cbded11735474b3612a895298bcb02e8469353fe827273e4a7aadfc1aa32578b46bce2006fe1482f062e2f27e43ad23e67a304c030"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "ac3087e2342498dfa6606faf700dc5782b9612bdbc8bbb67a87181baac2d693d2f8df70600534bcbd389bbbf733550ae3e7e9a78f80aafa70d6211640223800d"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "2dcb8503b438910b2a2ffcf242ef705e6cce2d25bd30444402427981ee2064")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "56d2ce5c1e7e28b7406b99aff512114313b811e17c0af6497baa906165ba31")
    }

    /// Noise_XX_25519_AESGCM_SHA256
    /// ```
    /// handshake=Noise_XX_25519_AESGCM_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484665393019dbd6f438795da206db0886610b26108e424142c2e9b5fd1f7ea70cde8767ce62d7e3c0e9bcefe4ab872c0505b9e824df091b74ffe10a2b32809cab21f
    /// msg_2_payload=
    /// msg_2_ciphertext=e610eadc4b00c17708bf223f29a66f02342fbedf6c0044736544b9271821ae40e70144cecd9d265dffdc5bb8e051c3f83db32a425e04d8f510c58a43325fbc56
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=9ea1da1ec3bfecfffab213e537ed1791bfa887dd9c631351b3f63d6315ab9a
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=217c5111fad7afde33bd28abaff3def88a57ab50515115d23a10f28621f842
    /// ```
    @Test func testNoise_XX_25519_AESGCM_SHA256() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484665393019dbd6f438795da206db0886610b26108e424142c2e9b5fd1f7ea70cde8767ce62d7e3c0e9bcefe4ab872c0505b9e824df091b74ffe10a2b32809cab21f"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "e610eadc4b00c17708bf223f29a66f02342fbedf6c0044736544b9271821ae40e70144cecd9d265dffdc5bb8e051c3f83db32a425e04d8f510c58a43325fbc56"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "9ea1da1ec3bfecfffab213e537ed1791bfa887dd9c631351b3f63d6315ab9a")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "217c5111fad7afde33bd28abaff3def88a57ab50515115d23a10f28621f842")

    }

    /// Noise_XX_25519_AESGCM_SHA512
    /// ```
    /// handshake=Noise_XX_25519_AESGCM_SHA512
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466881a9849f98286c79700c48c40e6667ce14ce8baabdf27b51fb80d248c2d56a65be777dc2ad2438d794410a91e1542a138b33b73a5ff808ecff2e90952defca9
    /// msg_2_payload=
    /// msg_2_ciphertext=a0c7c991f077df03c26762bb80c9dc4c830c71a012dc1a002363a684c659a3487c8a7c790075c7a5ac8de6fe1ccc7363d39bea6035a91323f511f662ee40d9de
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=d52095f5c41973904a84746d988f0e424ec0832c3257cb4675eab76c4c197f
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=86e1a5d80c71d13bde2e6b2559ecc953b97939de528e1ae166a64540265918
    /// ```
    @Test func testNoise_XX_25519_AESGCM_SHA512() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha512
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha512
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466881a9849f98286c79700c48c40e6667ce14ce8baabdf27b51fb80d248c2d56a65be777dc2ad2438d794410a91e1542a138b33b73a5ff808ecff2e90952defca9"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "a0c7c991f077df03c26762bb80c9dc4c830c71a012dc1a002363a684c659a3487c8a7c790075c7a5ac8de6fe1ccc7363d39bea6035a91323f511f662ee40d9de"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "d52095f5c41973904a84746d988f0e424ec0832c3257cb4675eab76c4c197f")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "86e1a5d80c71d13bde2e6b2559ecc953b97939de528e1ae166a64540265918")

    }

    /// The Noise_NN_25519_AESGCM_SHA256 test vector
    /// ```
    /// handshake=Noise_NN_25519_AESGCM_SHA256
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484667cc0d7b4540fd183ba30ecbd3f464f16
    /// msg_2_payload=79656c6c6f777375626d6172696e65
    /// msg_2_ciphertext=a0193b62b90fb3497108ec8adcc340a49ebb0a07f1654d71f7e38361f57ba5
    /// msg_3_payload=7375626d6172696e6579656c6c6f77
    /// msg_3_ciphertext=b2afdcb051e896fa5b6a23def5ee6bdd6032f1b39b2d22ef7da01857648389
    /// ```
    @Test func testNoise_NN_25519_AESGCM_SHA256() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.NN,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: nil,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.NN,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: nil,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, respCS1, respCS2) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484667cc0d7b4540fd183ba30ecbd3f464f16"
        )

        // Have initiator consume the message
        let (_, initCS1, initCS2) = try initiator.readMessage(msgResp2)

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "a0193b62b90fb3497108ec8adcc340a49ebb0a07f1654d71f7e38361f57ba5")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "b2afdcb051e896fa5b6a23def5ee6bdd6032f1b39b2d22ef7da01857648389")
    }

    /// handshake=Noise_KN_25519_AESGCM_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466827d5016baa63241017945dea7aeb9be
    /// msg_2_payload=79656c6c6f777375626d6172696e65
    /// msg_2_ciphertext=d8eb7e92e6ffa800b669953e5a1b99fe268df1161d7293a1c1836f7dd2d55b
    /// msg_3_payload=7375626d6172696e6579656c6c6f77
    /// msg_3_ciphertext=009f1432e8414277b5ddf687ae0daf50f76e24c5ed30b0d1e4af53544c70ad
    @Test func testNoise_KN_25519_AESGCM_SHA256() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha256
                    ),
                    handshake: .KN_Initiator,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha256
                    ),
                    handshake: .KN_Responder(remoteStatic: initiatorsStatic.publicKey),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, respCS1, respCS2) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466827d5016baa63241017945dea7aeb9be"
        )

        // Have initiator consume the message
        let (_, initCS1, initCS2) = try initiator.readMessage(msgResp2)

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "d8eb7e92e6ffa800b669953e5a1b99fe268df1161d7293a1c1836f7dd2d55b")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "009f1432e8414277b5ddf687ae0daf50f76e24c5ed30b0d1e4af53544c70ad")
    }

    /// handshake=Noise_XK_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd1662549963aa4003cb0f60f51f7f8b1c0e6a9c
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846630166c893dafe95f71d102a8ac640a52
    /// msg_2_payload=
    /// msg_2_ciphertext=24a819b832ab7a11dd1464c2baf72f2c49e0665757911662ab11495a5fd4437e0abe01f5c07176e776e02716c4cb98a005ec4c884c4dc7500d2d9b99e9670ab3
    /// msg_3_payload=79656c6c6f777375626d6172696e65
    /// msg_3_ciphertext=e8b0f2fc220f7edc287a91ba45c76f6da1327405789dc61e31a649f57d6d93
    /// msg_4_payload=7375626d6172696e6579656c6c6f77
    /// msg_4_ciphertext=ed6901a7cd973e880242b047fc86da03b498e8ed8e9838d6f3d107420dfcd9
    @Test func testNoise_XK_25519_ChaChaPoly_SHA256() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshake: .XK_Initiator(remoteStatic: respondersStatic.publicKey),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshake: .XK_Responder,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd1662549963aa4003cb0f60f51f7f8b1c0e6a9c"
        )

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d4846630166c893dafe95f71d102a8ac640a52"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "24a819b832ab7a11dd1464c2baf72f2c49e0665757911662ab11495a5fd4437e0abe01f5c07176e776e02716c4cb98a005ec4c884c4dc7500d2d9b99e9670ab3"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "e8b0f2fc220f7edc287a91ba45c76f6da1327405789dc61e31a649f57d6d93")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "ed6901a7cd973e880242b047fc86da03b498e8ed8e9838d6f3d107420dfcd9")

    }

    /// handshake=Noise_IXpsk1_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// prologue=6e6f74736563726574
    /// preshared_key=2176657279736563726574766572797365637265747665727973656372657421
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd1662545667d83bcfa7dbb6ee159dde3afffea915ce4084462fc02f7f7dc86c2d338a98e01aeac3a330e93ed7024c2417abe12b8b941d28126267f9ec338fb268badb92
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466f04c5f04c9ae8992c210cb0193a52ae8c081c44f33ab1490df3e3c344eb1457c60a8f15129cf89e14206666494b093758fbe507fcae17bee288b08f9c85ee4eb
    /// msg_2_payload=79656c6c6f777375626d6172696e65
    /// msg_2_ciphertext=b2590aa4f81a2fbc961d60abede55cc6a4a64a40f7bcd1642f0a31daace3fd
    /// msg_3_payload=7375626d6172696e6579656c6c6f77
    /// msg_3_ciphertext=82b96b24e3c63563ddab16453506322429609077c4182022c6b9ed126e172a
    @Test func testNoise_IX_25519_ChaChaPoly_SHA256_PresharedKey1() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshake: .IX_Initiator,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 1
                    ),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshake: .IX_Responder,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 1
                    ),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd1662545667d83bcfa7dbb6ee159dde3afffea915ce4084462fc02f7f7dc86c2d338a98e01aeac3a330e93ed7024c2417abe12b8b941d28126267f9ec338fb268badb92"
        )

        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)

        let (msgResp2, respCS1, respCS2) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d48466f04c5f04c9ae8992c210cb0193a52ae8c081c44f33ab1490df3e3c344eb1457c60a8f15129cf89e14206666494b093758fbe507fcae17bee288b08f9c85ee4eb"
        )

        // Have initiator consume the message
        let (_, initCS1, initCS2) = try initiator.readMessage(msgResp2)

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "b2590aa4f81a2fbc961d60abede55cc6a4a64a40f7bcd1642f0a31daace3fd")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "82b96b24e3c63563ddab16453506322429609077c4182022c6b9ed126e172a")
    }

    /// The Noise_XX_25519_ChaChaPoly_SHA256 test vector
    /// ```
    /// handshake=Noise_XX_25519_ChaChaPoly_SHA256
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// msg_0_payload=
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254
    /// msg_1_payload=
    /// msg_1_ciphertext=64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4560a34e36ea82109f26cf2e5a5caf992b608d55c747f615e5a3425a7a19eefb8f
    /// msg_2_payload=
    /// msg_2_ciphertext=87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d97e5ea11b16f3968710b23a3be3202dc1b5e1ce3c963347491e74f5c0768a9b42
    /// msg_3_payload=79656c6c6f777375626d6172696e65 -> yellowsubmarine //Uses shared CipherState 1
    /// msg_3_ciphertext=a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6
    /// msg_4_payload=7375626d6172696e6579656c6c6f77 -> submarineyellow //Uses shared CipherState 2
    /// msg_4_ciphertext=2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521
    /// ```
    @Test func testNoise_XX_25519_ChaChaPoly_SHA256_MULTICALL() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: true,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .ChaChaPoly1305,
                        hashFunction: .sha256
                    ),
                    handshakePattern: Noise.Handshakes.XX,
                    initiator: false,
                    prologue: [],
                    presharedKey: nil,
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, _, _) = try initiator.writeMessage(payload: [])
        // Calling write again should throw an error
        #expect(throws: Noise.Errors.custom("noise: unexpected call to WriteMessage should be ReadMessage")) {
            try initiator.writeMessage(payload: [])
        }

        print(msgInit1.toHexString)
        #expect(msgInit1.toHexString() == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd166254")

        // Calling Write when we should Read should throw an error
        #expect(throws: Noise.Errors.custom("noise: unexpected call to WriteMessage should be ReadMessage")) {
            try responder.writeMessage(payload: [])
        }
        // Have Responder consume the message
        let (_, _, _) = try responder.readMessage(msgInit1)
        // Calling read a second time should throw an error
        #expect(throws: Noise.Errors.custom("noise: unexpected call to ReadMessage should be WriteMessage")) {
            try responder.readMessage(msgInit1)
        }

        let (msgResp2, _, _) = try responder.writeMessage(payload: [])

        #expect(
            msgResp2.toHexString()
                == "64b101b1d0be5a8704bd078f9895001fc03e8e9f9522f188dd128d9846d484663414af878d3e46a2f58911a816d6e8346d4ea17a6f2a0bb4ef4ed56c133cff4560a34e36ea82109f26cf2e5a5caf992b608d55c747f615e5a3425a7a19eefb8f"
        )

        // Have initiator consume the message
        let (_, _, _) = try initiator.readMessage(msgResp2)

        // Have Initiator write message 3
        let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: [])

        #expect(
            msgInit3.toHexString()
                == "87f864c11ba449f46a0a4f4e2eacbb7b0457784f4fca1937f572c93603e9c4d97e5ea11b16f3968710b23a3be3202dc1b5e1ce3c963347491e74f5c0768a9b42"
        )

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Have responder consume message 3
        let (_, respCS1, respCS2) = try responder.readMessage(msgInit3)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "a52ef02ba60e12696d1d6b9ef4245c88fca757b6134ad6e76b56e310a6adf6")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "2445aa438ebd649281c636cc7269ca82f1d9023d72520943aeabf909cdf521")
    }

    /// handshake=Noise_Xpsk1_25519_AESGCM_SHA512
    /// init_static=000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f
    /// resp_static=0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20
    /// gen_init_ephemeral=202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f
    /// gen_resp_ephemeral=4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60
    /// prologue=6e6f74736563726574
    /// preshared_key=2176657279736563726574766572797365637265747665727973656372657421
    /// msg_0_payload=746573745f6d73675f30
    /// msg_0_ciphertext=358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd16625464d0a2b152420f100932269d5d383be29d5262c4287efbfc1a4689f88752b5e86e5ef46c6d25996ebdb230b7431be817ef2c2f22b87d9c53accd25f4ff987e5fab341d7f7fb4079e5c4b
    /// msg_1_payload=79656c6c6f777375626d6172696e65
    /// msg_1_ciphertext=e5abf632a1e20300ec96b8849b1debe07702a0474191af8ec95d2120703ac0
    /// msg_2_payload=7375626d6172696e6579656c6c6f77
    /// msg_2_ciphertext=0094ea104502c9337f6fdc742e949099f369f0f4c83a9327686b5fa3a39cb3
    @Test func testXHandshakeWithEnum() throws {

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha512
                    ),
                    handshake: .X_Initiator(remoteStatic: respondersStatic.publicKey),
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 1
                    ),
                    staticKeypair: initiatorsStatic,
                    ephemeralKeypair: initiatorsEphemeral
                )
        )

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: Noise.CipherSuite(
                        keyCurve: .ed25519,
                        cipher: .AESGCM,
                        hashFunction: .sha512
                    ),
                    handshake: .X_Responder,
                    prologue: Array(hex: "6e6f74736563726574"),
                    presharedKey: (
                        key: Array(hex: "2176657279736563726574766572797365637265747665727973656372657421"),
                        placement: 1
                    ),
                    staticKeypair: respondersStatic,
                    ephemeralKeypair: respondersEphemeral
                )
        )

        // Have Initiator write our first message
        let (msgInit1, initCS1, initCS2) = try initiator.writeMessage(payload: Array(hex: "746573745f6d73675f30"))

        print(msgInit1.toHexString)
        #expect(
            msgInit1.toHexString()
                == "358072d6365880d1aeea329adf9121383851ed21a28e3b75e965d0d2cd16625464d0a2b152420f100932269d5d383be29d5262c4287efbfc1a4689f88752b5e86e5ef46c6d25996ebdb230b7431be817ef2c2f22b87d9c53accd25f4ff987e5fab341d7f7fb4079e5c4b"
        )

        // Have Responder consume the message
        let (decryptedMessage1Payload, respCS1, respCS2) = try responder.readMessage(msgInit1)

        // Assert that our payload was recoverable
        #expect(decryptedMessage1Payload == Array(hex: "746573745f6d73675f30"))

        // Assert the our Initiators Handshake completed and they generated the shared CipherStates
        #expect(initCS1 != nil)
        #expect(initCS2 != nil)

        // Assert the our Responders Handshake completed and they generated the shared CipherStates
        #expect(respCS1 != nil)
        #expect(respCS2 != nil)

        // Assert that both of our shared CipherStates have been created and are equal
        #expect(initCS1!.k == respCS1!.k)
        #expect(initCS2!.k == respCS2!.k)

        // Encrypt and Decrypt the first message `yellowsubmarine` using our shared first CipherState
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        #expect(message1.toHexString() == "79656c6c6f777375626d6172696e65")
        let secureMessage1 = try initCS1!.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respCS1!.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)
        #expect(secureMessage1.toHexString() == "e5abf632a1e20300ec96b8849b1debe07702a0474191af8ec95d2120703ac0")

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        #expect(message2.toHexString() == "7375626d6172696e6579656c6c6f77")
        let secureMessage2 = try respCS2!.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initCS2!.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)
        #expect(secureMessage2.toHexString() == "0094ea104502c9337f6fdc742e949099f369f0f4c83a9327686b5fa3a39cb3")
    }

    // DH Shared Secret Without CryptoKit (SecKeyCopyKeyExchangeResult)
    //    func testDiffieSharedSecretFunction() throws {
    //        var error: Unmanaged<CFError>?
    //
    //        let keyPairAttr:[String : Any] = [kSecAttrKeySizeInBits as String: 256,
    //                                         SecKeyKeyExchangeParameter.requestedSize.rawValue as String: 32,
    //                                         kSecAttrKeyType as String: kSecAttrKeyTypeECSECPrimeRandom,
    //                                         kSecPrivateKeyAttrs as String: [kSecAttrIsPermanent as String: false],
    //                                         kSecPublicKeyAttrs as String:[kSecAttrIsPermanent as String: false]]
    //        let algorithm:SecKeyAlgorithm = SecKeyAlgorithm.ecdhKeyExchangeStandardX963SHA256//ecdhKeyExchangeStandardX963SHA256
    //
    //        do {
    //            guard let privateKey = SecKeyCreateRandomKey(keyPairAttr as CFDictionary, &error) else {
    //                throw error!.takeRetainedValue() as Error
    //            }
    //            let publicKey = SecKeyCopyPublicKey(privateKey)
    //            print("public ky1: \(String(describing: publicKey)),\n private key: \(privateKey)")
    //
    //
    //
    //            guard let privateKey2 = SecKeyCreateRandomKey(keyPairAttr as CFDictionary, &error) else {
    //                throw error!.takeRetainedValue() as Error
    //            }
    //            let publicKey2 = SecKeyCopyPublicKey(privateKey2)
    //            print("public ky2: \(String(describing: publicKey2)),\n private key2: \(privateKey2)")
    //
    //
    //
    //            let shared:CFData? = SecKeyCopyKeyExchangeResult(privateKey, algorithm, publicKey2!, keyPairAttr as CFDictionary, &error)
    //            let sharedData:Data = shared! as Data
    //            print("shared Secret key:   \(sharedData.asString(base: .base16))")
    //
    //            let shared2:CFData? = SecKeyCopyKeyExchangeResult(privateKey2, algorithm, publicKey!, keyPairAttr as CFDictionary, &error)
    //            let sharedData2:Data = shared2! as Data
    //            print("shared Secret key 2: \(sharedData2.asString(base: .base16))")
    //
    //            // shared secret key and shared secret key 2 should be same
    //
    //            /// This Fails....
    //            let priv = try Curve25519.KeyAgreement.PrivateKey(rawRepresentation: privateKey.rawRepresentation())
    //            let pub = try Curve25519.KeyAgreement.PublicKey(rawRepresentation: publicKey2!.rawRepresentation())
    //            let shared3 = try priv.sharedSecretFromKeyAgreement(with: pub)
    //            print("shared Secret key 3: \(shared3.asData.asString(base: .base16))")
    //
    //        } catch let error as NSError {
    //            print("error: \(error)")
    //        } catch  {
    //            print("unknown error")
    //        }
    //    }
}

struct NoiseTestSuite {
    static let initiatorsStatic = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f")
    )

    static let respondersStatic = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20")
    )

    static let initiatorsEphemeral = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f")
    )

    static let respondersEphemeral = try! Curve25519.KeyAgreement.PrivateKey(
        rawRepresentation:
            Array(hex: "4142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60")
    )

    static let presharedKey = Array(hex: "2176657279736563726574766572797365637265747665727973656372657421")

    /// This expects a set of one or more test vectors in the format presented [here](https://github.com/flynn/noise/blob/master/vectors.txt)
    internal static func doAllHandshakeVectors(_ vectors: String) throws -> Bool {
        let allTests = vectors.components(separatedBy: "\n\n")

        var failures: Int = 0

        for (i, test) in allTests.enumerated() {
            //let split = test.split(separator: "\n")
            let result = try NoiseTestSuite.doHandshakeVector(test, index: i)
            #expect(result)
            if result == false { failures += 1 }
        }

        print("--- Test Summary ---")
        print("Total Tests: \(allTests.count)")
        print("Passed: \(allTests.count - failures)")
        print("Failures: \(failures)")
        print("Percent Passing: \((Double(allTests.count - failures) / Double(allTests.count)) * 100)")

        return failures == 0
    }

    /// Takes a multi line String test vector, parses the Handshake, Payloads and Prologues, constructs the HandshakeState and performs the Handshake. Return True on success, False on error and prints out any failed Asserts...
    internal static func doHandshakeVector(_ str: String, index i: Int) throws -> Bool {
        let split = str.split(separator: "\n")
        guard let handshake = split.first(where: { $0.contains("handshake=") })?.split(separator: "=").last else {
            print("Failed to extract Handshake string from test vector")
            return false
        }

        var prologue: [UInt8] = []
        if let prologueStr = split.first(where: { $0.contains("prologue=") })?.split(separator: "=").last {
            //print("Found and appending prologue data: \(prologueStr)")
            prologue = Array(hex: String(prologueStr))
        }
        let plStr: [String] = split.filter({ $0.contains("payload") }).compactMap({ str -> Substring? in
            let comp = str.split(separator: "=")
            if comp.count > 1 {
                return comp.last
            } else {
                return nil
            }
        }).map { String($0) }.dropLast(2)
        var payloads: [Int: String] = [:]
        for (idx, p) in plStr.enumerated() {
            payloads[idx] = p
        }

        print(
            "Running Test Vector \(i) -> \(handshake)\(prologue.count > 0 ? " - Prologue" : "")\(payloads.count > 0 ? " - Payloads" : "")"
        )
        //print(payloads)

        let usesInitiatorsStaticKey = str.contains("init_static=")
        let usesRespondersStaticKey = str.contains("resp_static=")

        let result = try NoiseTestSuite.doArbitraryHandshakeVectorTextOutput(
            String(handshake),
            usesInitStatic: usesInitiatorsStaticKey,
            usesRespStatic: usesRespondersStaticKey,
            prologue: prologue,
            payloads: payloads
        )

        #expect(result == split.map { String($0) })
        //for (i, line) in split.enumerated() {
        //    XCTAssertEqual(String(line), result[i])
        //}

        if result != split.map({ String($0) }) {
            //failures += 1
            print("❌❌❌ Failed Test \(handshake) ❌❌❌")
            for (i, line) in split.enumerated() {
                if result[i] != String(line) {
                    print("-------------")
                    print(result[i])
                    print("Should equal")
                    print(String(line))
                    print("-------------")
                }
            }
            return false
        }

        return true
    }

    internal static func doArbitraryHandshakeVectorTextOutput(
        _ str: String,
        usesInitStatic: Bool,
        usesRespStatic: Bool,
        prologue: [UInt8],
        payloads: [Int: String] = [:]
    ) throws -> [String] {
        var messageLog: [String] = []

        let initiator = try createInitiator(
            str,
            initStatic: usesInitStatic,
            respStatic: usesRespStatic,
            prologue: prologue,
            messageLog: &messageLog
        )
        let responder = try createResponder(str, respStatic: usesRespStatic, prologue: prologue)

        var initiatorsCipherState: (c1: Noise.CipherState, c2: Noise.CipherState)? = nil
        var respondersCipherState: (c1: Noise.CipherState, c2: Noise.CipherState)? = nil

        // Alternate between writing and reading until our cipher suites are present...
        var msgIndex: Int = 0
        while initiatorsCipherState == nil && respondersCipherState == nil {

            let payloadOut: [UInt8]
            if let pl = payloads[msgIndex] {
                payloadOut = Array(hex: pl)
            } else {
                payloadOut = []
            }

            // Append msg_<idx>_payload
            messageLog.append("msg_\(msgIndex)_payload=\(payloadOut.toHexString())")

            if initiator.shouldWrite() {

                // Initiator Writes next message
                let (msgOut, initCS1, initCS2) = try initiator.writeMessage(payload: payloadOut)
                // Set our cipher states if their not nil
                if let ics1 = initCS1, let ics2 = initCS2 { initiatorsCipherState = (c1: ics1, c2: ics2) }

                // Responder Consumes message
                let (payload, respCS1, respCS2) = try responder.readMessage(msgOut)
                // Set our cipher states if their not nil
                if let rcs1 = respCS1, let rcs2 = respCS2 { respondersCipherState = (c1: rcs1, c2: rcs2) }
                // Assert that we we're able to recover the payload
                #expect(payload == payloadOut)

                // Append msg_<idx>_ciphertext
                messageLog.append("msg_\(msgIndex)_ciphertext=\(msgOut.toHexString())")

            } else {

                // Responder Writes next message
                let (msgOut, respCS1, respCS2) = try responder.writeMessage(payload: payloadOut)
                // Set our cipher states if their not nil
                if let rcs1 = respCS1, let rcs2 = respCS2 { respondersCipherState = (c1: rcs1, c2: rcs2) }

                // Initiator Consumes message
                let (payload, initCS1, initCS2) = try initiator.readMessage(msgOut)
                // Set our cipher states if their not nil
                if let ics1 = initCS1, let ics2 = initCS2 { initiatorsCipherState = (c1: ics1, c2: ics2) }
                // Assert that we we're able to recover the payload
                #expect(payload == payloadOut)

                // Append msg_<idx>_ciphertext
                messageLog.append("msg_\(msgIndex)_ciphertext=\(msgOut.toHexString())")
            }

            // Increment our msgIndex for logging purposes...
            msgIndex += 1
        }

        // Proceed to send our two messages using CS1 and CS2
        let message1 = [UInt8]("yellowsubmarine".data(using: .utf8)!)
        let secureMessage1 = try initiatorsCipherState!.c1.encryptWithAD(ad: [], plaintext: message1)

        #expect(try respondersCipherState!.c1.decryptWithAD(ad: [], ciphertext: secureMessage1) == message1)

        messageLog.append("msg_\(msgIndex)_payload=\(message1.toHexString())")
        messageLog.append("msg_\(msgIndex)_ciphertext=\(secureMessage1.toHexString())")

        msgIndex += 1

        // Encrypt and Decrypt the second message `submarineyellow` using our shared second CipherState
        let message2 = [UInt8]("submarineyellow".data(using: .utf8)!)
        let secureMessage2 = try respondersCipherState!.c2.encryptWithAD(ad: [], plaintext: message2)

        #expect(try initiatorsCipherState!.c2.decryptWithAD(ad: [], ciphertext: secureMessage2) == message2)

        messageLog.append("msg_\(msgIndex)_payload=\(message2.toHexString())")
        messageLog.append("msg_\(msgIndex)_ciphertext=\(secureMessage2.toHexString())")

        // Assert that our Message Log == Test Vector String
        //print(messageLog.joined(separator: "\n"))

        return messageLog
    }

    internal static func createInitiator(
        _ str: String,
        initStatic: Bool,
        respStatic: Bool,
        prologue: [UInt8],
        messageLog: inout [String]
    ) throws -> Noise.HandshakeState {
        guard let cs = cipherSuiteFromString(str) else { throw Errors.unableToExtractCipherSuiteFromString }
        guard let hs = handshakeFromString(str) else { throw Errors.unableToExtractHandshakeFromString }

        //        if hs.hs.messagePattern.contains(where: { $0.messages.contains(where: { $0 == .s }) }) {
        //            messageLog.append("init_static=\(Array(initiatorsStatic.rawRepresentation).toHexString())")
        //            messageLog.append("resp_static=\(Array(respondersStatic.rawRepresentation).toHexString())")
        //        }
        if initStatic {
            messageLog.append("init_static=\(Array(initiatorsStatic.rawRepresentation).toHexString())")
        }
        if respStatic {
            messageLog.append("resp_static=\(Array(respondersStatic.rawRepresentation).toHexString())")
        }
        messageLog.append("gen_init_ephemeral=\(Array(initiatorsEphemeral.rawRepresentation).toHexString())")
        messageLog.append("gen_resp_ephemeral=\(Array(respondersEphemeral.rawRepresentation).toHexString())")

        if prologue.count > 0 {
            messageLog.append("prologue=\(prologue.toHexString())")
        }

        var psk: (key: [UInt8], placement: Int)? = nil
        if let placement = hs.psk {
            psk = (key: presharedKey, placement: placement)
            messageLog.append("preshared_key=\(presharedKey.toHexString())")
        }

        var remoteStatic: Curve25519.KeyAgreement.PublicKey? = nil
        if hs.hs.responderPreMessages.count > 0 {
            remoteStatic = respondersStatic.publicKey
        }

        let initiator = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: cs,
                    handshakePattern: hs.hs,
                    initiator: true,
                    prologue: prologue,
                    presharedKey: psk,
                    staticKeypair: initStatic ? initiatorsStatic : nil,
                    ephemeralKeypair: initiatorsEphemeral,
                    remoteStaticKeypair: remoteStatic
                )
        )

        messageLog.insert("handshake=\(initiator.protocolName)", at: 0)

        return initiator
    }

    internal static func createResponder(
        _ str: String,
        respStatic: Bool,
        prologue: [UInt8]
    ) throws -> Noise.HandshakeState {
        guard let cs = cipherSuiteFromString(str) else { throw Errors.unableToExtractCipherSuiteFromString }
        guard let hs = handshakeFromString(str) else { throw Errors.unableToExtractHandshakeFromString }

        var psk: (key: [UInt8], placement: Int)? = nil
        if let placement = hs.psk {
            psk = (key: presharedKey, placement: placement)
        }

        var remoteStatic: Curve25519.KeyAgreement.PublicKey? = nil
        if hs.hs.initiatorPreMessages.count > 0 {
            remoteStatic = initiatorsStatic.publicKey
        }

        let responder = try Noise.HandshakeState(
            config:
                Noise.Config(
                    cipherSuite: cs,
                    handshakePattern: hs.hs,
                    initiator: false,
                    prologue: prologue,
                    presharedKey: psk,
                    staticKeypair: respStatic ? respondersStatic : nil,
                    ephemeralKeypair: respondersEphemeral,
                    remoteStaticKeypair: remoteStatic
                )
        )
        return responder
    }

    internal static func cipherSuiteFromString(_ str: String) -> Noise.CipherSuite? {
        let parts = str.split(separator: "_")
        guard parts.count == 5 else { return nil }
        let hf: Noise.NoiseHashFunction
        if parts[4] == "SHA256" {
            hf = .sha256
        } else if parts[4] == "SHA512" {
            hf = .sha512
        } else {
            return nil
        }
        let ci: Noise.NoiseCipherAlgorithm
        if parts[3] == "ChaChaPoly" {
            ci = .ChaChaPoly1305
        } else if parts[3] == "AESGCM" {
            ci = .AESGCM
        } else {
            return nil
        }

        return Noise.CipherSuite(keyCurve: .ed25519, cipher: ci, hashFunction: hf)
    }

    internal static func handshakeFromString(_ str: String) -> (hs: Noise.Handshakes.Handshake, psk: Int?)? {
        let parts = str.split(separator: "_")
        guard parts.count == 5 else { return nil }

        var hs = parts[1]
        var psk: Int? = nil

        if hs.contains("psk") {
            psk = Int("\(hs.last!)")
            hs.removeLast(4)
        }

        guard let shake = Noise.Handshakes.allHandshakes.first(where: { $0.name == hs }) else {
            return nil
        }

        return (hs: shake, psk: psk)
    }

    public enum Errors: Error {
        case unableToExtractCipherSuiteFromString
        case unableToExtractHandshakeFromString
    }
}

//  CryptoSwift
//
//  Copyright (C) 2014-2017 Marcin Krzyżanowski <marcin@krzyzanowskim.com>
//  This software is provided 'as-is', without any express or implied warranty.
//
//  In no event will the authors be held liable for any damages arising from the use of this software.
//
//  Permission is granted to anyone to use this software for any purpose,including commercial applications, and to alter it and redistribute it freely, subject to the following restrictions:
//
//  - The origin of this software must not be misrepresented; you must not claim that you wrote the original software. If you use this software in a product, an acknowledgment in the product documentation is required.
//  - Altered source versions must be plainly marked as such, and must not be misrepresented as being the original software.
//  - This notice may not be removed or altered from any source or binary distribution.
extension Array {
    init(reserveCapacity: Int) {
        self = [Element]()
        self.reserveCapacity(reserveCapacity)
    }

    var slice: ArraySlice<Element> {
        self[self.startIndex..<self.endIndex]
    }
}

extension Array where Element == UInt8 {
    public init(hex: String) {
        self.init(reserveCapacity: hex.unicodeScalars.lazy.underestimatedCount)
        var buffer: UInt8?
        var skip = hex.hasPrefix("0x") ? 2 : 0
        for char in hex.unicodeScalars.lazy {
            guard skip == 0 else {
                skip -= 1
                continue
            }
            guard char.value >= 48 && char.value <= 102 else {
                removeAll()
                return
            }
            let v: UInt8
            let c: UInt8 = UInt8(char.value)
            switch c {
            case let c where c <= 57:
                v = c - 48
            case let c where c >= 65 && c <= 70:
                v = c - 55
            case let c where c >= 97:
                v = c - 87
            default:
                removeAll()
                return
            }
            if let b = buffer {
                append(b << 4 | v)
                buffer = nil
            } else {
                buffer = v
            }
        }
        if let b = buffer {
            append(b)
        }
    }

    public func toHexString() -> String {
        `lazy`.reduce(into: "") {
            var s = String($1, radix: 16)
            if s.count == 1 {
                s = "0" + s
            }
            $0 += s
        }
    }
}
