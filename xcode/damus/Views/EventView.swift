//
//  EventView.swift
//  damus
//
//  Created by William Casarin on 2022-04-11.
//

import Foundation
import SwiftUI
import GnostrGit

enum Highlight {
    case none
    case main
    case reply
    case custom(Color, Float)

    var is_main: Bool {
        if case .main = self {
            return true
        }
        return false
    }

    var is_none: Bool {
        if case .none = self {
            return true
        }
        return false
    }

    var is_replied_to: Bool {
        switch self {
        case .reply: return true
        default: return false
        }
    }
}

enum EventViewKind {
    case small
    case normal
    case big
    case selected
}

func eventviewsize_to_font(_ size: EventViewKind) -> Font {
    switch size {
    case .small:
        return .body
    case .normal:
        return .body
    case .big:
        return .headline
    case .selected:
        return .custom("selected", size: 21.0)
    }
}

struct BuilderEventView: View {
    let damus: DamusState
    let event_id: String
    @State var event: NostrEvent?
    @State var subscription_uuid: String = UUID().description
    
    func unsubscribe() {
        damus.pool.unsubscribe(sub_id: subscription_uuid)
    }
    
    func subscribe(filters: [NostrFilter]) {
        damus.pool.register_handler(sub_id: subscription_uuid, handler: handle_event)
        damus.pool.send(.subscribe(.init(filters: filters, sub_id: subscription_uuid)))
    }
    
    func handle_event(relay_id: String, ev: NostrConnectionEvent) {
        guard case .nostr_event(let nostr_response) = ev else {
            return
        }
        
        guard case .event(let id, let nostr_event) = nostr_response else {
            return
        }
        
        // Is current event
        if id == subscription_uuid {
            if event != nil {
                return
            }
            
            event = nostr_event
            
            unsubscribe()
        }
    }
    
    func load() {
        subscribe(filters: [
            NostrFilter(
                ids: [self.event_id],
                limit: 1
            )
        ])
    }
    
    var body: some View {
        VStack {
            if event == nil {
                ProgressView().padding()
            } else {
                NavigationLink(destination: BuildThreadV2View(damus: damus, event_id: event!.id)) {
                    EventView(damus: damus, event: event!, show_friend_icon: true, size: .small, embedded: true)
                }.buttonStyle(.plain)
            }
        }
        .frame(minWidth: 0, maxWidth: .infinity)
        .border(Color.gray.opacity(0.2), width: 1)
        .cornerRadius(2)
        .onAppear {
            self.load()
        }
    }
}

