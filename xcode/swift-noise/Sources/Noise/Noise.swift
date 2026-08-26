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
//  A Noise Protocol handshake implementation

// TODO: remove the @preconcurrency tag once we drop support for swift 6.0
@preconcurrency import Crypto

internal protocol Handshake {
    var isInitiator: Bool { get }
    var remoteStatic: Curve25519.KeyAgreement.PublicKey? { get }
    var handshakePattern: Noise.Handshakes.Handshake { get }
}

public struct Noise {

    /// Max message length allowed by spec
    static let MaxMsgLen = 65_535

    /// Noise defined Message types
    public enum Message: Sendable {
        case s
        case e
        case ee
        case es
        case se
        case ss
        case psk
    }

    public enum MessagePattern: Sendable {
        case inbound([Message])
        case outbound([Message])

        internal var messages: [Message] {
            switch self {
            case .inbound(let messages):
                return messages
            case .outbound(let messages):
                return messages
            }
        }
    }

    public enum FundamentalHandshake: Handshake, Sendable {

        // Handshake NN
        case NN_Initiator
        case NN_Responder

        // Handshake KN
        case KN_Initiator
        case KN_Responder(remoteStatic: Curve25519.KeyAgreement.PublicKey)

        // Handshake NK
        case NK_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case NK_Responder

        // Handshake KK
        case KK_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case KK_Responder(remoteStatic: Curve25519.KeyAgreement.PublicKey)

        // Handshake NX
        case NX_Initiator
        case NX_Responder

        // Handshake KX
        case KX_Initiator
        case KX_Responder(remoteStatic: Curve25519.KeyAgreement.PublicKey)

        // Handshake XN
        case XN_Initiator
        case XN_Responder

        // Handshake IN
        case IN_Initiator
        case IN_Responder

        // Handshake XK
        case XK_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case XK_Responder

        // Handshake IK
        case IK_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case IK_Responder

        // Handshake XX
        case XX_Initiator
        case XX_Responder

        // Handshake IX
        case IX_Initiator
        case IX_Responder

        // Handshake N
        case N_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case N_Responder

        // Handshake K
        case K_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case K_Responder(remoteStatic: Curve25519.KeyAgreement.PublicKey)

        // Handshake X
        case X_Initiator(remoteStatic: Curve25519.KeyAgreement.PublicKey)
        case X_Responder

        internal var isInitiator: Bool {
            switch self {
            case .NN_Initiator, .KN_Initiator, .NK_Initiator, .KK_Initiator, .NX_Initiator, .KX_Initiator,
                .XN_Initiator, .IN_Initiator, .XK_Initiator, .IK_Initiator, .XX_Initiator, .IX_Initiator, .N_Initiator,
                .K_Initiator, .X_Initiator:
                return true
            default:
                return false
            }
        }

        internal var remoteStatic: Curve25519.KeyAgreement.PublicKey? {
            switch self {
            case .KN_Responder(let remoteStatic):
                return remoteStatic
            case .NK_Initiator(let remoteStatic):
                return remoteStatic
            case .KK_Initiator(let remoteStatic):
                return remoteStatic
            case .KK_Responder(let remoteStatic):
                return remoteStatic
            case .KX_Responder(let remoteStatic):
                return remoteStatic
            case .XK_Initiator(let remoteStatic):
                return remoteStatic
            case .IK_Initiator(let remoteStatic):
                return remoteStatic
            case .N_Initiator(let remoteStatic):
                return remoteStatic
            case .K_Initiator(let remoteStatic):
                return remoteStatic
            case .K_Responder(let remoteStatic):
                return remoteStatic
            case .X_Initiator(let remoteStatic):
                return remoteStatic
            default:
                return nil
            }
        }

        internal var handshakePattern: Handshakes.Handshake {
            switch self {
            case .NN_Initiator, .NN_Responder:
                return Handshakes.NN
            case .KN_Initiator, .KN_Responder:
                return Handshakes.KN
            case .NK_Initiator, .NK_Responder:
                return Handshakes.NK
            case .KK_Initiator, .KK_Responder:
                return Handshakes.KK
            case .NX_Initiator, .NX_Responder:
                return Handshakes.NX
            case .KX_Initiator, .KX_Responder:
                return Handshakes.KX
            case .XN_Initiator, .XN_Responder:
                return Handshakes.XN
            case .IN_Initiator, .IN_Responder:
                return Handshakes.IN
            case .XK_Initiator, .XK_Responder:
                return Handshakes.XK
            case .IK_Initiator, .IK_Responder:
                return Handshakes.IK
            case .XX_Initiator, .XX_Responder:
                return Handshakes.XX
            case .IX_Initiator, .IX_Responder:
                return Handshakes.IX
            case .N_Initiator, .N_Responder:
                return Handshakes.N
            case .K_Initiator, .K_Responder:
                return Handshakes.K
            case .X_Initiator, .X_Responder:
                return Handshakes.X
            }
        }
    }

    /// A set of pre-configured / defined Noise Handshake message patterns
    public struct Handshakes {
        public struct Handshake: Sendable {
            let name: String
            let messagePattern: [MessagePattern]
            let initiatorPreMessages: [Message]
            let responderPreMessages: [Message]

            init(
                name: String,
                messagePattern: [MessagePattern],
                initiatorPreMessages: [Message] = [],
                responderPreMessages: [Message] = []
            ) {
                self.name = name
                self.messagePattern = messagePattern
                self.initiatorPreMessages = initiatorPreMessages
                self.responderPreMessages = responderPreMessages
            }
        }

        /// XX Handshake Pattern
        /// - Note: Commonly used in LibP2P
        /// ```
        ///   // Message pattern / flow
        ///   -> e
        ///   <- e, ee, s, es
        ///   -> s, se
        /// ```
        static let XX: Handshake = Handshake(
            name: "XX",
            messagePattern: [
                .outbound([.e]),
                .inbound([.e, .ee, .s, .es]),
                .outbound([.s, .se]),
            ]
        )

        static let NN: Handshake = Handshake(
            name: "NN",
            messagePattern: [
                .outbound([.e]),
                .inbound([.e, .ee]),
            ]
        )

