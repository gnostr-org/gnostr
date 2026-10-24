//
//  damusTests.swift
//  damusTests
//
//  Created by William Casarin on 2022-04-01.
//

import XCTest
@testable import gnostr

class damusTests: XCTestCase {

    private func relayListEvent(tags: [[String]], kind: Int = 10002) -> NostrEvent {
        let privkey = String(repeating: "0", count: 63) + "1"
        let event = NostrEvent(content: "", pubkey: privkey_to_pubkey(privkey: privkey)!,
                               kind: kind, tags: tags)
        event.calculate_id()
        event.sign(privkey: privkey)
        return event
    }

    func testRelayRecommendationsNIP65Markers() {
        let event = relayListEvent(tags: [
            ["r", "wss://relay.damus.io", "read"],
            ["r", "wss://nos.lol", "write"],
            ["r", "wss://relay.nostr.band"],
        ])
        let recommendations = relay_recommendations(event)
        XCTAssertEqual(recommendations.map { $0.url.absoluteString },
                       ["wss://relay.damus.io", "wss://nos.lol", "wss://relay.nostr.band"])
        XCTAssertEqual(recommendations.map { $0.info.read }, [true, false, true])
        XCTAssertEqual(recommendations.map { $0.info.write }, [false, true, true])
    }

    func testRelayRecommendationsIgnoreNonRelayTags() {
        let event = relayListEvent(tags: [
            [], ["r"], ["r", ""], ["r", "https://example.com"],
            ["r", "wss://"], ["r", "wss://relay.example/a b"],
            ["r", "wss://user:password@relay.example"],
            ["r", "wss://relay.example#fragment"],
            ["r", "wss://relay.example", "unsupported"],
            ["e", "wss://not-a-relay.example"],
            ["r", "WS://RELAY.EXAMPLE:80/", ""],
            ["r", "wss://relay.example/path?token=value", "read"],
        ])
        XCTAssertEqual(relay_recommendations(event).map { $0.url.absoluteString },
                       ["ws://relay.example", "wss://relay.example/path?token=value"])
    }

    func testRelayRecommendationsCollectedAcrossSubscriptions() throws {
        let pool = RelayPool(defaults: nil)
        let event = relayListEvent(tags: [
            ["r", "wss://DISCOVERED.example:443/", "read"],
            ["r", "wss://discovered.example", "write"],
        ])
        pool.handle_event(relay_id: "wss://source.example",
                          event: .nostr_event(.event("other-view", event)))
        pool.handle_event(relay_id: "wss://other-source.example",
                          event: .nostr_event(.event("another-view", event)))
        XCTAssertEqual(pool.suggested_relays.count, 1)
        XCTAssertTrue(pool.suggested_relays[0].info.read)
        XCTAssertTrue(pool.suggested_relays[0].info.write)
        XCTAssertTrue(pool.relays.isEmpty)
        XCTAssertEqual(pool.recommended_relays.filter {
            $0.url.absoluteString == "wss://discovered.example"
        }.count, 1)

        try pool.add_relay(URL(string: "wss://discovered.example/")!, info: .rw)
        XCTAssertFalse(pool.recommended_relays.contains {
            $0.url.absoluteString == "wss://discovered.example"
        })
        pool.remove_relay("wss://discovered.example/")
        XCTAssertTrue(pool.recommended_relays.contains {
            $0.url.absoluteString == "wss://discovered.example"
        })
    }

    func testRelayRecommendationsFromOtherKindsAndInvalidEvents() {
        let pool = RelayPool(defaults: nil)
        let event = relayListEvent(tags: [
            ["r", "wss://git-relay.example"],
            ["r", "https://repository.example/project"],
        ], kind: NostrKind.repository_announcement.rawValue)
        pool.collect_relay_recommendations(event)
        XCTAssertEqual(pool.suggested_relays.map { $0.url.absoluteString },
                       ["wss://git-relay.example"])

        let invalid = relayListEvent(tags: [["r", "wss://invalid.example"]])
        invalid.sig = String(repeating: "0", count: 128)
        pool.collect_relay_recommendations(invalid)
        XCTAssertEqual(pool.suggested_relays.count, 1)
    }

