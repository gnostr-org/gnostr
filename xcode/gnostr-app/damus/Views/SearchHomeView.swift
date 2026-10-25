//
//  SearchHomeView.swift
//  damus
//
//  Created by William Casarin on 2022-05-19.
//

import SwiftUI
import CryptoKit

struct SearchHomeView: View {
    let damus_state: DamusState
    @StateObject var model: SearchHomeModel
    @Binding var search: String
    @EnvironmentObject var user_settings: UserSettingsStore
    @State private var is_active = false
    
    var GlobalContent: some View {
        return TimelineView(events: $model.events, loading: $model.loading, damus: damus_state, show_friend_icon: true, filter: {
            !user_settings.nip34_only || user_settings.search_nip34_kinds.contains($0.kind)
        })
            .refreshable {
                // Fetch new information by unsubscribing and resubscribing to the relay
                model.unsubscribe()
                model.subscribe(nip34_only: user_settings.nip34_only,
                                selected_kinds: user_settings.search_nip34_kinds)
            }
    }
    
    var SearchContent: some View {
        SearchResultsView(damus_state: damus_state, search: $search)
            .refreshable {
                // Fetch new information by unsubscribing and resubscribing to the relay
                model.unsubscribe()
                model.subscribe(nip34_only: user_settings.nip34_only,
                                selected_kinds: user_settings.search_nip34_kinds)
            }.padding(.horizontal)
    }
    
    var MainContent: some View {
        Group {
            if search.isEmpty {
                EmptyView()
                //GlobalContent
            } else {
                SearchContent
            }
        }
    }
    
    var body: some View {
        VStack {
            EmptyView()
            // MainContent // Entrypoint
        }
        .onChange(of: search.isEmpty) { empty in
            guard is_active else { return }
            if empty {
                model.subscribe(nip34_only: user_settings.nip34_only,
                                selected_kinds: user_settings.search_nip34_kinds)
            } else {
                model.unsubscribe()
            }
        }
        .onChange(of: user_settings.nip34_only) { enabled in
            guard is_active, search.isEmpty else { return }
            model.unsubscribe()
            model.subscribe(nip34_only: enabled, selected_kinds: user_settings.search_nip34_kinds)
        }
        .onChange(of: user_settings.search_nip34_kinds) { selection in
            guard is_active, search.isEmpty, user_settings.nip34_only else { return }
            model.unsubscribe()
            model.subscribe(nip34_only: true, selected_kinds: selection)
        }
        .onAppear {
            is_active = true
            if search.isEmpty && (model.events.isEmpty || model.nip34_only != user_settings.nip34_only ||
                model.selected_nip34_kinds != user_settings.search_nip34_kinds) {
                model.subscribe(nip34_only: user_settings.nip34_only,
                                selected_kinds: user_settings.search_nip34_kinds)
            }
        }
        .onDisappear {
            is_active = false
            model.unsubscribe()
        }
    }
}

struct SearchHomeView_Previews: PreviewProvider {
    static var previews: some View {
        let state = test_damus_state()
        SearchHomeView(
            damus_state: state,
            model: SearchHomeModel(damus_state: state),
            search: .constant("")
        )
        .environmentObject(UserSettingsStore())
    }
}
