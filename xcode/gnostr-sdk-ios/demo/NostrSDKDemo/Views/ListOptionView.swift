//
//  ListOptionView.swift
//  NostrSDKDemo
//
//  Created by Joel Klabo on 6/14/23.
//

import SwiftUI

struct ListOptionView: View {
    var destinationView: AnyView
    var imageName: String
    var labelText: String
    var useAssetImage: Bool = false
    var showsLabel: Bool = true

    var body: some View {
        NavigationLink(destination: destinationView) {
            HStack(spacing: showsLabel ? 12 : 0) {
                if useAssetImage {
                    Image(imageName)
                        .renderingMode(.template)
                } else {
                    Image(systemName: imageName)
                }

                if showsLabel {
                    Text(labelText)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }
}

struct ListOptionView_Previews: PreviewProvider {
    static var previews: some View {
        ListOptionView(destinationView: AnyView(GenerateKeyDemoView()), imageName: "key", labelText: "Key Generation")
    }
}