    func testRelayRecommendationsReturnAfterManualRelayRemoval() throws {
        let pool = RelayPool(defaults: nil)
        let url = URL(string: "wss://manual.example/")!
        try pool.add_relay(url, info: RelayInfo(read: true, write: false))
        XCTAssertFalse(pool.recommended_relays.contains { $0.url.host == "manual.example" })
        pool.remove_relay(url.absoluteString)
        let restored = try XCTUnwrap(pool.recommended_relays.first { $0.url.host == "manual.example" })
        XCTAssertTrue(restored.info.read)
        XCTAssertFalse(restored.info.write)
        XCTAssertEqual(pool.recommended_relays.filter { $0.url.host == "manual.example" }.count, 1)
    }

    func testRelayRecommendationsIgnoreSurvivesRediscoveryAndRemoval() throws {
        let pool = RelayPool(defaults: nil)
        let descriptor = RelayDescriptor(url: URL(string: "wss://IGNORED.example:443/")!, info: .rw)
        pool.set_relay_ignored(descriptor, ignored: true)
        pool.collect_relay_recommendations(relayListEvent(tags: [["r", "wss://ignored.example"]]))
        XCTAssertFalse(pool.recommended_relays.contains { $0.url.host == "ignored.example" })
        XCTAssertEqual(pool.available_recommendations(show_ignored: true).filter {
            $0.url.host == "ignored.example"
        }.count, 1)

        try pool.add_relay(descriptor.url, info: descriptor.info)
        XCTAssertFalse(pool.available_recommendations(show_ignored: true).contains {
            $0.url.host == "ignored.example"
        })
        pool.remove_relay(descriptor.url.absoluteString)
        XCTAssertFalse(pool.recommended_relays.contains { $0.url.host == "ignored.example" })
        pool.set_relay_ignored(descriptor, ignored: false)
        XCTAssertEqual(pool.recommended_relays.filter { $0.url.host == "ignored.example" }.count, 1)
    }

    func testRelayRecommendationsIgnorePersistsAndCanBeRestored() throws {
        let suite = "RelayRecommendationsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let relay = RelayDescriptor(url: URL(string: "wss://saved.example/")!, info: .rw)
        RelayPool(defaults: defaults).set_relay_ignored(relay, ignored: true)

        let restored = RelayPool(defaults: defaults)
        XCTAssertTrue(restored.is_relay_ignored(URL(string: "wss://saved.example")!))
        XCTAssertFalse(restored.recommended_relays.contains { $0.url.host == "saved.example" })
        let ignored = try XCTUnwrap(restored.available_recommendations(show_ignored: true).first {
            $0.url.host == "saved.example"
        })
        restored.set_relay_ignored(ignored, ignored: false)
        XCTAssertTrue(RelayPool(defaults: defaults).ignored_relays.isEmpty)
        XCTAssertTrue(restored.recommended_relays.contains { $0.url.host == "saved.example" })
    }

    func testRelayInformationRequestUsesHTTPAndNIP11AcceptHeader() throws {
        let secure = try relay_information_request(URL(string: "wss://relay.example:8443/info?key=value")!)
        XCTAssertEqual(secure.url?.absoluteString, "https://relay.example:8443/info?key=value")
        XCTAssertEqual(secure.value(forHTTPHeaderField: "Accept"), "application/nostr+json")
        let insecure = try relay_information_request(URL(string: "ws://localhost:8080/")!)
        XCTAssertEqual(insecure.url?.absoluteString, "http://localhost:8080")
        XCTAssertThrowsError(try relay_information_request(URL(string: "https://relay.example")!))
    }