struct
EventView: View {
    let event: NostrEvent
    let highlight: Highlight
    let has_action_bar: Bool
    let damus: DamusState
    let pubkey: String
    let show_friend_icon: Bool
    let size: EventViewKind
    let embedded: Bool

    @EnvironmentObject var webViewURL: WebViewURL
    @EnvironmentObject var webViewModel: WebViewModel
    @EnvironmentObject var gitOperationTracker: GitOperationTracker // Add this line
    @State private var repoToClone: (url: String, name: String)?
    @State private var announcementEvent: NostrEvent?
    @State private var combinedGitRefs: [[String]] = [] // New state
    @State private var activeGitRef: (refName: String, commitHash: String)?
    @State private var showGitRefPanel: Bool = false

    func fetchAnnouncementEvent() {
        guard event.known_kind == .repository_state_announcement else { return }
        
        if let d_tag = event.tags.first(where: { $0.first == "d" })?.last {
            let filter = NostrFilter(kinds: [NostrKind.repository_announcement.rawValue], limit: 1, tags: ["d": [d_tag]])
            let sub_id = UUID().description
            
            damus.pool.register_handler(sub_id: sub_id) { relay_id, ev in
                if case .nostr_event(let nostr_response) = ev, case .event(_, let announcement) = nostr_response {
                    DispatchQueue.main.async {
                        self.announcementEvent = announcement
                        let announcement_refs = announcement.tags.filter { $0.count > 1 && ($0[0].starts(with: "refs/") || $0[0] == "HEAD") }
                        self.combinedGitRefs.append(contentsOf: announcement_refs)
                    }
                    self.damus.pool.unsubscribe(sub_id: sub_id)
                }
            }
            damus.pool.send(.subscribe(.init(filters: [filter], sub_id: sub_id)))
        }
    }
    
    init(event: NostrEvent, highlight: Highlight, has_action_bar: Bool, damus: DamusState, show_friend_icon: Bool, size: EventViewKind = .normal, embedded: Bool = false) {
        self.event = event
        self.highlight = highlight
        self.has_action_bar = has_action_bar
        self.damus = damus
        self.pubkey = event.pubkey
        self.show_friend_icon = show_friend_icon
        self.size = size
        self.embedded = embedded
    }

    init(damus: DamusState, event: NostrEvent, show_friend_icon: Bool, size: EventViewKind = .normal, embedded: Bool = false) {
        self.event = event
        self.highlight = .none
        self.has_action_bar = false
        self.damus = damus
        self.pubkey = event.pubkey
        self.show_friend_icon = show_friend_icon
        self.size = size
        self.embedded = embedded
    }

    init(damus: DamusState, event: NostrEvent, pubkey: String, show_friend_icon: Bool, size: EventViewKind = .normal, embedded: Bool = false) {
        self.event = event
        self.highlight = .none
        self.has_action_bar = false
        self.damus = damus
        self.pubkey = pubkey
        self.show_friend_icon = show_friend_icon
        self.size = size
        self.embedded = embedded
    }

    private static func fetchContent(from url: URL, completion: @escaping (String?) -> Void) {
        URLSession.shared.dataTask(with: url) { data, response, error in
            guard let data = data, error == nil else {
                print("Error fetching content from \(url): \(error?.localizedDescription ?? "Unknown error")")
                completion(nil)
                return
            }
            completion(String(data: data, encoding: .utf8))
        }.resume()
    }

    private static func testNip11Support(for url: URL, completion: @escaping (Bool) -> Void) {
        var request = URLRequest(url: url)
        request.httpMethod = "GET" // Use GET
        request.setValue("application/nostr+json", forHTTPHeaderField: "Accept") // Check for JSON support first

        URLSession.shared.dataTask(with: request) { _, response, error in
            guard let httpResponse = response as? HTTPURLResponse, error == nil else {
                completion(false)
                return
            }

            // A NIP-11 compliant relay should respond with application/nostr+json
            if let contentType = httpResponse.allHeaderFields["Content-Type"] as? String,
               contentType.contains("application/nostr+json") {
                completion(true)
            } else {
                // For relays that might not correctly set the content type but are NIP-11 compliant,
                // we can also check for a successful status code and assume support.
                // Or we can try a second request with Accept: text/html and check for that content type.
                // For simplicity, we'll just check for the expected content type for now.
                completion(false)
            }
        }.resume()
    }

    private func parseRelays(_ relaysString: String) -> [String] {
        if let data = relaysString.data(using: .utf8),
           let array = try? JSONDecoder().decode([String].self, from: data) {
            return array
        }
        // Fallback to splitting by comma
        return relaysString.components(separatedBy: ",").map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
    }

    var body: some View {
        return Group {
            if event.known_kind == .boost, let inner_ev = event.inner_event {
                VStack(alignment: .leading) {
                    let prof_model = ProfileModel(pubkey: event.pubkey, damus: damus)
                    let follow_model = FollowersModel(damus_state: damus, target: event.pubkey)
                    let prof = damus.profiles.lookup(id: event.pubkey)
                    let booster_profile = ProfileView(damus_state: damus, profile: prof_model, followers: follow_model)
                    
                    NavigationLink(destination: booster_profile) {
                        HStack {
                            Image(systemName: "arrow.2.squarepath")
                                .font(.footnote.weight(.bold))
                                .foregroundColor(Color.gray)
                            ProfileName(pubkey: event.pubkey, profile: prof, contacts: damus.contacts, show_friend_confirmed: true)
                                    .font(.footnote.weight(.bold))
                                    .foregroundColor(Color.gray)
                            Text("Boosted")
                                .font(.footnote.weight(.bold))
                                .foregroundColor(Color.gray)
                        }
                    }
                    .buttonStyle(PlainButtonStyle())
                    TextEvent(inner_ev, pubkey: inner_ev.pubkey)
                        .padding([.top], 1)
                }
            } else {
                TextEvent(event, pubkey: pubkey)
                    .padding([.top], 6)
            }
        }
    }

    func TextEvent(_ event: NostrEvent, pubkey: String) -> some View {
        let content = event.get_content(damus.keypair.privkey)
        
        return HStack(alignment: .top) {
            let profile = damus.profiles.lookup(id: pubkey)
            
            if size != .selected {
                VStack {
                    let pmodel = ProfileModel(pubkey: pubkey, damus: damus)
                    let pv = ProfileView(damus_state: damus, profile: pmodel, followers: FollowersModel(damus_state: damus, target: pubkey))
                    
                    if !embedded {
                        NavigationLink(destination: pv) {
                            ProfilePicView(pubkey: pubkey, size: PFP_SIZE, highlight: highlight, profiles: damus.profiles)
                        }
                    }
                    
                    Spacer()
                }
            }

            VStack(alignment: .leading) {
                HStack(alignment: .center) {
                    if size == .selected {
                        VStack {
                            let pmodel = ProfileModel(pubkey: pubkey, damus: damus)
                            let pv = ProfileView(damus_state: damus, profile: pmodel, followers: FollowersModel(damus_state: damus, target: pubkey))
                            
                            NavigationLink(destination: pv) {
                                ProfilePicView(pubkey: pubkey, size: PFP_SIZE, highlight: highlight, profiles: damus.profiles)
                            }
                        }
                    }
                    
                    EventProfileName(pubkey: pubkey, profile: profile, contacts: damus.contacts, show_friend_confirmed: show_friend_icon, size: size)
                    if size != .selected {
                        Text("\(format_relative_time(event.created_at))")
                            .font(eventviewsize_to_font(size))
                            .foregroundColor(.gray)
                    }
                }
                
                if event.is_reply(damus.keypair.privkey) {
                    Text("\(reply_desc(profiles: damus.profiles, event: event))")
                        .font(.footnote)
                        .foregroundColor(.gray)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }

                if event.known_kind == .repository_state_announcement {
                    if let announcement = announcementEvent {
                        let announcement_should_show_img = should_show_images(contacts: damus.contacts, ev: announcement, our_pubkey: damus.pubkey)
                        NoteContentView(privkey: damus.keypair.privkey, event: announcement, profiles: damus.profiles, show_images: announcement_should_show_img, artifacts: .just_content(announcement.get_content(damus.keypair.privkey)), size: .small)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .allowsHitTesting(!embedded)
                    }
                    
                    GitRefsView(tags: combinedGitRefs, onRefTapped: { refName, commitHash in
                        self.activeGitRef = (refName, commitHash)
                        self.showGitRefPanel = true
                    })
                        .onAppear {
                            if self.combinedGitRefs.isEmpty { // Fetch only once
                                self.combinedGitRefs = event.tags.filter { $0.count > 1 && ($0[0].starts(with: "refs/") || $0[0] == "HEAD") }
                                fetchAnnouncementEvent()
                            }
                        }
                        .sheet(isPresented: $showGitRefPanel) {
                            if let activeGitRef = activeGitRef {
                                let d_tag = event.tags.first(where: { $0.first == "d" })?.last ?? "unknown-repo"
                                GitRefDetailView(repo_name: d_tag, refName: activeGitRef.refName, commitHash: activeGitRef.commitHash, isPresented: $showGitRefPanel)
                            }
                        }
                } else {
                    let should_show_img = should_show_images(contacts: damus.contacts, ev: event, our_pubkey: damus.pubkey)
                    NoteContentView(privkey: damus.keypair.privkey, event: event, profiles: damus.profiles, show_images: should_show_img, artifacts: .just_content(content), size: self.size)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .allowsHitTesting(!embedded)
                }
                
                let isNip34 = event.known_kind == .repository_announcement || event.known_kind == .repository_state_announcement || event.known_kind == .repository_patch || event.known_kind == .repository_issue_draft
                if isNip34 {
                    HStack {
                        TagsView(tags: event.tags, onCloneTapped: { url in
                            let repoName = url.lastPathComponent.replacingOccurrences(of: ".git", with: "")
                            print("User tapped clone URL: \(url.absoluteString)")
                            self.webViewModel.repo_url = url.absoluteString
                            self.webViewModel.repo_name = repoName
                            let commitTags = event.tags.filter { $0.first == "commit" || $0.first == "parent-commit" }
                            self.webViewModel.commitsToFetch = commitTags.compactMap { $0.count > 1 ? $0[1] : nil }
                            self.webViewModel.clone() // Directly initiate clone from webViewModel
                        }, onWebTapped: { url in
                            print("User tapped web URL: \(url.absoluteString)")
                            self.webViewURL.url = url
                            self.webViewModel.htmlContent = nil
                            // Existing repo info logic for WebView clone button
                            if let repoInfo = self.repoToClone {
                                self.webViewModel.repo_url = repoInfo.url
                                self.webViewModel.repo_name = repoInfo.name
                                let commitTags = event.tags.filter { $0.first == "commit" || $0.first == "parent-commit" }
                                self.webViewModel.commitsToFetch = commitTags.compactMap { $0.count > 1 ? $0[1] : nil }
                            } else {
                                self.webViewModel.repo_url = nil
                                self.webViewModel.repo_name = nil
                                self.webViewModel.commitsToFetch = []
                            }
                        }, onRelaysTapped: { relaysString in
                            let relays = self.parseRelays(relaysString)
                            for relay in relays {
                                if let url = URL(string: relay) {
                                    // Add the relay
                                    if let privkey = self.damus.keypair.privkey, let ev = self.damus.contacts.event {
                                        if let new_ev = add_relay(ev: ev, privkey: privkey, current_relays: self.damus.pool.descriptors, relay: url.absoluteString, info: .rw) {
                                            if (try? self.damus.pool.add_relay(url, info: .rw)) != nil {
                                                self.damus.pool.connect(to: [url.absoluteString])
                                                process_contact_event(pool: self.damus.pool, contacts: self.damus.contacts, pubkey: self.damus.pubkey, ev: new_ev)
                                                self.damus.pool.send(.event(new_ev))
                                            }
                                        }
                                    }
                                }
                            }
                        }, onDTapped: { d_tag in
                            let cloneURLs = event.tags.filter { $0.first == "clone" && $0.count > 1 }.compactMap { URL(string: $0[1]) }
                            if cloneURLs.count == 1 {
                                let url = cloneURLs[0]
                                print("User tapped d: tag, automatically cloning repository: \(url.absoluteString)")
                                self.gitOperationTracker.startGitOperation(repo_url: url.absoluteString, repo_name: d_tag, commitsToFetch: [])
                            } else {
                                print("User tapped d: tag, but there are \(cloneURLs.count) clone URLs. Cannot automatically clone. Please select a specific clone URL.")
                            }
                        }, onSearchTapped: { searchString in
                            NotificationCenter.default.post(name: .search_string, object: searchString)
                        })
                        
                        Button(action: {
                            let d_tag = event.tags.first(where: { $0.first == "d" })?.last ?? "unknown-repo"
                            let cloneURLs = event.tags.filter { $0.first == "clone" && $0.count > 1 }.compactMap { URL(string: $0[1]) }
                            if cloneURLs.count == 1 {
                                let url = cloneURLs[0]
                                print("User tapped git icon, automatically selecting repository: \(url.absoluteString)")
                                self.repoToClone = (url: url.absoluteString, name: d_tag)
                            } else {
                                print("User tapped git icon, but there are \(cloneURLs.count) clone URLs. Please select a specific clone URL.")
                            }
                        }) {
                            Image(systemName: "arrow.down.circle")
                                .font(.footnote)
                        }
                        .buttonStyle(.plain)
                    }
                    .onAppear {
                        let d_tag = event.tags.first(where: { $0.first == "d" })?.last
                        let cloneURLs = event.tags.filter { $0.first == "clone" && $0.count > 1 }.compactMap { URL(string: $0[1]) }
                        
                        if cloneURLs.isEmpty, let d = d_tag {
                            print("No clone URLs found, searching for 30617 event for d_tag: \(d)")
                            let announcement_sub_id = UUID().description
                            var announcement_filter = NostrFilter()
                            announcement_filter.kinds = [30617]
                            announcement_filter.tags = ["d": [d]]
                            announcement_filter.limit = 1
                            
                            damus.pool.register_handler(sub_id: announcement_sub_id) { relay_id, ev in
                                if case .nostr_event(let nostr_response) = ev, case .event(_, let announcement_event) = nostr_response {
                                    print("Received 30617 event for d_tag: \(d)")
                                    let foundCloneURLs = announcement_event.tags.filter { $0.first == "clone" && $0.count > 1 }.compactMap { URL(string: $0[1]) }
                                    if let url = foundCloneURLs.first {
                                        print("Found clone URL from 30617 event: \(url.absoluteString)")
                                        self.repoToClone = (url: url.absoluteString, name: d)
                                    }
                                    self.damus.pool.unsubscribe(sub_id: announcement_sub_id)
                                }
                            }
                            damus.pool.send(.subscribe(.init(filters: [announcement_filter], sub_id: announcement_sub_id)))
                        } else if cloneURLs.count == 1 {
                            let url = cloneURLs[0]
                            let repoName = url.lastPathComponent.replacingOccurrences(of: ".git", with: "")
                            print("Automatically selecting repository: \(url.absoluteString)")
                            self.repoToClone = (url: url.absoluteString, name: d_tag ?? repoName)
                        }
                    }

                    if let repoInfo = repoToClone {
                        let commitTags = event.tags.filter { $0.first == "commit" || $0.first == "parent-commit" }
                        let commits = commitTags.compactMap { $0.count > 1 ? $0[1] : nil }
                        GitView(repo_url: repoInfo.url, repo_name: repoInfo.name, commitsToFetch: commits)
                    }
                }
            }
            .padding([.leading], 2)
        }
        .contentShape(Rectangle())
        .background(event_validity_color(event.validity))
        .id(event.id)
        .frame(maxWidth: .infinity, minHeight: PFP_SIZE)
        .padding([.bottom], 2)
        .event_context_menu(event, privkey: damus.keypair.privkey)
    }
}

