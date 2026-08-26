import Foundation
import Testing
@testable import GnostrTypes

@Test func eventKindConstantsMatchRust() {
    #expect(NIP34.repoAnnouncementKind == 30617)
    #expect(NIP34.repoStateKind == 30618)
    #expect(NIP34.patchesKind == 1617)
    #expect(EventKind.longFormContent.rawValue == 30023)
    #expect(EventKind.repoAnnouncement.rawValue == 30617)
    #expect(EventKind.gitRepoAnnouncement.rawValue == 30618)
    #expect(statusKinds() == [.gitStatusOpen, .gitStatusApplied, .gitStatusClosed, .gitStatusDraft])
}

@Test func gitNoteCodableRoundTrip() throws {
    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )

    let data = try JSONEncoder().encode(note)
    let decoded = try JSONDecoder().decode(GitNote.self, from: data)
    #expect(decoded == note)
}

@Test func tagHelpersBuildExpectedFields() {
    let id = Id(hex: "0123")
    let key = PublicKey(hex: "abcd")

    #expect(Tag.event(id, marker: "root").fields == ["e", "0123", "root"])
    #expect(Tag.pubkey(key, relay: "wss://example.com", marker: "reply").fields == ["p", "abcd", "wss://example.com", "reply"])
}

@Test func rustJsonShapesMatch() throws {
    let event = Event(
        id: Id(hex: "deadbeef"),
        pubkey: PublicKey(hex: "cafebabe"),
        createdAt: Unixtime(1234),
        kind: .patches,
        sig: Signature(hex: "facefeed"),
        content: "hello",
        tags: [Tag.event(Id(hex: "0123"), marker: "root")]
    )

    let data = try JSONEncoder().encode(event)
    let json = String(decoding: data, as: UTF8.self)
    #expect(json.contains(#""created_at":1234"#))
    #expect(json.contains(#""tags":[["e","0123","root"]]"#))
    #expect(json.contains(#""id":"deadbeef""#))
    #expect(json.contains(#""sig":"facefeed""#))
}

@Test func coreModelsCodableRoundTrip() throws {
    let metadata = Metadata(
        name: "alice",
        about: nil,
        picture: "https://example.com/p.png",
        nip05: "alice@example.com",
        other: ["lud16": .string("alice@example.com")]
    )
    #expect(try JSONDecoder().decode(Metadata.self, from: JSONEncoder().encode(metadata)) == metadata)

    let filter = Filter(
        ids: [Id(hex: "deadbeef")],
        authors: [PublicKey(hex: "cafebabe")],
        kinds: [.textNote, .longFormContent],
        tags: ["#e": ["abc"], "#p": ["def"]],
        since: Unixtime(1),
        limit: 10
    )
    let event = Event(
        id: Id(hex: "deadbeef"),
        pubkey: PublicKey(hex: "cafebabe"),
        createdAt: Unixtime(1234),
        kind: .textNote,
        sig: Signature(hex: "facefeed"),
        content: "hello",
        tags: [Tag.event(Id(hex: "0123"), marker: "root")]
    )
    let filterJSON = String(decoding: try JSONEncoder().encode(filter), as: UTF8.self)
    #expect(filterJSON.contains("\"#e\":[\"abc\"]"))
    #expect(filterJSON.contains("\"#p\":[\"def\"]"))

    let relayDoc = RelayInformationDocument(
        name: "Relay",
        description: "desc",
        supportedNips: [11, 12],
        retention: [RelayRetention(kinds: [EventKindOrRange(.kind(.metadata))], time: 3600)],
        relayCountries: ["US"],
        languageTags: ["en"],
        tags: ["sfw"],
        other: ["anything": .null]
    )
    #expect(try JSONDecoder().decode(RelayInformationDocument.self, from: JSONEncoder().encode(relayDoc)) == relayDoc)

    let payReq = PayRequestData(
        callback: UncheckedUrl("https://example.com/pay"),
        metadata: [.init(key: "text/plain", value: "hello")],
        allowsNostr: true,
        nostrPubkey: PublicKey(hex: "cafebabe"),
        other: ["tag": .string("payRequest")]
    )
    #expect(try JSONDecoder().decode(PayRequestData.self, from: JSONEncoder().encode(payReq)) == payReq)

    let simpleRelayList = SimpleRelayList([
        UncheckedUrl("wss://relay.example.com"): SimpleRelayUsage(write: true, read: true)
    ])
    #expect(try JSONDecoder().decode(SimpleRelayList.self, from: JSONEncoder().encode(simpleRelayList)) == simpleRelayList)

    let relayList = RelayList([
        UncheckedUrl("wss://relay.example.com"): .both
    ])
    #expect(try JSONDecoder().decode(RelayList.self, from: JSONEncoder().encode(relayList)) == relayList)

    let clientMessage: ClientMessage = .req(SubscriptionId("sub-1"), [filter])
    #expect(try JSONDecoder().decode(ClientMessage.self, from: JSONEncoder().encode(clientMessage)) == clientMessage)

    let relayMessage: RelayMessage = .event(SubscriptionId("sub-1"), event)
    #expect(try JSONDecoder().decode(RelayMessage.self, from: JSONEncoder().encode(relayMessage)) == relayMessage)
}