    func testRelayInformationDetectsAdvertisedNIP34() throws {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        let supported = try decoder.decode(RelayInformation.self, from: Data("""
        {"supported_nips":[1,11,34,65],"fees":{"publication":[{"amount":1,"unit":"sats"}]}}
        """.utf8))
        XCTAssertTrue(supported.supportsNIP34)
        let unsupported = try decoder.decode(RelayInformation.self,
                                             from: Data(#"{"supported_nips":[1,11]}"#.utf8))
        XCTAssertFalse(unsupported.supportsNIP34)
        let unknown = try decoder.decode(RelayInformation.self, from: Data("{}".utf8))
        XCTAssertNil(unknown.supportedNips)
    }

    private func relayInformation(nips: [Int]) throws -> RelayInformation {
        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase
        return try decoder.decode(RelayInformation.self, from: Data(
            "{\"supported_nips\":[\(nips.map(String.init).joined(separator: ","))]}".utf8))
    }

    @MainActor
    func testRelayAutoSubscribeDefaultsOnAndHonorsRemoveAndIgnore() async throws {
        let metadata = try relayInformation(nips: [1, 11, 34])
        var requests = 0
        var connections: [String] = []
        let pool = RelayPool(defaults: nil, information_fetcher: { _ in
            requests += 1
            return metadata
        }, relay_connector: { connections.append($0.id) })
        let url = URL(string: "wss://automatic.example")!
        let descriptor = RelayDescriptor(url: url, info: .rw)
        XCTAssertTrue(pool.auto_subscribe_nip34)
        _ = try await pool.load_relay_information(url)
        _ = try await pool.load_relay_information(URL(string: "wss://AUTOMATIC.example:443/")!)
        XCTAssertEqual(requests, 1)
        XCTAssertEqual(connections, [url.absoluteString])
        XCTAssertNotNil(pool.get_relay(url.absoluteString))
        XCTAssertFalse(pool.recommended_relays.contains { $0.url == url })

        pool.remove_relay(url.absoluteString)
        _ = try await pool.load_relay_information(url)
        XCTAssertNil(pool.get_relay(url.absoluteString))
        XCTAssertTrue(pool.recommended_relays.contains { $0.url == url })
        XCTAssertEqual(connections.count, 1)

        pool.allow_relay(url)
        _ = try await pool.load_relay_information(url)
        XCTAssertNotNil(pool.get_relay(url.absoluteString))
        pool.set_relay_ignored(descriptor, ignored: true)
        _ = try await pool.load_relay_information(url)
        XCTAssertNil(pool.get_relay(url.absoluteString))
        XCTAssertFalse(pool.recommended_relays.contains { $0.url == url })
        XCTAssertEqual(connections.count, 2)

        pool.set_relay_ignored(descriptor, ignored: false)
        _ = try await pool.load_relay_information(url)
        XCTAssertNotNil(pool.get_relay(url.absoluteString))
        XCTAssertEqual(connections.count, 3)
    }

    @MainActor
    func testRelayAutoSubscribeOnlyAddsAdvertisedNIP34WhenEnabled() async throws {
        let supported = try relayInformation(nips: [34])
        let unsupported = try relayInformation(nips: [1, 11])
        let pool = RelayPool(defaults: nil, information_fetcher: { url in
            return url.host == "git.example" ? supported : unsupported
        }, relay_connector: { _ in })
        pool.auto_subscribe_nip34 = false
        let url = URL(string: "wss://git.example")!
        _ = try await pool.load_relay_information(url)
        XCTAssertTrue(pool.relays.isEmpty)
        pool.auto_subscribe_nip34 = true
        XCTAssertNotNil(pool.get_relay(url.absoluteString))
        _ = try await pool.load_relay_information(URL(string: "wss://other.example")!)
        XCTAssertEqual(pool.relays.count, 1)
    }

    @MainActor
    func testRelayAutoSubscribeRemovalAndPreferencePersist() async throws {
        let suite = "RelayAutoSubscribeTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let metadata = try relayInformation(nips: [34])
        let pool = RelayPool(defaults: defaults, information_fetcher: { _ in metadata },
                             relay_connector: { _ in })
        let url = URL(string: "wss://removed.example")!
        _ = try await pool.load_relay_information(url)
        pool.remove_relay(url.absoluteString)
        pool.auto_subscribe_nip34 = false

        let restored = RelayPool(defaults: defaults, information_fetcher: { _ in metadata },
                                 relay_connector: { _ in })
        XCTAssertFalse(restored.auto_subscribe_nip34)
        restored.auto_subscribe_nip34 = true
        _ = try await restored.load_relay_information(url)
        XCTAssertTrue(restored.relays.isEmpty)
        XCTAssertTrue(restored.recommended_relays.contains { $0.url == url })
        restored.allow_relay(url)
        _ = try await restored.load_relay_information(url)
        XCTAssertNotNil(restored.get_relay(url.absoluteString))
    }

    @MainActor
    func testRelayAutoSubscribeDiscoversTagsWithoutSettings() async throws {
        let supported = try relayInformation(nips: [34])
        let unsupported = try relayInformation(nips: [1])
        let connected = expectation(description: "Discovered NIP-34 relay connected")
        let url = URL(string: "wss://discovered-git.example")!
        let pool = RelayPool(defaults: nil, information_fetcher: { relay in
            return relay == url ? supported : unsupported
        }, relay_connector: { relay in
            if relay.id == url.absoluteString { connected.fulfill() }
        })
        pool.connect(to: [])
        pool.collect_relay_recommendations(relayListEvent(tags: [["r", url.absoluteString]]))
        await fulfillment(of: [connected], timeout: 5)
        XCTAssertNotNil(pool.get_relay(url.absoluteString))
    }

    func testGnostrDiscoveryFiltersEncodeHashtagAndTextAsAlternatives() throws {
        let hashtagOnly = gnostr_discovery_filters(supports_search: false)
        XCTAssertEqual(hashtagOnly.count, 1)
        XCTAssertEqual(hashtagOnly[0].hashtag, ["gnostr"])
        XCTAssertNil(hashtagOnly[0].authors)
        XCTAssertNil(hashtagOnly[0].until)
        let filters = gnostr_discovery_filters(supports_search: true, since: 123)
        let encoded = try JSONEncoder().encode(filters)
        let objects = try XCTUnwrap(JSONSerialization.jsonObject(with: encoded) as? [[String: Any]])
        XCTAssertEqual(objects.count, 2)
        XCTAssertEqual(objects[0]["#t"] as? [String], ["gnostr"])
        XCTAssertNil(objects[0]["search"])
        XCTAssertEqual(objects[1]["search"] as? String, "gnostr")
        XCTAssertNil(objects[1]["#t"])
        XCTAssertEqual(filters.map { $0.limit }, [500, 500])
        XCTAssertEqual(filters.map { $0.since }, [123, 123])
        XCTAssertEqual(NostrFilter.copy(from: filters[1]).search, "gnostr")
    }

    func testGnostrDiscoveryRoutesMatchingPostsAndGitEventsAndRejectsNoise() {
        let home = HomeModel()
        let tagged = relayListEvent(tags: [["t", "gnostr"]], kind: 1)
        let text = NostrEvent(content: "Building GNOSTR", pubkey: tagged.pubkey, kind: 1618)
        text.calculate_id()
        text.sign(privkey: String(repeating: "0", count: 63) + "1")
        let unrelated = relayListEvent(tags: [["t", "other"]], kind: 1)
        let invalid = relayListEvent(tags: [["t", "gnostr"]], kind: 1621)
        invalid.sig = String(repeating: "0", count: 128)
        for event in [tagged, text, tagged, unrelated, invalid] {
            home.handle_event(relay_id: "wss://source.example",
                              conn_event: .nostr_event(.event(home.gnostr_subid, event)))
        }
        XCTAssertEqual(Set(home.events.map { $0.id }), Set([tagged.id, text.id]))
        home.loading = true
        home.handle_event(relay_id: "wss://source.example",
                          conn_event: .nostr_event(.eose(home.gnostr_subid)))
        XCTAssertTrue(home.loading)
        home.handle_event(relay_id: "wss://source.example",
                          conn_event: .nostr_event(.eose(home.home_subid)))
        XCTAssertFalse(home.loading)
    }

    func testGlobalSearchNIP34ToggleChangesSubscriptionAndAcceptsMultipleGitEvents() {
        let model = SearchHomeModel(damus_state: .empty)
        XCTAssertTrue(model.get_base_filter().kinds?.contains(1) == true)
        model.subscribe(nip34_only: true)
        XCTAssertFalse(model.get_base_filter().kinds?.contains(1) == true)
        XCTAssertTrue(model.get_base_filter().kinds?.contains(1618) == true)
        XCTAssertTrue(model.get_base_filter().kinds?.contains(1621) == true)
        let post = relayListEvent(tags: [], kind: 1)
        let issue = relayListEvent(tags: [], kind: 1621)
        let pullRequest = relayListEvent(tags: [], kind: 1618)
        for event in [post, issue, pullRequest] {
            model.handle_event(relay_id: "wss://source.example",
                               conn_ev: .nostr_event(.event(model.base_subid, event)))
        }
        XCTAssertEqual(Set(model.events.map { $0.id }), Set([issue.id, pullRequest.id]))
        model.unsubscribe()
        model.subscribe(nip34_only: false)
        XCTAssertTrue(model.events.isEmpty)
        XCTAssertTrue(model.get_base_filter().kinds?.contains(1) == true)
        model.unsubscribe()
    }

    func testSearchNIP34ToggleKeepsExplicitScopeAndFiltersOtherSubscriptions() {
        let pool = RelayPool(defaults: nil)
        let model = SearchModel(pool: pool, search: .filter_hashtag(["gnostr"]))
        model.subscribe(nip34_only: true)
        let post = relayListEvent(tags: [["t", "gnostr"]], kind: 1)
        let issue = relayListEvent(tags: [["t", "gnostr"]], kind: 1621)
        let untagged = relayListEvent(tags: [], kind: 1618)
        for event in [post, issue, untagged] {
            model.handle_event(relay_id: "wss://source.example",
                               ev: .nostr_event(.event(model.sub_id, event)))
        }
        model.handle_event(relay_id: "wss://source.example",
                           ev: .nostr_event(.event("another-subscription", untagged)))
        XCTAssertEqual(model.events.map { $0.id }, [issue.id])
        model.unsubscribe()
        model.subscribe(nip34_only: false)
        XCTAssertTrue(model.events.isEmpty)
        model.add_event(post)
        XCTAssertEqual(model.events.map { $0.id }, [post.id])
        model.unsubscribe()

        let scoped = SearchModel(pool: pool, search: .filter_kinds([30617]))
        scoped.subscribe(nip34_only: true)
        XCTAssertEqual(scoped.search.kinds, [30617])
        scoped.unsubscribe()
        scoped.subscribe(nip34_only: false)
        XCTAssertEqual(scoped.search.kinds, [30617])
        scoped.unsubscribe()
    }

    func testGlobalSearchNIP34KindSelectionsRejectDisabledKindsAndStopWhenEmpty() {
        let model = SearchHomeModel(damus_state: .empty)
        let patch = relayListEvent(tags: [], kind: 1617)
        let issue = relayListEvent(tags: [], kind: 1621)
        model.subscribe(nip34_only: true, selected_kinds: [1617])
        XCTAssertEqual(model.get_base_filter().kinds, [1617])
        for event in [patch, issue] {
            model.handle_event(relay_id: "wss://source.example",
                               conn_ev: .nostr_event(.event(model.base_subid, event)))
        }
        XCTAssertEqual(model.events.map { $0.id }, [patch.id])
        model.subscribe(nip34_only: true, selected_kinds: [1621])
        XCTAssertTrue(model.events.isEmpty)
        for event in [patch, issue] {
            model.handle_event(relay_id: "wss://source.example",
                               conn_ev: .nostr_event(.event(model.base_subid, event)))
        }
        XCTAssertEqual(model.events.map { $0.id }, [issue.id])
        model.subscribe(nip34_only: true, selected_kinds: [])
        XCTAssertTrue(model.events.isEmpty)
        XCTAssertFalse(model.loading)
        XCTAssertFalse(model.damus_state.pool.handlers.contains { $0.sub_id == model.base_subid })
    }

    func testSearchNIP34KindSelectionsKeepScopeAndStopWhenEmpty() {
        let pool = RelayPool(defaults: nil)
        let model = SearchModel(pool: pool, search: .filter_hashtag(["gnostr"]))
        let patch = relayListEvent(tags: [["t", "gnostr"]], kind: 1617)
        let issue = relayListEvent(tags: [["t", "gnostr"]], kind: 1621)
        model.subscribe(nip34_only: true, selected_kinds: [1617, 1621])
        model.add_event(patch)
        model.add_event(issue)
        XCTAssertEqual(model.events.count, 2)
        model.subscribe(nip34_only: true, selected_kinds: [1617])
        model.add_event(patch)
        model.add_event(issue)
        XCTAssertEqual(model.events.map { $0.id }, [patch.id])
        model.subscribe(nip34_only: true, selected_kinds: [])
        XCTAssertTrue(model.events.isEmpty)
        XCTAssertFalse(model.loading)
        XCTAssertFalse(pool.handlers.contains { $0.sub_id == model.sub_id })

        let scoped = SearchModel(pool: pool, search: .filter_kinds([30617]))
        scoped.subscribe(nip34_only: true, selected_kinds: [1617])
        XCTAssertEqual(scoped.search.kinds, [])
        XCTAssertFalse(scoped.loading)
        scoped.subscribe(nip34_only: false, selected_kinds: [])
        XCTAssertEqual(scoped.search.kinds, [30617])
        scoped.unsubscribe()
    }

    func testSearchNIP34KindSelectionsPersistIncludingAllOff() throws {
        let suite = "SearchKindSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let settings = UserSettingsStore(defaults: defaults)
        settings.search_nip34_kinds.remove(1617)
        settings.search_nip34_kinds.remove(1621)
        let restored = UserSettingsStore(defaults: defaults)
        XCTAssertFalse(restored.search_nip34_kinds.contains(1617))
        XCTAssertFalse(restored.search_nip34_kinds.contains(1621))
        XCTAssertTrue(restored.search_nip34_kinds.contains(30617))
        restored.search_nip34_kinds = []
        XCTAssertTrue(UserSettingsStore(defaults: defaults).search_nip34_kinds.isEmpty)
        restored.search_nip34_kinds.insert(1621)
        XCTAssertEqual(UserSettingsStore(defaults: defaults).search_nip34_kinds, [1621])
    }

    func testPatchPreviewFoldsBelowFourCommitHeaderLines() throws {
        let hash = String(repeating: "1234567890", count: 4)
        let header = """
        From \(hash) Mon Sep 17 00:00:00 2001
        From: Example <example@example.com>
        Date: Sat, 24 Oct 2026 12:00:00 +0000
        Subject: [PATCH] Keep the complete commit header
        """
        let body = """

        Commit message.
        ---
        diff --git a/file b/file
        --- a/file
        +++ b/file
        @@ -1 +1 @@
        -old
        +new
        """
        let preview = try XCTUnwrap(PatchPreview(content: header + "\n" + body))
        XCTAssertEqual(preview.header, header)
        XCTAssertEqual(preview.body, body)
        XCTAssertTrue(preview.header.contains(hash))
        XCTAssertFalse(preview.header.contains("diff --git"))
    }

    func testPatchPreviewDoesNotFoldShortPatchesOrTrailingBlankLines() {
        XCTAssertNil(PatchPreview(content: ""))
        XCTAssertNil(PatchPreview(content: "one\ntwo\nthree\nfour"))
        XCTAssertNil(PatchPreview(content: "one\ntwo\nthree\nfour\n\n \t\n"))
        XCTAssertNotNil(PatchPreview(content: "one\ntwo\nthree\nfour\nfive"))
    }

    func testPatchPreviewHandlesCRLFAndPreservesDiffWhitespaceAndURLs() throws {
        let content = "From commit\r\nFrom: author\r\nDate: date\r\nSubject: change\r\n\r\n+\t  indented\r\n+https://example.com/image.png\r\n+#not-markdown\r\n"
        let preview = try XCTUnwrap(PatchPreview(content: content))
        XCTAssertEqual(preview.header, "From commit\nFrom: author\nDate: date\nSubject: change")
        XCTAssertEqual(preview.body, "\n+\t  indented\n+https://example.com/image.png\n+#not-markdown\n")
    }

    override func setUpWithError() throws {
        // Put setup code here. This method is called before the invocation of each test method in the class.
    }

    override func tearDownWithError() throws {
        // Put teardown code here. This method is called after the invocation of each test method in the class.
    }

    func testExample() throws {
        // This is an example of a functional test case.
        // Use XCTAssert and related functions to verify your tests produce the correct results.
        // Any test you write for XCTest can be annotated as throws and async.
        // Mark your test throws to produce an unexpected failure when your test encounters an uncaught error.
        // Mark your test async to allow awaiting for asynchronous code to complete. Check the results with assertions afterwards.
    }

    func testPerformanceExample() throws {
        // This is an example of a performance test case.
        self.measure {
            // Put the code you want to measure the time of here.
        }
    }
    
    func testParseMentionWithMarkdown() {
        let md = """
        Testing markdown in damus
        
        **bold**

        _italics_

        `monospace`

        # h1

        ## h2

        ### h3

        * list1
        * list2

        > some awesome quote

        [my website](https://jb55.com)
        """
        
        let parsed = parse_mentions(content: md, tags: [])
        
        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 1)
        XCTAssertNotNil(parsed[0].is_text)
    }
    
    func testParseUrlUpper() {
        let parsed = parse_mentions(content: "a HTTPS://jb55.COM b", tags: [])

        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 3)
        XCTAssertEqual(parsed[1].is_url?.absoluteString, "HTTPS://jb55.COM")
    }
    