        static let NX: Handshake = Handshake(
            name: "NX",
            messagePattern: [
                .outbound([.e]),
                .inbound([.e, .ee, .s, .es]),
            ]
        )

        static let XN: Handshake = Handshake(
            name: "XN",
            messagePattern: [
                .outbound([.e]),
                .inbound([.e, .ee]),
                .outbound([.s, .se]),
            ]
        )

        static let IN: Handshake = Handshake(
            name: "IN",
            messagePattern: [
                .outbound([.e, .s]),
                .inbound([.e, .ee, .se]),
            ]
        )

        static let IX: Handshake = Handshake(
            name: "IX",
            messagePattern: [
                .outbound([.e, .s]),
                .inbound([.e, .ee, .se, .s, .es]),
            ]
        )

        /// Pre-messages...
        static let KN: Handshake = Handshake(
            name: "KN",
            messagePattern: [
                .outbound([.e]),
                .inbound([.e, .ee, .se]),
            ],
            initiatorPreMessages: [.s]
        )

        static let NK: Handshake = Handshake(
            name: "NK",
            messagePattern: [
                .outbound([.e, .es]),
                .inbound([.e, .ee]),
            ],
            responderPreMessages: [.s]
        )

        static let KK: Handshake = Handshake(
            name: "KK",
            messagePattern: [
                .outbound([.e, .es, .ss]),
                .inbound([.e, .ee, .se]),
            ],
            initiatorPreMessages: [.s],
            responderPreMessages: [.s]
        )

        static let KX: Handshake = Handshake(
            name: "KX",
            messagePattern: [
                .outbound([.e]),
                .inbound([.e, .ee, .se, .s, .es]),
            ],
            initiatorPreMessages: [.s]
        )

        static let XK: Handshake = Handshake(
            name: "XK",
            messagePattern: [
                .outbound([.e, .es]),
                .inbound([.e, .ee]),
                .outbound([.s, .se]),
            ],
            responderPreMessages: [.s]
        )

        static let IK: Handshake = Handshake(
            name: "IK",
            messagePattern: [
                .outbound([.e, .es, .s, .ss]),
                .inbound([.e, .ee, .se]),
            ],
            responderPreMessages: [.s]
        )

        static let N: Handshake = Handshake(
            name: "N",
            messagePattern: [
                .outbound([.e, .es])
            ],
            responderPreMessages: [.s]
        )

        static let K: Handshake = Handshake(
            name: "K",
            messagePattern: [
                .outbound([.e, .es, .ss])
            ],
            initiatorPreMessages: [.s],
            responderPreMessages: [.s]
        )

        static let X: Handshake = Handshake(
            name: "X",
            messagePattern: [
                .outbound([.e, .es, .s, .ss])
            ],
            responderPreMessages: [.s]
        )

        static let allHandshakes: [Handshake] =
            [XX, NN, NX, XN, IN, IX, KN, NK, KK, KX, XK, IK, N, K, X]

    }

    public enum NoiseHashFunction: Sendable {
        case sha256
        //case sha384
        case sha512

        internal var hashLength: Int {
            switch self {
            case .sha256: return 32
            //case .sha384: return 48
            case .sha512: return 64
            }
        }

        internal var protocolName: String {
            switch self {
            case .sha256:
                return "SHA256"
            //case .sha384:
            //    return "SHA384"
            case .sha512:
                return "SHA512"
            }
        }

        internal func hash(data: [UInt8]) -> [UInt8] {
            switch self {
            case .sha256:
                return Array(SHA256.hash(data: data))
            //case .sha384:
            //    return Array(SHA384.hash(data: data))
            case .sha512:
                return Array(SHA512.hash(data: data))
            }
        }

        internal func HKDF(
            chainingKey: SymmetricKey,
            inputKeyMaterial: [UInt8],
            numOutputs: Int
        ) throws -> ([UInt8], [UInt8], [UInt8]?) {
            guard numOutputs == 2 || numOutputs == 3 else {
                throw Noise.Errors.custom("Invalid numOutputs specified. numOutputs must either be 2 or 3")
            }

            switch self {
            case .sha256:
                guard chainingKey.bitCount == 256 else {
                    throw Noise.Errors.custom(
                        "ChainingKey is expected to be 32 Bytes in length, but is \(chainingKey.bitCount) bits instead"
                    )
                }
                return try hkdf(
                    chainingKey: chainingKey,
                    inputKeyMaterial: inputKeyMaterial,
                    numOutputs: numOutputs,
                    usingHashFunction: SHA256.self
                )

            //case .sha384:
            //    return try hkdf(chainingKey: chainingKey, inputKeyMaterial: inputKeyMaterial, numOutputs: numOutputs, usingHashFunction: SHA384.self)

            case .sha512:
                guard chainingKey.bitCount == 512 else {
                    throw Noise.Errors.custom(
                        "ChainingKey is expected to be 64 Bytes in length, but is \(chainingKey.bitCount) bits instead"
                    )
                }
                return try hkdf(
                    chainingKey: chainingKey,
                    inputKeyMaterial: inputKeyMaterial,
                    numOutputs: numOutputs,
                    usingHashFunction: SHA512.self
                )
            }
        }

        /// An OSX 10.X compatible implementation of the HMAC-based Extract-and-Expand Key Derivation Function (HKDF)
        /// - Note: ChainingKey is expected to be 32 Bytes in length
        /// - Note: [Reference](https://tools.ietf.org/html/rfc5869)
        /// - Note: We instantiate a new HMAC instance for each expansion, instead of calling update() multiple times. We do this because the results are not equal and the reference doc seems to specify the current behavior
        private func hkdf<H: HashFunction>(
            chainingKey: SymmetricKey,
            inputKeyMaterial: [UInt8],
            numOutputs: Int,
            usingHashFunction: H.Type
        ) throws -> ([UInt8], [UInt8], [UInt8]?) {
            var hmac = HMAC<H>(key: chainingKey)
            hmac.update(data: inputKeyMaterial)
            let tempKey = SymmetricKey(data: hmac.finalize())

            var hmac1 = HMAC<H>(key: tempKey)
            hmac1.update(data: [0x01])
            let output1 = [UInt8](hmac1.finalize())

            var hmac2 = HMAC<H>(key: tempKey)
            hmac2.update(data: output1 + [0x02])
            let output2 = [UInt8](hmac2.finalize())

            if numOutputs == 2 {
                return (output1, output2, nil)
            }

            var hmac3 = HMAC<H>(key: tempKey)
            hmac3.update(data: output2 + [0x03])
            let output3 = [UInt8](hmac3.finalize())

            return (output1, output2, output3)
        }
    }

