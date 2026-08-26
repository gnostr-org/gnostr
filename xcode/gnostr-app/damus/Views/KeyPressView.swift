//
//  KeyPressView.swift
//  damus
//
//  Created by William Casarin on 2022-05-04.
//

import SwiftUI

struct KeyPressView: View {
    var onEscape: () -> Void

    var body: some View {
        Button("") {
            onEscape()
        }
        .keyboardShortcut(.escape, modifiers: [])
    }
}
