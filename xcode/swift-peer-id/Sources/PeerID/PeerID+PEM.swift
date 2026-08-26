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

/// - MARK: PEM Imports and Exports
extension PeerID {

    /// PeerID PEM Related Errors
    public enum PEMError: Error {
        /// No Underlying Key Pair to Export
        case noKeypairToExport
        /// This PeerID doesn't have a Private Key to Export
        case noPrivateKeyAvailable
        /// Password shouldn't be empty
        case invalidPassword
    }

    /// Initializes a PeerID using a PEM string
    /// - Parameters:
    ///   - pem: The PEM file in string form
    ///   - password: An optional password used to decrypt the PEM if it's encrypted
    public init(pem: String, password: String?) throws {
        try self.init(keyPair: LibP2PCrypto.Keys.KeyPair(pem: pem, password: password))
    }

    /// PEM Export Type
    public enum ExportType {
        /// Exports the PeerID's backing Public Key as a PEM string
        case publicPEMString
        /// Exports the PeerID's backing PrivateKey as a PEM string, encrypted with the password provided
        case privatePEMString(encryptedWithPassword: String)
        /// Exports the PeerID's backing PrivateKey as an UNENCRYPTED PEM string
        /// - WARNING: Not Recommended
        /// - NOTE: Use the `.privatePEMString(encryptedWithPassword:)` method instead
        case unencrypredPrivatePEMString
    }

    /// Exports the KeyPair as  PEM structured String. Private Keys can be encrypted with a password before export.
    /// - Parameter exportType: The type of PEM string to export
    /// - Returns: The PEM string
    public func exportKeyPair(as exportType: ExportType) throws -> String {
        guard let keyPair = self.keyPair else {
            throw PEMError.noKeypairToExport
        }
        switch exportType {
        case .publicPEMString:
            return try keyPair.publicKey.exportPublicKeyPEMString(withHeaderAndFooter: true)
        case .unencrypredPrivatePEMString:
            guard keyPair.privateKey != nil else {
                throw PEMError.noPrivateKeyAvailable
            }
            return try keyPair.exportPrivatePEMString(withHeaderAndFooter: true)
        case .privatePEMString(let password):
            guard !password.isEmpty else {
                throw PEMError.invalidPassword
            }
            return try keyPair.exportEncryptedPrivatePEMString(withPassword: password)
        }
    }
}