    public enum NoiseCipherAlgorithm: Sendable {
        case ChaChaPoly1305
        case AESGCM

        internal var protocolName: String {
            switch self {
            case .ChaChaPoly1305:
                return "ChaChaPoly"
            case .AESGCM:
                return "AESGCM"
            }
        }

        internal func encrypt(
            plaintext: [UInt8],
            usingKey symKey: SymmetricKey,
            nonce: UInt64,
            withAuthenticatingData ad: [UInt8]
        ) throws -> [UInt8] {
            switch self {
            case .ChaChaPoly1305:
                // Encrypt the plaintext using our sym key, nonce, and authenticating data
                let enc = try ChaChaPoly.seal(
                    plaintext,
                    using: symKey,
                    nonce: getChaChaPolyNonce(nonce),
                    authenticating: ad
                )

                return [UInt8](enc.ciphertext + enc.tag)

            case .AESGCM:
                let enc = try AES.GCM.seal(plaintext, using: symKey, nonce: getAESGCMNonce(nonce), authenticating: ad)

                return [UInt8](enc.ciphertext + enc.tag)
            }
        }

        internal func decrypt(
            ciphertext: [UInt8],
            usingKey symKey: SymmetricKey,
            nonce: UInt64,
            withAuthenticatingData ad: [UInt8]
        ) throws -> [UInt8] {
            guard ciphertext.count >= 16 else {
                throw Noise.Errors.custom("Invalid ciphertext length (no tag data found)")
            }

            switch self {
            case .ChaChaPoly1305:
                // Init a ChaChaPoly sealed box using our nonce, the cipher text and the cipher tag (last 16 bytes)
                let sealedBox = try ChaChaPoly.SealedBox(
                    nonce: getChaChaPolyNonce(nonce),
                    ciphertext: ciphertext.dropLast(16),
                    tag: ciphertext.suffix(16)
                )

                // Decrypt the sealed box using our sym key, and authenticating data
                let plaintext = try ChaChaPoly.open(sealedBox, using: symKey, authenticating: ad)

                return [UInt8](plaintext)

            case .AESGCM:
                let sealedBox = try AES.GCM.SealedBox(
                    nonce: getAESGCMNonce(nonce),
                    ciphertext: ciphertext.dropLast(16),
                    tag: ciphertext.suffix(16)
                )

                let plaintext = try AES.GCM.open(sealedBox, using: symKey, authenticating: ad)

                return [UInt8](plaintext)
            }
        }

        /// Returns the nonce as a UInt32 Little Endian byte array zero padded to 12 bytes
        private func getChaChaPolyNonce(_ n: UInt64) throws -> ChaChaPoly.Nonce {
            let padding: [UInt8] = [0x00, 0x00, 0x00, 0x00]
            let littleBytes = n.littleEndianBytes  // n.littleEndian.toBytes

            return try ChaChaPoly.Nonce(data: padding + littleBytes)
        }

        /// Returns the nonce as a UInt32 Big Endian byte array zero padded to 12 bytes
        private func getAESGCMNonce(_ n: UInt64) throws -> AES.GCM.Nonce {
            let padding: [UInt8] = [0x00, 0x00, 0x00, 0x00]
            let bigBytes = n.bigEndianBytes  // n.bigEndian.toBytes

            return try AES.GCM.Nonce(data: padding + bigBytes)
        }
    }

    public enum NoiseKeypairCurve: Sendable {
        case ed25519

        internal var protocolName: String {
            switch self {
            case .ed25519:
                return "25519"
            }
        }
    }

    public struct CipherSuite: Sendable {
        let keyCurve: NoiseKeypairCurve
        let cipher: NoiseCipherAlgorithm
        let hashFunction: NoiseHashFunction

        public init(keyCurve: NoiseKeypairCurve, cipher: NoiseCipherAlgorithm, hashFunction: NoiseHashFunction) {
            self.keyCurve = keyCurve
            self.cipher = cipher
            self.hashFunction = hashFunction
        }

        internal var protocolName: String {
            "\(keyCurve.protocolName)_\(cipher.protocolName)_\(hashFunction.protocolName)"
        }
    }

    public struct Config: Sendable {
        let cipherSuite: CipherSuite
        let handshakePattern: Handshakes.Handshake
        let initiator: Bool
        let prologue: [UInt8]
        let presharedKey: (key: [UInt8], placement: Int)?
        let staticKeypair: Curve25519.KeyAgreement.PrivateKey?
        let ephemeralKeypair: Curve25519.KeyAgreement.PrivateKey?
        let remoteStaticKeypair: Curve25519.KeyAgreement.PublicKey?
        let remoteEphemeralKeypair: Curve25519.KeyAgreement.PublicKey?

        public init(
            cipherSuite: CipherSuite,
            handshakePattern: Handshakes.Handshake,
            initiator: Bool,
            prologue: [UInt8] = [],
            presharedKey: (key: [UInt8], placement: Int)? = nil,
            staticKeypair: Curve25519.KeyAgreement.PrivateKey? = nil,
            ephemeralKeypair: Curve25519.KeyAgreement.PrivateKey? = nil,
            remoteStaticKeypair: Curve25519.KeyAgreement.PublicKey? = nil
        ) {
            self.cipherSuite = cipherSuite
            self.handshakePattern = handshakePattern
            self.initiator = initiator
            self.prologue = prologue
            self.presharedKey = presharedKey
            self.staticKeypair = staticKeypair
            self.ephemeralKeypair = ephemeralKeypair
            self.remoteStaticKeypair = remoteStaticKeypair
            self.remoteEphemeralKeypair = nil
        }