// blame the porn bots for this code
func should_show_images(contacts: Contacts, ev: NostrEvent, our_pubkey: String) -> Bool {
    if ev.pubkey == our_pubkey {
        return true
    }
    if contacts.is_in_friendosphere(ev.pubkey) {
        return true
    }
    return false
}

func event_validity_color(_ validation: ValidationResult) -> some View {
    Group {
        switch validation {
        case .ok:
            EmptyView()
        case .bad_id:
            Color.orange.opacity(0.4)
        case .bad_sig:
            Color.red.opacity(0.4)
        }
    }
}

extension View {
    func pubkey_context_menu(bech32_pubkey: String) -> some View {
        return self.contextMenu {
            Button {
                    UIPasteboard.general.string = bech32_pubkey
            } label: {
                Label("Copy Account ID", systemImage: "doc.on.doc")
            }
        }
    }
    
    func event_context_menu(_ event: NostrEvent, privkey: String?) -> some View {
        return self.contextMenu {
            Button {
                UIPasteboard.general.string = event.get_content(privkey)
            } label: {
                Label("Copy Text", systemImage: "doc.on.doc")
            }

            Button {
                UIPasteboard.general.string = bech32_pubkey(event.pubkey) ?? event.pubkey
            } label: {
                Label("Copy User ID", systemImage: "tag")
            }

            Button {
                UIPasteboard.general.string = bech32_note_id(event.id) ?? event.id
            } label: {
                Label("Copy Note ID", systemImage: "tag")
            }

            Button {
                UIPasteboard.general.string = event_to_json(ev: event)
            } label: {
                Label("Copy Note JSON", systemImage: "note")
            }

            Button {
                NotificationCenter.default.post(name: .broadcast_event, object: event)
            } label: {
                Label("Broadcast", systemImage: "globe")
            }
        }

    }
}

