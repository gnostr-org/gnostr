//
//  SearchView.swift
//  damus
//
//  Created by William Casarin on 2022-05-09.
//

import SwiftUI

struct SearchView: View {
    let appstate: DamusState
    @StateObject var search: SearchModel
    @Environment(\.dismiss) var dismiss
    @EnvironmentObject var user_settings: UserSettingsStore
    @State private var is_active = false
    
    var body: some View {
        TimelineView(events: $search.events, loading: $search.loading, damus: appstate, show_friend_icon: true, filter: {
            !user_settings.nip34_only || user_settings.search_nip34_kinds.contains($0.kind)
        }, show_results_while_loading: true)
            .navigationBarTitle(describe_search(search.search))
            .padding([.leading, .trailing], 6)
            .onReceive(handle_notify(.switched_timeline)) { obj in
                dismiss()
            }
            .onAppear() {
                is_active = true
                search.subscribe(nip34_only: user_settings.nip34_only,
                                 selected_kinds: user_settings.search_nip34_kinds)
            }
            .onChange(of: user_settings.nip34_only) { enabled in
                guard is_active else { return }
                search.unsubscribe()
                search.subscribe(nip34_only: enabled, selected_kinds: user_settings.search_nip34_kinds)
            }
            .onChange(of: user_settings.search_nip34_kinds) { selection in
                guard is_active, user_settings.nip34_only else { return }
                search.unsubscribe()
                search.subscribe(nip34_only: true, selected_kinds: selection)
            }
            .onDisappear() {
                is_active = false
                search.unsubscribe()
            }
    }
}

func describe_search(_ filter: NostrFilter) -> String {
    if let hashtags = filter.hashtag {
        if hashtags.count >= 1 {
            return "#" + hashtags[0]
        }
    }
    return "Search"
}

struct SearchView_Previews: PreviewProvider {
    static var previews: some View {
        let test_state = test_damus_state()
        let filter = NostrFilter.filter_hashtag(["bitcoin"])
        let pool = test_state.pool
        let model = SearchModel(pool: pool, search: filter)
        
        SearchView(appstate: test_state, search: model)
            .environmentObject(UserSettingsStore())
    }
}
