//
//  EncryptMessageDemoView.swift
//  NostrSDKDemo
//
//  Created by Fabian Lachman on 31/12/24.
//

import SwiftUI
import GnostrSDK

struct EncryptMessageDemoView: View, EventCreating {

    @EnvironmentObject var relayPool: RelayPool
    @StateObject private var metadataLoader = PubkeyMetadataLoader()

    @State private var recipientPublicKey = ""
    @State private var recipientPublicKeyIsValid: Bool = false

    @State private var senderPrivateKey = ""
    @State private var senderPrivateKeyIsValid: Bool = false

    @State private var message: String = ""
    @State private var encryptedMessage: String = ""

    var body: some View {
        Form {
            Text("Encrypt Demo")
            Section("Recipient") {
                KeyInputSectionView(key: $recipientPublicKey,
                                    isValid: $recipientPublicKeyIsValid,
                                    type: .public)
            }
            if recipientPublicKeyIsValid {
                Section("Recipient Metadata") {
                    PubkeyMetadataPreviewView(metadata: metadataLoader.metadata)
                }
            }
            Section("Sender") {
                KeyInputSectionView(key: $senderPrivateKey,
                                    isValid: $senderPrivateKeyIsValid,
                                    type: .private)
            }
            Section("Message") {
                TextField("Enter a message.", text: $message)
            }
            Button("Encrypt") {
                guard let recipientPublicKey = publicKey(),
                      let senderKeyPair = keypair() else {
                    return
                }
                do {
                    encryptedMessage = try encrypt(plaintext: message, privateKeyA: senderKeyPair.privateKey, publicKeyB: recipientPublicKey)
                } catch {
                    encryptedMessage = ""
                    print(error.localizedDescription)
                }
            }
            .disabled(!ready())

            if encryptedMessage != "" {
                Section("Encrypted Message") {
                    TextField("Encrypted Message", text: $encryptedMessage)
                }
            }
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            ContextAwareHeaderView(
                title: "NIP-44 Encrypt",
                subtitle: "Encrypt a message for a recipient.",
                systemImage: "lock.fill",
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)
        }
        .onAppear {
            metadataLoader.attach(relayPool: relayPool)
            updateRecipientMetadata()
        }
        .onChange(of: recipientPublicKey) { _ in
            updateRecipientMetadata()
        }
        .onChange(of: recipientPublicKeyIsValid) { _ in
            updateRecipientMetadata()
        }
        .onDisappear {
            metadataLoader.stop()
        }
    }

    private func keypair() -> Keypair? {
        if senderPrivateKey.contains("nsec") {
            return Keypair(nsec: senderPrivateKey)
        } else {
            return Keypair(hex: senderPrivateKey)
        }
    }

    private func publicKey() -> PublicKey? {
        if recipientPublicKey.contains("npub") {
            return PublicKey(npub: recipientPublicKey)
        } else {
            return PublicKey(hex: recipientPublicKey)
        }
    }

    private func ready() -> Bool {
        !message.isEmpty &&
        recipientPublicKeyIsValid &&
        senderPrivateKeyIsValid
    }

    private func updateRecipientMetadata() {
        metadataLoader.update(publicKeyInput: recipientPublicKey, isValid: recipientPublicKeyIsValid)
    }
}

struct EncryptDecryptDemoView_Previews: PreviewProvider {
    static var previews: some View {
        EncryptMessageDemoView()
    }
}
