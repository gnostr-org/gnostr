//
//  SideBar.swift
//  Universal App (macOS)
//
//  Created by Can Balkaya on 12/10/20.
//

import SwiftUI

enum NavigationItem {
    case tech
    case science
    case design
    case services
}

struct SideBar: View {
    
    // MARK: - Properties
    @State var selection: Set<NavigationItem> = [.tech]
    
    // MARK: - UI Elements
    @ViewBuilder
    var body: some View {
        List(selection: $selection) {
            NavigationLink(
                destination: ArticlesListView(articles: techArticles),
                label: {
                    Label("Tech", systemImage: "newspaper.fill")
                }
            )
            .tag(NavigationItem.tech)
            
            NavigationLink(
                destination: ArticlesListView(articles: scienceArticles),
                label: {
                    Label("Science", systemImage: "paperclip")
                }
            )
            .tag(NavigationItem.science)
            
            NavigationLink(
                destination: ArticlesListView(articles: designArticles),
                label: {
                    Label("Design", systemImage: "rectangle.and.paperclip")
                }
            )
            .tag(NavigationItem.design)

            NavigationLink(
                destination: P2PListView(),
                label: {
                    Label("P2P", systemImage: "antenna.radiowaves.left.and.right")
                }
            )
            .tag(NavigationItem.services)
        }
        .navigationTitle("gnostr")
#if os(macOS)
        .listStyle(SidebarListStyle())
#endif
    }
}

struct SideBar_Previews: PreviewProvider {
    static var previews: some View {
        SideBar()
    }
}
