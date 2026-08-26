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
import LibP2PCrypto

/// - MARK: PeerID Signatures and Verification Methods
extension PeerID {

    /// PeerID PEM Related Errors
    public enum SignatureError: Error {
        /// A public key is required for verifying signatures and this PeerID doesn't contain a public key.
        case noPublicKeyAvailable
        /// A private key is required for generating signatures and this PeerID doesn't contain a private key.
        case noPrivateKeyAvailable
    }

    /// Signs data using this PeerID's private key. This signature can then be verified by a remote peer using this PeerID's public key
    /// - Parameter msg: The message to sign
    /// - Returns: The signed message
    public func signature(for msg: Data) throws -> Data {
        guard let priv = keyPair?.privateKey else {
            throw SignatureError.noPrivateKeyAvailable
        }

        return try priv.sign(message: msg)
    }

    /// Using this PeerID's public key, this method checks to see if the signature data was in fact signed by this peer and is a valid signature for the expected data
    /// - Parameters:
    ///   - signature: The signed message you want to validate
    ///   - expectedData: The data you're comparing the signature against
    /// - Returns: True if the expected data was signed by this PeerID's public key, or False if not.
    public func isValidSignature(_ signature: Data, for expectedData: Data) throws -> Bool {
        guard let pub = keyPair?.publicKey else {
            throw SignatureError.noPublicKeyAvailable
        }

        return try pub.verify(signature: signature, for: expectedData)
    }
}
