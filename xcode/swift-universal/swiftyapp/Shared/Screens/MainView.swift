//
//  MainView.swift
//  Universal App
//
//  Created by Can Balkaya on 12/11/20.
//

import SwiftUI

struct MainView: View {
    
    // MARK: - Properties
    #if os(iOS)
    @Environment(\.horizontalSizeClass) var horizontalSizeClass: UserInterfaceSizeClass?
    #endif
    
    // MARK: - UI Elements
    @ViewBuilder
    var body: some View {
        VStack(spacing: 0) {
            persistentHeader

            NavigationView {
                #if os(iOS)
                if horizontalSizeClass == .compact {
                    TabBar()
                } else {
                    SideBar()
                }
                #else
                SideBar()
                ArticlesListView(articles: techArticles)
                #endif
            }
        }
    }

    private var persistentHeader: some View {
        HStack(alignment: .center, spacing: 12) {
            Image("Icon")
                .resizable()
                .scaledToFit()
                .frame(width: 28, height: 28)
                .clipShape(RoundedRectangle(cornerRadius: 6, style: .continuous))
                .accessibilityHidden(true)

            VStack(alignment: .leading, spacing: 2) {
                Text("gnostr")
                    .font(.headline)
                Text("Universal App")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()
        }
        .padding(.horizontal)
        .padding(.vertical, 10)
        .background(.regularMaterial)
    }
}

struct MainView_Previews: PreviewProvider {
    static var previews: some View {
        MainView()
    }
}
