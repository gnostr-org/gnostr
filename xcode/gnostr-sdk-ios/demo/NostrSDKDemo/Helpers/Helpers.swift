//
//  Helpers.swift
//  NostrSDKDemo
//
//  Created by Joel Klabo on 8/12/23.
//

import SwiftUI
import GnostrSDK

struct DemoHelper {
    private static let defaultHexPublicKey = "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"

    static var emptyString: Binding<String> {
        Binding.constant("")
    }
    static var previewRelay: Binding<Relay?> {
        let urlString = "wss://nos.lol"

        guard let url = URL(string: urlString) else {
            fatalError("Invalid URL: \(urlString)")
        }
        // If the Relay initializer throws an error, replace 'try?' with your error handling.
        let relay = try? Relay(url: url)

        return Binding.constant(relay)
    }
    static var validNpub: Binding<String> {
        guard let publicKey = PublicKey(hex: defaultHexPublicKey) else {
            fatalError("Invalid default public key hex")
        }

        return Binding.constant(publicKey.npub)
    }
    /// The Nostr SDK project and its maintainers take no responsibility of events signed with this private key which has been open sourced.
    /// Its purpose is for only testing and demos.
    static var validNsec: Binding<String> {
        Binding.constant("nsec1uwcvgs5clswpfxhm7nyfjmaeysn6us0yvjdexn9yjkv3k7zjhp2sv7rt36")
    }
    static var validHexPublicKey: Binding<String> {
        Binding.constant(defaultHexPublicKey)
    }
    /// The Nostr SDK project and its maintainers take no responsibility of events signed with this private key which has been open sourced.
    /// Its purpose is for only testing and demos.
    static var validHexPrivateKey: Binding<String> {
        Binding.constant("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    }
    static var invalidKey: Binding<String> {
        Binding.constant("not-valid")
    }
}
