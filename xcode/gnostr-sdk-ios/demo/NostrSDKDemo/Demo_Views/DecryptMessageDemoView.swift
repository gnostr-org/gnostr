//
//  DecryptMessageDemoView.swift
//  NostrSDKDemo
//
//  Created by Fabian Lachman on 31/12/24.
//

import SwiftUI
import GnostrSDK

struct DecryptMessageDemoView: View, EventCreating {

    @EnvironmentObject var relayPool: RelayPool
    @StateObject private var metadataLoader = PubkeyMetadataLoader()

    @State private var senderPublicKey = ""
    @State private var senderPublicKeyIsValid: Bool = false

    @State private var receiverPrivateKey = ""
    @State private var receiverPrivateKeyIsValid: Bool = false

    @State private var encryptedMessage: String = ""
    @State private var message: String = ""

    var body: some View {
        Form {
            Text("Decrypt Demo")
            Section("Sender") {
                KeyInputSectionView(key: $senderPublicKey,
                                    isValid: $senderPublicKeyIsValid,
                                    type: .public)
            }
            if senderPublicKeyIsValid {
                Section("Sender Metadata") {
                    PubkeyMetadataPreviewView(metadata: metadataLoader.metadata)
                }
            }
            Section("Receiver") {
                KeyInputSectionView(key: $receiverPrivateKey,
                                    isValid: $receiverPrivateKeyIsValid,
                                    type: .private)
            }
            Section("Encrypted Message") {
                TextField("Enter encrypted message.", text: $encryptedMessage)
            }
            Button("Decrypt") {
                guard let senderPublicKey = publicKey(),
                      let receiverPrivateKey = keypair() else {
                    return
                }
                do {
                    message = try decrypt(payload: encryptedMessage, privateKeyA: receiverPrivateKey.privateKey, publicKeyB: senderPublicKey)
                } catch {
                    message = ""
                    print(error.localizedDescription)
                }
            }
            .disabled(!ready())

            if message != "" {
                Section("Decrypted Message") {
                    TextField("Decrypted Message", text: $message)
                }
            }
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            ContextAwareHeaderView(
                title: "NIP-44 Decrypt",
                subtitle: "Decrypt an encrypted message.",
                systemImage: "lock.open.fill",
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)
        }
        .onAppear {
            metadataLoader.attach(relayPool: relayPool)
            updateSenderMetadata()
        }
        .onChange(of: senderPublicKey) { _ in
            updateSenderMetadata()
        }
        .onChange(of: senderPublicKeyIsValid) { _ in
            updateSenderMetadata()
        }
        .onDisappear {
            metadataLoader.stop()
        }
    }

    private func keypair() -> Keypair? {
        if receiverPrivateKey.contains("nsec") {
            return Keypair(nsec: receiverPrivateKey)
        } else {
            return Keypair(hex: receiverPrivateKey)
        }
    }

    private func publicKey() -> PublicKey? {
        if senderPublicKey.contains("npub") {
            return PublicKey(npub: senderPublicKey)
        } else {
            return PublicKey(hex: senderPublicKey)
        }
    }

    private func ready() -> Bool {
        !encryptedMessage.isEmpty &&
        senderPublicKeyIsValid &&
        receiverPrivateKeyIsValid
    }

    private func updateSenderMetadata() {
        metadataLoader.update(publicKeyInput: senderPublicKey, isValid: senderPublicKeyIsValid)
    }
}

struct DecryptMessageDemoView_Previews: PreviewProvider {
    static var previews: some View {
        DecryptMessageDemoView()
    }
}