        public init(
            cipherSuite: CipherSuite,
            handshake: FundamentalHandshake,
            prologue: [UInt8] = [],
            presharedKey: (key: [UInt8], placement: Int)? = nil,
            staticKeypair: Curve25519.KeyAgreement.PrivateKey? = nil,
            ephemeralKeypair: Curve25519.KeyAgreement.PrivateKey? = nil
        ) {
            self.cipherSuite = cipherSuite
            self.handshakePattern = handshake.handshakePattern
            self.initiator = handshake.isInitiator
            self.prologue = prologue
            self.presharedKey = presharedKey
            self.staticKeypair = staticKeypair
            self.ephemeralKeypair = ephemeralKeypair
            self.remoteStaticKeypair = handshake.remoteStatic
            self.remoteEphemeralKeypair = nil
        }
    }

    public enum Errors: Error, Equatable, Sendable {
        case invalidPSK
        case remoteEphemeralKeyAlreadySet
        case remoteStaticKeyAlreadySet
        case unexpectedPayloadLength
        case invalidProtocolName
        case invalidChainingKey
        case invalidHKDFOutput
        case unsupportedPreMessage
        case custom(String)
    }

    /// A HandshakeState object contains a `SymmetricState` plus DH variables (`s`, `e`, `rs`, `re`) and a variable representing the handshake pattern.
    /// - Note: During the handshake phase each party has a single HandshakeState, which can be deleted once the handshake is finished.
    /// - WARNING: HandshakeState is not thread safe, it's designed to be used within a single thread such as a NIO Channel
    public final class HandshakeState: @unchecked Sendable {
        private let symmetricState: SymmetricState

        private var s: Curve25519.KeyAgreement.PrivateKey?  //Our Local Libp2p Keys
        private var e: Curve25519.KeyAgreement.PrivateKey?  //Our Local Noise Ephemeral Keys
        private var rs: Curve25519.KeyAgreement.PublicKey?  //Remote Peer Libp2p Public Key
        private var re: Curve25519.KeyAgreement.PublicKey?  //Remote Peer Noise Ephemeral PubKey
        private var psk: [UInt8] = []

        let initiator: Bool
        private var messagePattern: [MessagePattern]
        private let prologue: [UInt8]

        private var _msgIndex: Int = 0
        public var msgIndex: Int {
            _msgIndex
        }

        public let protocolName: String