func format_relative_time(_ created_at: Int64) -> String
{
    return time_ago_since(Date(timeIntervalSince1970: Double(created_at)))
}

func format_date(_ created_at: Int64) -> String {
    let date = Date(timeIntervalSince1970: TimeInterval(created_at))
    let dateFormatter = DateFormatter()
    dateFormatter.timeStyle = .short
    dateFormatter.dateStyle = .short
    return dateFormatter.string(from: date)
}


func reply_desc(profiles: Profiles, event: NostrEvent) -> String {
    let desc = make_reply_description(event.tags)
    let pubkeys = desc.pubkeys
    let n = desc.others

    if desc.pubkeys.count == 0 {
        return "Reply to self"
    }

    let names: [String] = pubkeys.map {
        let prof = profiles.lookup(id: $0)
        return Profile.displayName(profile: prof, pubkey: $0)
    }

    if names.count == 2 {
        if n > 2 {
            let and_other = reply_others_desc(n: n, n_pubkeys: pubkeys.count)
            return "Replying to \(names[0]), \(names[1])\(and_other)"
        }
        return "Replying to \(names[0]) & \(names[1])"
    }

    let and_other = reply_others_desc(n: n, n_pubkeys: pubkeys.count)
    return "Replying to \(names[0])\(and_other)"
}

