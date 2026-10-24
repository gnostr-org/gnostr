//
//  UserSettingsStore.swift
//  damus
//
//  Created by Suhail Saqan on 12/29/22.
//

import Foundation

class UserSettingsStore: ObservableObject {
    private let defaults: UserDefaults

    @Published var default_wallet: Wallet {
        didSet {
            defaults.set(default_wallet.rawValue, forKey: "default_wallet")
        }
    }
    
    @Published var show_wallet_selector: Bool {
        didSet {
            defaults.set(show_wallet_selector, forKey: "show_wallet_selector")
        }
    }
    
    @Published var nip34_only: Bool {
        didSet {
            defaults.set(nip34_only, forKey: "nip34_only")
        }
    }

    @Published var search_nip34_kinds: Set<Int> {
        didSet {
            defaults.set(search_nip34_kinds.sorted(), forKey: "search_nip34_kinds")
        }
    }

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        if let defaultWalletName = defaults.string(forKey: "default_wallet"),
           let default_wallet = Wallet(rawValue: defaultWalletName) {
            self.default_wallet = default_wallet
        } else {
            self.default_wallet = .system_default_wallet
        }
        self.show_wallet_selector = defaults.object(forKey: "show_wallet_selector") as? Bool ?? true
        self.nip34_only = defaults.object(forKey: "nip34_only") as? Bool ?? false
        self.search_nip34_kinds = Set(defaults.array(forKey: "search_nip34_kinds") as? [Int]
                                     ?? NostrKind.nip34_kinds).intersection(NostrKind.nip34_kinds)
    }
}