        public init(config: Config) throws {
            /// Sets message_patterns to the message patterns from handshake_pattern.
            self.messagePattern = config.handshakePattern.messagePattern
            self.prologue = config.prologue

            /// Sets the initiator, s, e, rs, and re variables to the corresponding arguments.
            self.initiator = config.initiator
            self.s = config.staticKeypair
            self.e = config.ephemeralKeypair
            self.rs = config.remoteStaticKeypair
            self.re = config.remoteEphemeralKeypair

            /// Handle Pre Shared Key if one was provided
            var pskModifier = ""
            if let psk = config.presharedKey {
                self.psk = psk.key
                guard psk.key.count == 32 else {
                    throw Noise.Errors.invalidPSK
                }
                pskModifier = "psk\(psk.placement)"

                if psk.placement == 0 {
                    switch messagePattern[0] {
                    case .inbound(let messages):
                        messagePattern[0] = .inbound([.psk] + messages)
                    case .outbound(let messages):
                        messagePattern[0] = .outbound([.psk] + messages)
                    }
                } else {
                    guard messagePattern.count > (psk.placement - 1) else {
                        throw Errors.custom("Invalid presharedKey placement")
                    }
                    switch messagePattern[psk.placement - 1] {
                    case .inbound(let messages):
                        messagePattern[psk.placement - 1] = .inbound(messages + [.psk])
                    case .outbound(let messages):
                        messagePattern[psk.placement - 1] = .outbound(messages + [.psk])
                    }
                }
            }

            self.protocolName =
                "Noise_" + config.handshakePattern.name + pskModifier + "_" + config.cipherSuite.protocolName

            /// Calls InitializeSymmetric(protocol_name)
            self.symmetricState = try SymmetricState(protocolName: protocolName, cipherSuite: config.cipherSuite)

            /// Calls MixHash(prologue)
            self.symmetricState.mixHash(data: prologue)

            for preMessage in config.handshakePattern.initiatorPreMessages {
                switch preMessage {
                case .s:
                    if initiator {
                        guard let s = self.s else {
                            throw Noise.Errors.custom("Initiator PreMessage: Invalid local static key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](s.publicKey.rawRepresentation))
                    } else {
                        guard let rs = self.rs else {
                            throw Noise.Errors.custom("Responder PreMessage: Invalid remote static key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](rs.rawRepresentation))
                    }

                case .e:
                    if initiator {
                        guard let e = self.e else {
                            throw Noise.Errors.custom("Initiator PreMessage: Invalid local ephemeral key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](e.publicKey.rawRepresentation))
                    } else {
                        guard let re = self.re else {
                            throw Noise.Errors.custom("Responder PreMessage: Invalid remote ephemeral key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](re.rawRepresentation))
                    }

                default:
                    throw Noise.Errors.unsupportedPreMessage
                }
            }

            for preMessage in config.handshakePattern.responderPreMessages {
                switch preMessage {
                case .s:
                    if !initiator {
                        guard let s = self.s else {
                            throw Noise.Errors.custom("Responder PreMessage: Invalid local static key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](s.publicKey.rawRepresentation))
                    } else {
                        guard let rs = self.rs else {
                            throw Noise.Errors.custom("Initiator PreMessage: Invalid remote static key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](rs.rawRepresentation))
                    }

                case .e:
                    if !initiator {
                        guard let e = self.e else {
                            throw Noise.Errors.custom("Responder PreMessage: Invalid local ephemeral key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](e.publicKey.rawRepresentation))
                    } else {
                        guard let re = self.re else {
                            throw Noise.Errors.custom("Initiator PreMessage: Invalid remote ephemeral key")
                        }
                        self.symmetricState.mixHash(data: [UInt8](re.rawRepresentation))
                    }

                default:
                    throw Noise.Errors.unsupportedPreMessage
                }
            }
        }

        /// Takes a payload byte sequence which may be zero-length, and a message_buffer to write the output into
        /// - Note: This method aborts if any EncryptAndHash() call returns an error
        public func writeMessage(payload: [UInt8]) throws -> (buffer: [UInt8], c1: CipherState?, c2: CipherState?) {

            guard self.shouldWrite() else {
                throw Noise.Errors.custom("noise: unexpected call to WriteMessage should be ReadMessage")
            }
            guard _msgIndex < messagePattern.count else {
                throw Noise.Errors.custom("noise: no handshake messages left")
            }
            guard payload.count < Noise.MaxMsgLen else {
                throw Noise.Errors.custom("noise: message is too long")
            }

            // Get the next set of messages to process...
            let pattern = messagePattern[_msgIndex].messages

            var messageBuffer: [UInt8] = []

            // Fetches and deletes the next message pattern from message_patterns, then sequentially processes each token from the message pattern:
            for message in pattern {
                switch message {
                case .e:
                    // For "e": Sets e (which must be empty) to GENERATE_KEYPAIR(). Appends e.public_key to the buffer. Calls MixHash(e.public_key).
                    if e == nil { e = generateKeypair() }
                    //else { print("Warning: e already set, this is only acceptable during testing") }
                    //messageBuffer.writeBytes(e!.publicKey.rawRepresentation)
                    messageBuffer.append(contentsOf: e!.publicKey.rawRepresentation)
                    symmetricState.mixHash(data: [UInt8](e!.publicKey.rawRepresentation))
                    if psk.count > 0 {
                        try symmetricState.mixKey(inputKeyMaterial: [UInt8](e!.publicKey.rawRepresentation))
                    }

                case .s:
                    // For "s": Appends EncryptAndHash(s.public_key) to the buffer.
                    guard let s = s else {
                        throw Noise.Errors.custom("Op 's': Local Static Key isn't available. Aborting")
                    }
                    let spk = try symmetricState.encryptAndHash(plaintext: [UInt8](s.publicKey.rawRepresentation))
                    messageBuffer.append(contentsOf: spk)
                //messageBuffer.writeBytes(spk)

                case .ee:
                    // For "ee": Calls MixKey(DH(e, re)).
                    guard let e = e, let re = re else {
                        throw Noise.Errors.custom(
                            "Op 'ee': Local and/or Remote Ephemeral Keys aren't available. Aborting"
                        )
                    }
                    try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: e, pubKey: re))

                case .es:
                    // For "es": Calls MixKey(DH(e, rs)) if initiator, MixKey(DH(s, re)) if responder.
                    if initiator {
                        guard let e = e, let rs = rs else {
                            throw Noise.Errors.custom(
                                "Op 'es': Local Ephemeral and/or Remote Static Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: e, pubKey: rs))
                    } else {
                        guard let s = s, let re = re else {
                            throw Noise.Errors.custom(
                                "Op 'es': Local Static and/or Remote Ephemeral Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: s, pubKey: re))
                    }

                case .se:
                    // For "se": Calls MixKey(DH(s, re)) if initiator, MixKey(DH(e, rs)) if responder.
                    if initiator {
                        guard let s = s, let re = re else {
                            throw Noise.Errors.custom(
                                "Op 'se': Local Static and/or Remote Ephemeral Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: s, pubKey: re))
                    } else {
                        guard let e = e, let rs = rs else {
                            throw Noise.Errors.custom(
                                "Op 'se': Local Ephemeral and/or Remote Static Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: e, pubKey: rs))
                    }

                case .ss:
                    // For "ss": Calls MixKey(DH(s, rs)).
                    guard let s = s, let rs = rs else {
                        throw Noise.Errors.custom(
                            "Op 'ss': Local Static and/or Remote Static Keys aren't available. Aborting"
                        )
                    }
                    try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: s, pubKey: rs))

                case .psk:
                    guard psk.count == 32 else { throw Noise.Errors.invalidPSK }
                    try symmetricState.mixKeyAndHash(inputKeyMaterial: psk)

                }
            }

            // Increment our message index counter
            _msgIndex += 1

            // Appends EncryptAndHash(payload) to the buffer.
            try messageBuffer.append(contentsOf: symmetricState.encryptAndHash(plaintext: payload))
            //try messageBuffer.writeBytes( symmetricState.encryptAndHash(plaintext: payload) )

            // If there are no more message patterns returns two new CipherState objects by calling Split().
            if _msgIndex >= messagePattern.count {
                let split = try symmetricState.split()
                return (buffer: messageBuffer, c1: split.c1, c2: split.c2)
            }

            return (buffer: messageBuffer, c1: nil, c2: nil)
        }

        /// Takes a byte sequence containing a Noise handshake message, and a payload_buffer to write the message's plaintext payload into
        /// - Note: This method aborts if any DecryptAndHash() call returns an error
        public func readMessage(
            _ inboundMessage: [UInt8]
        ) throws -> (payload: [UInt8], c1: CipherState?, c2: CipherState?) {
            //public func readMessage(_ inboundMessage:ByteBuffer) throws -> (payload:ByteBuffer, c1:CipherState?, c2:CipherState?) {

            guard self.shouldRead() else {
                throw Noise.Errors.custom("noise: unexpected call to ReadMessage should be WriteMessage")
            }
            guard _msgIndex < messagePattern.count else {
                throw Noise.Errors.custom("noise: no handshake messages left")
            }

            // TODO: rsSet = false
            // TODO: ss.checkpoint()
            symmetricState.checkpoint()

            // Get the next set of messages to process...
            let pattern = messagePattern[_msgIndex].messages

            var inboundMsg: [UInt8] = inboundMessage  //Array(inboundMessage.readableBytesView)
            var bytesRead: Int = 0

            // Fetches and deletes the next message pattern from message_patterns, then sequentially processes each token from the message pattern
            for message in pattern {
                //print("Consuming message \(message)")
                switch message {
                case .e, .s:
                    var expected: Int = 32
                    if message == .s && symmetricState.cipherState.hasKey() {
                        expected += 16
                    }
                    guard inboundMsg.count >= expected else { throw Noise.Errors.custom("Err msg too short") }

                    do {
                        if message == .e {
                            // For "e": Sets re (which must be empty) to the next DHLEN bytes from the message. Calls MixHash(re.public_key).
                            guard re == nil else { throw Noise.Errors.remoteEphemeralKeyAlreadySet }
                            //guard inboundMsg.count >= symmetricState.HASHLEN else { throw Noise.Errors.custom("Message payload unexpected length") }
                            re = try Curve25519.KeyAgreement.PublicKey(rawRepresentation: inboundMsg.prefix(expected))
                            symmetricState.mixHash(data: [UInt8](re!.rawRepresentation))
                            bytesRead += expected
                            if psk.count > 0 {
                                try symmetricState.mixKey(inputKeyMaterial: [UInt8](re!.rawRepresentation))
                            }

                        } else if message == .s {
                            // For "s": Sets temp to the next DHLEN + 16 bytes of the message if HasKey() == True, or to the next DHLEN bytes otherwise. Sets rs (which must be empty) to DecryptAndHash(temp).
                            guard rs == nil else {
                                throw Noise.Errors.custom("Remote static key has previously been set. Aborting")
                            }
                            rs = try Curve25519.KeyAgreement.PublicKey(
                                rawRepresentation: symmetricState.decryptAndHash(
                                    ciphertext: Array(inboundMsg.prefix(expected))
                                )
                            )
                            bytesRead += expected

                        }
                    } catch {
                        symmetricState.rollback()
                        // if rSet { rs = nil }
                        throw error
                    }
                    inboundMsg = Array(inboundMsg.dropFirst(expected))

                case .ee:
                    // For "ee": Calls MixKey(DH(e, re)).
                    guard let e = e, let re = re else {
                        throw Noise.Errors.custom(
                            "Op 'ee': Local and/or Remote Ephermeral Keys aren't available. Aborting"
                        )
                    }
                    try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: e, pubKey: re))

                case .es:
                    // For "es": Calls MixKey(DH(e, rs)) if initiator, MixKey(DH(s, re)) if responder.
                    if initiator {
                        guard let e = e, let rs = rs else {
                            throw Noise.Errors.custom(
                                "Op 'es': Local Ephemeral and/or Remote Static Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: e, pubKey: rs))
                    } else {
                        guard let s = s, let re = re else {
                            throw Noise.Errors.custom(
                                "Op 'es': Local Static and/or Remote Ephemeral Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: s, pubKey: re))
                    }

                case .se:
                    // For "se": Calls MixKey(DH(s, re)) if initiator, MixKey(DH(e, rs)) if responder.
                    if initiator {
                        guard let s = s, let re = re else {
                            throw Noise.Errors.custom(
                                "Op 'se': Local Static and/or Remote Ephemeral Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: s, pubKey: re))
                    } else {
                        guard let e = e, let rs = rs else {
                            throw Noise.Errors.custom(
                                "Op 'se': Local Ephemeral and/or Remote Static Keys aren't available. Aborting"
                            )
                        }
                        try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: e, pubKey: rs))
                    }

                case .ss:
                    // For "ss": Calls MixKey(DH(s, rs)).
                    guard let s = s, let rs = rs else {
                        throw Noise.Errors.custom(
                            "Op 'ss': Local Static and/or Remote Static Keys aren't available. Aborting"
                        )
                    }
                    try symmetricState.mixKey(inputKeyMaterial: dh(keyPair: s, pubKey: rs))

                case .psk:
                    guard psk.count == 32 else { throw Noise.Errors.invalidPSK }
                    try symmetricState.mixKeyAndHash(inputKeyMaterial: psk)

                }
            }

            var decryptedPayload: [UInt8] = []
            //var decryptedPayload = ByteBuffer()

            do {

                // Calls DecryptAndHash() on the remaining bytes of the message and stores the output into payload_buffer.
                //decryptedPayload.writeBytes( try symmetricState.decryptAndHash(ciphertext: inboundMsg ) )
                decryptedPayload.append(contentsOf: try symmetricState.decryptAndHash(ciphertext: inboundMsg))
            } catch {

                // Rollback
                symmetricState.rollback()
                rs = nil
                throw error

            }

            _msgIndex += 1

            // If there are no more message patterns returns two new CipherState objects by calling Split().
            if _msgIndex >= messagePattern.count {
                let split = try symmetricState.split()
                return (payload: decryptedPayload, c1: split.c1, c2: split.c2)
            }

            return (payload: decryptedPayload, c1: nil, c2: nil)
        }

        public func shouldRead() -> Bool {
            !shouldWrite()
        }

        public func shouldWrite() -> Bool {
            guard messagePattern.count > _msgIndex else { return false }
            let msg = messagePattern[_msgIndex]
            switch msg {
            case .inbound:
                return self.initiator != true
            case .outbound:
                return self.initiator == true
            }
        }

        private func generateKeypair() -> Curve25519.KeyAgreement.PrivateKey {
            Curve25519.KeyAgreement.PrivateKey()
        }

        private func dh(
            keyPair: Curve25519.KeyAgreement.PrivateKey,
            pubKey: Curve25519.KeyAgreement.PublicKey
        ) throws -> [UInt8] {
            let shared = try keyPair.sharedSecretFromKeyAgreement(with: pubKey)
            return shared.withUnsafeBytes { Array($0) }
        }

        public func encrypt(msg: [UInt8]) throws -> [UInt8] {
            try symmetricState.encryptAndHash(plaintext: msg)
        }

        public func decrypt(msg: [UInt8]) throws -> [UInt8] {
            try symmetricState.decryptAndHash(ciphertext: msg)
        }

        /// ChannelBinding provides a value that uniquely identifies the session and can
        /// be used as a channel binding. It is an error to call this method before the
        /// handshake is complete.
        public func channelBinding() -> [UInt8] {
            symmetricState.h
        }

        /// PeerStatic returns the static key provided by the remote peer during
        /// a handshake. It is an error to call this method if a handshake message
        /// containing a static key has not been read.
        public func peerStatic() throws -> Curve25519.KeyAgreement.PublicKey {
            guard let rs = rs else { throw Noise.Errors.custom("Peer Static Key not set yet") }
            return rs
        }

        /// PeerEphemeral returns the ephemeral key provided by the remote peer during
        /// a handshake. It is an error to call this method if a handshake message
        /// containing a static key has not been read.
        public func peerEphemeral() throws -> Curve25519.KeyAgreement.PublicKey {
            guard let re = re else { throw Noise.Errors.custom("Peer Ephemeral Key not set yet") }
            return re
        }

        /// LocalEphemeral returns the local ephemeral key pair generated during a handshake.
        public func localEphemeral() throws -> Curve25519.KeyAgreement.PrivateKey {
            guard let e = e else { throw Noise.Errors.custom("Local Ephemeral KeyPair not set yet") }
            return e
        }

        /// MessageIndex returns the current handshake message id
        public func messageIndex() -> Int {
            _msgIndex
        }

    }

    /// A SymmetricState object contains a CipherState plus `ck` and `h` variables.
    /// - Note: It is so-named because it encapsulates all the "symmetric crypto" used by Noise.
    /// - Note: During the handshake phase each party has a single SymmetricState, which can be deleted once the handshake is finished.
    /// - WARNING: HandshakeState is not thread safe, it's designed to be used within a single thread such as a NIO Channel
    internal final class SymmetricState: @unchecked Sendable {
        private let hashFunction: NoiseHashFunction
        private let cipher: NoiseCipherAlgorithm

        let HASHLEN: Int
        let cipherState: CipherState

        /// A chaining key of `HASHLEN` bytes.
        var ck: SymmetricKey

        /// A hash output of `HASHLEN` bytes
        var h: [UInt8]

        private var previousCK: SymmetricKey
        private var previousH: [UInt8]

        /// Takes an arbitrary-length protocol_name byte sequence (see Section 8).
        ///
        /// Executes the following steps:
        /// ```
        /// 1) if protocol_name is less than or equal to HASHLEN bytes in length
        ///      sets h equal to protocol_name with zero bytes appended to make HASHLEN bytes.
        ///    else
        ///      Otherwise sets h = HASH(protocol_name).
        ///
        /// 2) Sets ck = h
        ///
        /// 3) Calls InitializeKey(empty)
        /// ```
        init(protocolName: String, cipherSuite: CipherSuite) throws {
            //var buf:ByteBuffer
            //buf.writeString(protocolName)
            guard var proto = protocolName.data(using: .utf8) else { throw Noise.Errors.invalidProtocolName }

            hashFunction = cipherSuite.hashFunction
            cipher = cipherSuite.cipher

            HASHLEN = hashFunction.hashLength

            if proto.count <= HASHLEN {
                while proto.count < HASHLEN { proto.append(0) }
                h = [UInt8](proto)
            } else {
                h = hashFunction.hash(data: [UInt8](proto))
            }

            //if h.count > 32 { print("Using first 32 bytes of H") }
            //ck = SymmetricKey(data: h.prefix(32))
            ck = SymmetricKey(data: h)

            //Used for checkpoints and rollbacks
            previousCK = ck
            previousH = h

            cipherState = try CipherState(cipher: cipher, key: nil)
        }

        func mixKey(inputKeyMaterial: [UInt8]) throws {
            let (newCK, tempK, _) = try hashFunction.HKDF(
                chainingKey: ck,
                inputKeyMaterial: inputKeyMaterial,
                numOutputs: 2
            )

            //if newCK.count > 32 { print("Warning! Using first 32 bytes of newCK") }
            //ck = SymmetricKey(data: newCK.prefix(32))
            ck = SymmetricKey(data: newCK)

            if HASHLEN == 64 {
                // If HASHLEN is 64, then truncates temp_k to 32 bytes.
                try cipherState.initializeKey(key: Array(tempK.prefix(32)))
            } else {
                try cipherState.initializeKey(key: tempK)
            }
        }

        /// Sets h = HASH(h || data)
        func mixHash(data: [UInt8]) {
            h = hashFunction.hash(data: h + data)
        }

        /// This function is used for handling pre-shared symmetric keys, as described in [section 9](https://noiseprotocol.org/noise.html#pre-shared-symmetric-keys)
        func mixKeyAndHash(inputKeyMaterial: [UInt8]) throws {
            // Sets ck, temp_h, temp_k = HKDF(ck, input_key_material, 3).
            let (newCK, tempH, tempK) = try hashFunction.HKDF(
                chainingKey: ck,
                inputKeyMaterial: inputKeyMaterial,
                numOutputs: 3
            )

            //if newCK.count > 32 { print("Warning! Using first 32 bytes of newCK") }
            //ck = SymmetricKey(data: newCK.prefix(32))
            ck = SymmetricKey(data: newCK)

            // Calls MixHash(temp_h)
            mixHash(data: tempH)

            guard let tk = tempK else { throw Noise.Errors.custom("Failed to generate 3 outputs, tempK is nil") }
            if HASHLEN == 64 {
                // If HASHLEN is 64, then truncates temp_k to 32 bytes.
                try cipherState.initializeKey(key: Array(tk.prefix(32)))
            } else {
                try cipherState.initializeKey(key: tk)
            }
        }

        /// Returns h.
        /// - Note: This function should only be called at the end of a handshake, i.e. after the Split() function has been called.
        /// - Note: This function is used for channel binding, as described in Section [11.2](https://noiseprotocol.org/noise.html#channel-binding)
        func getHandshakeHash() -> [UInt8] {
            h
        }

        /// Sets ciphertext = EncryptWithAd(h, plaintext), calls MixHash(ciphertext), and returns ciphertext.
        /// - Note: If k is empty, the EncryptWithAd() call will set ciphertext equal to plaintext.
        func encryptAndHash(plaintext: [UInt8]) throws -> [UInt8] {
            let cipherText = try cipherState.encryptWithAD(ad: h, plaintext: plaintext)
            mixHash(data: cipherText)
            return cipherText
        }

        /// Sets plaintext = DecryptWithAd(h, ciphertext), calls MixHash(ciphertext), and returns plaintext.
        /// - Note: If k is empty, the DecryptWithAd() call will set plaintext equal to ciphertext.
        func decryptAndHash(ciphertext: [UInt8]) throws -> [UInt8] {
            let plaintext = try cipherState.decryptWithAD(ad: h, ciphertext: ciphertext)
            mixHash(data: ciphertext)
            return plaintext
        }

        /// Returns a pair of CipherState objects for encrypting transport messages
        func split() throws -> (c1: CipherState, c2: CipherState) {
            var (tempK1, tempK2, _) = try hashFunction.HKDF(chainingKey: ck, inputKeyMaterial: [], numOutputs: 2)

            if HASHLEN == 64 {
                tempK1 = Array(tempK1.prefix(32))
                tempK2 = Array(tempK2.prefix(32))
            }

            let c1 = try CipherState(cipher: cipher, key: SymmetricKey(data: tempK1))
            let c2 = try CipherState(cipher: cipher, key: SymmetricKey(data: tempK2))

            return (c1, c2)
        }

        func checkpoint() {
            previousCK = ck
            previousH = h
        }

        func rollback() {
            ck = previousCK
            h = previousH
        }
    }

    /// A CipherState object contains `k` and `n` variables, which it uses to encrypt and decrypt ciphertexts.
    /// - Note: During the handshake phase each party has a single CipherState, but during the transport phase each party has two CipherState objects: one for sending, and one for receiving.
    /// - WARNING: HandshakeState is not thread safe, it's designed to be used within a single thread such as a NIO Channel
    public final class CipherState: @unchecked Sendable {
        private let cipher: NoiseCipherAlgorithm
        /// A cipher key of 32 bytes (which may be empty). Empty is a special value which indicates k has not yet been initialized.
        var k: SymmetricKey?
        /// An 8-byte (64-bit) unsigned integer nonce.
        var n: UInt64

        /// A CipherState
        /// - Note: The ++ post-increment operator applied to n means "use the current n value, then increment it".
        /// - Note: The maximum n value (264-1) is reserved for other use. If incrementing n results in 264-1, then any further EncryptWithAd() or DecryptWithAd() calls will signal an error to the caller.
        init(cipher: NoiseCipherAlgorithm, key: SymmetricKey? = nil) throws {
            self.cipher = cipher
            k = key
            n = 0
        }

        func initializeKey(key: [UInt8]) throws {
            if key.count > 32 { print("Warning! Using first 32 bytes of key") }
            k = SymmetricKey(data: key.prefix(32))
            n = 0
        }

        /// Returns true if k is non-empty, false otherwise.
        func hasKey() -> Bool {
            k != nil
        }

        /// Sets n = nonce. This function is used for handling out-of-order transport messages, as described in Section 11.4.
        func setNonce(_ nonce: UInt64) throws {
            n = nonce
        }

        /// If k is non-empty returns ENCRYPT(k, n++, ad, plaintext). Otherwise returns plaintext.
        func encryptWithAD(ad: [UInt8], plaintext: [UInt8]) throws -> [UInt8] {
            guard let symmetricKey = k else {
                // Return the unencrypted plain text
                return plaintext
            }

            // Ask our cipher function to encrypt the plaintext using our sym key, nonce, and authenticating data
            let enc = try cipher.encrypt(
                plaintext: plaintext,
                usingKey: symmetricKey,
                nonce: n,
                withAuthenticatingData: ad
            )

            // Increment the nonce
            n = n + 1

            //return Array<UInt8>(enc.ciphertext + enc.tag)
            return enc
        }

        /// If k is non-empty returns DECRYPT(k, n++, ad, ciphertext). Otherwise returns ciphertext.
        /// - Note: If an authentication failure occurs in DECRYPT() then n is not incremented and an error is signaled to the caller.
        func decryptWithAD(ad: [UInt8], ciphertext: [UInt8]) throws -> [UInt8] {
            guard let symmetricKey = k else {
                // Return ciphertext as is
                return ciphertext
            }

            // Ask our cipher function to decrypt the ciphertext using our sym key, nonce, and authenticating data
            let plaintext = try cipher.decrypt(
                ciphertext: ciphertext,
                usingKey: symmetricKey,
                nonce: n,
                withAuthenticatingData: ad
            )

            // Increment the nonce
            n = n + 1

            // Return the plaintext
            return plaintext
        }

        /// Swaps out the current Key for the new specified one
        func reKey(key: SymmetricKey) {
            k = key
        }

        public func encrypt(plaintext: [UInt8]) throws -> [UInt8] {
            try self.encryptWithAD(ad: [], plaintext: plaintext)
        }

        public func decrypt(ciphertext: [UInt8]) throws -> [UInt8] {
            try self.decryptWithAD(ad: [], ciphertext: ciphertext)
        }

        /// Returns the nonce as a UInt32 Little Endian byte array padded to 12 bytes
        //        private func getNonce() throws -> ChaChaPoly.Nonce {
        //            let padding:[UInt8] = [0x00, 0x00, 0x00, 0x00]
        //            let littleBytes = n.littleEndian.toBytes
        //
        //            return try ChaChaPoly.Nonce(data: padding + littleBytes)
        //        }
    }

}

private protocol UIntToBytesConvertable {
    var littleEndianBytes: [UInt8] { get }
    var bigEndianBytes: [UInt8] { get }
}

extension UIntToBytesConvertable {
    func toByteArr<T: BinaryInteger>(endian: T, count: Int) -> [UInt8] {
        var _endian = endian
        let bytePtr = withUnsafePointer(to: &_endian) {
            $0.withMemoryRebound(to: UInt8.self, capacity: count) {
                UnsafeBufferPointer(start: $0, count: count)
            }
        }
        return [UInt8](bytePtr)
    }
}

extension UInt64: UIntToBytesConvertable {
    //    var toBytes: [UInt8] {
    //        if OSHostByteOrder() == OSLittleEndian {
    //            return toByteArr(endian: self.littleEndian,
    //                             count: MemoryLayout<UInt64>.size)
    //        } else {
    //            return toByteArr(endian: self.bigEndian,
    //                             count: MemoryLayout<UInt64>.size)
    //        }
    //    }
    fileprivate var littleEndianBytes: [UInt8] {
        toByteArr(endian: self.littleEndian, count: MemoryLayout<UInt64>.size)
    }
    fileprivate var bigEndianBytes: [UInt8] {
        toByteArr(endian: self.bigEndian, count: MemoryLayout<UInt64>.size)
    }
}