@Test func rustBridgeRoundTripsWhenAvailable() throws {
    guard NIP34.rustBridgeAvailable else { return }

    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello from rust",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )

    #expect(NIP34.rustGitNoteEventID(commitID: note.annotatedID) != nil)

    let tags = try NIP34.rustGitNoteTags(note: note)
    #expect(tags.contains(where: { $0.tagName() == "commit" }))
    #expect(tags.contains(where: { $0.tagName() == "notes-ref" }))

    let event = try NIP34.rustGenerateGitNoteEvent(
        note: note,
        privateKeyHex: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )
    #expect(event.kind == .patches)
    #expect(event.content == note.message)
    #expect(event.tags.contains(where: { $0.tagName() == "commit" }))
}

@Test func rustCoreRoundTripsWhenAvailable() throws {
    guard RustGnostrTypesBridge.shared.isAvailable else { return }

    let tag = Tag.event(Id(hex: "0123"), marker: "root")
    #expect(try NIP34.rustNormalizeTag(tag) == tag)

    let preEvent = PreEvent(
        pubkey: PublicKey(hex: "cafebabe"),
        createdAt: Unixtime(1234),
        kind: .longFormContent,
        tags: [tag],
        content: "hello"
    )
    #expect(try NIP34.rustNormalizePreEvent(preEvent) == preEvent)

    let event = Event(
        id: Id(hex: "deadbeef"),
        pubkey: PublicKey(hex: "cafebabe"),
        createdAt: Unixtime(1234),
        kind: .longFormContent,
        sig: Signature(hex: "facefeed"),
        content: "hello",
        tags: [tag]
    )
    #expect(try NIP34.rustNormalizeEvent(event) == event)

    let naddr = NAddr(
        d: "identifier",
        relays: ["wss://relay.example.com"],
        kind: .longFormContent,
        author: PublicKey(hex: "cafebabe")
    )
    #expect(try NIP34.rustNormalizeNAddr(naddr) == naddr)

    let metadata = Metadata(name: "alice", other: ["testing": .string("123")])
    #expect(try RustGnostrTypesBridge.shared.normalize(metadata) == metadata)

    let profile = Profile(pubkey: PublicKey(hex: "cafebabe"), relays: [UncheckedUrl("wss://relay.example.com")])
    #expect(try RustGnostrTypesBridge.shared.normalize(profile) == profile)

    let nprofile = NProfile(pubkey: PublicKey(hex: "cafebabe"), relays: [UncheckedUrl("wss://relay.example.com")])
    #expect(try RustGnostrTypesBridge.shared.normalize(nprofile) == nprofile)

    let filter = Filter(ids: [Id(hex: "deadbeef")], kinds: [.textNote])
    #expect(try RustGnostrTypesBridge.shared.normalize(filter) == filter)

    let relayDoc = RelayInformationDocument(name: "relay", supportedNips: [11])
    #expect(try RustGnostrTypesBridge.shared.normalize(relayDoc) == relayDoc)

    let subscriptionId = SubscriptionId("sub-1")
    #expect(try RustGnostrTypesBridge.shared.normalize(subscriptionId) == subscriptionId)

    let dims = ImageDimensions(width: 1280, height: 720)
    #expect(try RustGnostrTypesBridge.shared.normalize(dims) == dims)
}

@Test func leadingZeroBitsCountMatchesExamples() {
    #expect(getLeadingZeroBits([0x00, 0x00, 0x10]) == 19)
    #expect(getLeadingZeroBits([0xff]) == 0)
}