    func testParseUrl() {
        let parsed = parse_mentions(content: "a https://jb55.com b", tags: [])

        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 3)
        XCTAssertEqual(parsed[1].is_url?.absoluteString, "https://jb55.com")
    }
    
    func testParseUrlEnd() {
        let parsed = parse_mentions(content: "a https://jb55.com", tags: [])

        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 2)
        XCTAssertEqual(parsed[0].is_text, "a ")
        XCTAssertEqual(parsed[1].is_url?.absoluteString, "https://jb55.com")
    }
    
    func testParseUrlStart() {
        let parsed = parse_mentions(content: "https://jb55.com br", tags: [])

        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 2)
        XCTAssertEqual(parsed[0].is_url?.absoluteString, "https://jb55.com")
        XCTAssertEqual(parsed[1].is_text, " br")
    }
    
    func testParseMentionBlank() {
        let parsed = parse_mentions(content: "", tags: [["e", "event_id"]])
        
        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 0)
    }
    
    func testMakeHashtagPost() {
        let privkey = "d05f5fcceef3e4529703f62a29222d6ee2d1b7bf1f24729b5e01df7c633cec8a"
        let pubkey = "6e59d3b78b1c1490a6489c94405873b57d8ef398a830ae5e39608f4107e9a790"
        let post = NostrPost(content: "#damus some content #bitcoin derp", references: [])
        let ev = post_to_event(post: post, privkey: privkey, pubkey: pubkey)
        
        XCTAssertEqual(ev.tags.count, 2)
        XCTAssertEqual(ev.content, "#damus some content #bitcoin derp")
        XCTAssertEqual(ev.tags[0][0], "t")
        XCTAssertEqual(ev.tags[0][1], "damus")
        XCTAssertEqual(ev.tags[1][0], "t")
        XCTAssertEqual(ev.tags[1][1], "bitcoin")
        
    }
    
    func testParseHashtag() {
        let parsed = parse_mentions(content: "some hashtag #bitcoin derp", tags: [])
        
        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 3)
        XCTAssertEqual(parsed[0].is_text, "some hashtag ")
        XCTAssertEqual(parsed[1].is_hashtag, "bitcoin")
        XCTAssertEqual(parsed[2].is_text, " derp")
    }
    
    func testParseHashtagEnd() {
        let parsed = parse_mentions(content: "some hashtag #bitcoin", tags: [])
        
        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 2)
        XCTAssertEqual(parsed[0].is_text, "some hashtag ")
        XCTAssertEqual(parsed[1].is_hashtag, "bitcoin")
    }
    
    func testParseMentionOnlyText() {
        let parsed = parse_mentions(content: "there is no mention here", tags: [["e", "event_id"]])
        
        XCTAssertNotNil(parsed)
        XCTAssertEqual(parsed.count, 1)
        XCTAssertEqual(parsed[0].is_text, "there is no mention here")
        
        guard case .text(let txt) = parsed[0] else {
            XCTAssertTrue(false)
            return
        }
        
        XCTAssertEqual(txt, "there is no mention here")
    }

}