func reply_others_desc(n: Int, n_pubkeys: Int) -> String {
    let other = n - n_pubkeys
    let plural = other == 1 ? "" : "s"
    return n > 1 ? " & \(other) other\(plural)" : ""
}



func make_actionbar_model(ev: NostrEvent, damus: DamusState) -> ActionBarModel {
    let likes = damus.likes.counts[ev.id]
    let boosts = damus.boosts.counts[ev.id]
    let tips = damus.tips.tips[ev.id]
    let our_like = damus.likes.our_events[ev.id]
    let our_boost = damus.boosts.our_events[ev.id]
    let our_tip = damus.tips.our_tips[ev.id]

    return ActionBarModel(likes: likes ?? 0,
                          boosts: boosts ?? 0,
                          tips: tips ?? 0,
                          our_like: our_like,
                          our_boost: our_boost,
                          our_tip: our_tip
    )
}


struct EventView_Previews: PreviewProvider {
    static var previews: some View {
        VStack {
            EventView(damus: test_damus_state(), event: NostrEvent(content: "hello there https://jb55.com/s/Oct12-150217.png https://jb55.com/red-me.jb55 cool", pubkey: "pk"), show_friend_icon: true, size: .small)
            EventView(damus: test_damus_state(), event: NostrEvent(content: "hello there https://jb55.com/s/Oct12-150217.png https://jb55.com/red-me.jb55 cool", pubkey: "pk"), show_friend_icon: true, size: .normal)
            EventView(damus: test_damus_state(), event: NostrEvent(content: "hello there https://jb55.com/s/Oct12-150217.png https://jb55.com/red-me.jb55 cool", pubkey: "pk"), show_friend_icon: true, size: .big)
            
            EventView(
                event: NostrEvent(
                    content: "hello there https://jb55.com/s/Oct12-150217.png https://jb55.com/red-me.jb55 cool",
                    pubkey: "pk",
                    createdAt: Int64(Date().timeIntervalSince1970 - 100)
                ),
                highlight: .none,
                has_action_bar: true,
                damus: test_damus_state(),
                show_friend_icon: true,
                size: .selected
            )
        }
    }
}

