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
