//
//  SearchView.swift
//  damus
//
//  Created by William Casarin on 2022-05-09.
//

import SwiftUI

struct NIP34SearchControls: View {
    @EnvironmentObject var user_settings: UserSettingsStore

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            Toggle("NIP-34 only", isOn: $user_settings.nip34_only)
            if user_settings.nip34_only {
                ScrollView(.horizontal, showsIndicators: true) {
                    HStack(spacing: 16) {
                        ForEach(NostrKind.nip34_kinds, id: \.self) { kind in
                            let selected = user_settings.search_nip34_kinds.contains(kind)
                            Button {
                                if selected {
                                    user_settings.search_nip34_kinds.remove(kind)
                                } else {
                                    user_settings.search_nip34_kinds.insert(kind)
                                }
                            } label: {
                                HStack(spacing: 5) {
                                    Image(systemName: selected ? "largecircle.fill.circle" : "circle")
                                    Text("\(String(kind)) \(NostrKind.nip34_kind_name(kind))")
                                }
                                .font(.caption)
                            }
                            .buttonStyle(.borderless)
                            .accessibilityValue(selected ? "On" : "Off")
                        }
                    }
                    .padding(.vertical, 4)
                }
            }
        }
    }
}

struct SearchView: View {
    let appstate: DamusState
    @StateObject var search: SearchModel
    @Environment(\.dismiss) var dismiss
    @EnvironmentObject var user_settings: UserSettingsStore
    
    var body: some View {
        TimelineView(events: $search.events, loading: $search.loading, damus: appstate, show_friend_icon: true, filter: {
            !user_settings.nip34_only || user_settings.search_nip34_kinds.contains($0.kind)
        }, show_results_while_loading: true)
            .safeAreaInset(edge: .top) {
                NIP34SearchControls().padding()
            }
            .navigationBarTitle(describe_search(search.search))
            .padding([.leading, .trailing], 6)
            .onReceive(handle_notify(.switched_timeline)) { obj in
                dismiss()
            }
            .onAppear() {
                search.subscribe(nip34_only: user_settings.nip34_only,
                                 selected_kinds: user_settings.search_nip34_kinds)
            }
            .onChange(of: user_settings.nip34_only) { enabled in
                search.unsubscribe()
                search.subscribe(nip34_only: enabled, selected_kinds: user_settings.search_nip34_kinds)
            }
            .onChange(of: user_settings.search_nip34_kinds) { selection in
                guard user_settings.nip34_only else { return }
                search.unsubscribe()
                search.subscribe(nip34_only: true, selected_kinds: selection)
            }
            .onDisappear() {
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
