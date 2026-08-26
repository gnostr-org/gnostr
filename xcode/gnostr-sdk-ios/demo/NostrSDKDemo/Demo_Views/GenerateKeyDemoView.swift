//
//  GenerateKeyView.swift
//  NostrSDKDemo
//
//  Created by Joel Klabo on 6/14/23.
//

import SwiftUI
import GnostrSDK

struct GenerateKeyDemoView: View {

    @State private var privateKey: String?
    @State private var publicKey: String?
    @State private var nsec: String?
    @State private var npub: String?

    private let noValueString = "Must generate key"

    var body: some View {
        Form {
            Button("Generate Key") {
                let keypair = Keypair()
                privateKey = keypair?.privateKey.hex ?? ""
                publicKey = keypair?.publicKey.hex ?? ""
                nsec = keypair?.privateKey.nsec ?? ""
                npub = keypair?.publicKey.npub
            }
            Section("Private Key") {
                Text(privateKey ?? noValueString)
            }
            Section("32:Public Key") {
                Text(publicKey ?? noValueString)
            }
            Section("35:nsec") {
                Text(nsec ?? noValueString)
            }
            Section("38:npub") {
                Text(npub ?? noValueString)
            }
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            ContextAwareHeaderView(
                title: "Key Generation",
                subtitle: "Generate and inspect a nostr keypair.",
                systemImage: "key",
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)
        }
    }
}

struct GenerateKeyView_Previews: PreviewProvider {
    static var previews: some View {
        GenerateKeyDemoView()
    }
}