struct GitRefsView: View {
    let tags: [[String]]
    var onRefTapped: ((String, String)) -> Void

    var body: some View {
        VStack(alignment: .leading) {
            Text("Git References").font(.headline)
            ForEach(tags, id: \.self) { tag in
                if tag.count >= 2 {
                    Button(action: {
                        onRefTapped((tag[0], tag[1]))
                    }) {
                        HStack {
                            Text(tag[0]).font(.caption).bold()
                            Text(tag[1]).font(.caption.monospaced())
                        }
                    }
                    .buttonStyle(.plain)
                } else {}
            }
        }
        .padding()
        .background(Color.gray.opacity(0.1))
        .cornerRadius(8)
    }
}

struct _GitRefDetailView: View {
    let repo_name: String
    let refName: String
    let commitHash: String
    
    @StateObject private var repo: GitRepository
    @State private var commitMessage: String = "Loading..."

    init(repo_name: String, refName: String, commitHash: String) {
        self.repo_name = repo_name
        self.refName = refName
        self.commitHash = commitHash
        
        let localRepoLocation = documentURL.appendingPathComponent(repo_name)
        self._repo = StateObject(wrappedValue: GitRepository(localRepoLocation, credentialManager))
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(refName)
                .font(.title)
                .bold()
            
            Text(commitHash)
                .font(.body.monospaced())
                .foregroundColor(.gray)
            
            Divider()
            
            Text(commitMessage)
                .font(.body)
            
            Spacer()
        }
        .padding()
        .onAppear(perform: findCommit)
    }

    private func findCommit() {
        repo.open()
        
        if !repo.exists() {
            self.commitMessage = "Repository not found locally. Please clone it first."
            return
        }

        repo.updateCommitGraph()
        
        if let commit = repo.commitGraph.commits.first(where: { $0.id.description == commitHash }) {
            self.commitMessage = commit.message
        } else {
            self.commitMessage = "Commit not found locally. Fetching from remote..."
            let allRemotes = repo.getRemotes()
            if let remoteOrigin = allRemotes.first {
                repo.fetch(remoteOrigin)
                // After fetch, we'd ideally refresh. For now, the user needs to re-open.
                DispatchQueue.main.asyncAfter(deadline: .now() + 5) {
                    self.repo.updateCommitGraph()
                    if let commit = self.repo.commitGraph.commits.first(where: { $0.id.description == self.commitHash }) {
                        self.commitMessage = commit.message
                    } else {
                        self.commitMessage = "Commit not found after fetching."
                    }
                }
            } else {
                self.commitMessage = "Commit not found and no remote to fetch from."
            }
        }
    }
}
