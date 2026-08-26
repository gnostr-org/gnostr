import XCTest
@testable import Git

final class TestPeerProtocol: XCTestCase {
    private var urlsToRemove = [URL]()

    override func setUp() {
        Hub.session = Session()
        Hub.session.name = "peer"
        Hub.session.email = "peer@example.com"
        Hub.factory.rest = MockRest()
    }

    override func tearDown() {
        urlsToRemove.forEach { try? FileManager.default.removeItem(at: $0) }
        urlsToRemove.removeAll()
    }

    func testEnvelopeRoundTrip() throws {
        let refs = GitPeerRepositoryRefs(
            repository: "demo",
            branch: "main",
            reference: "refs/heads/main",
            commit: "abc123",
            remote: "example.com/demo.git",
            peer: "peer-1"
        )
        print("refs payload: \(refs)")
        let payload = try JSONEncoder().encode(refs)
        let envelope = GitPeerEnvelope(
            kind: .refs,
            repository: refs.repository,
            payload: String(decoding: payload, as: UTF8.self),
            peer: refs.peer
        )
        print("encoded envelope: \(envelope)")
        let data = try JSONEncoder().encode(envelope)
        let decoded = try JSONDecoder().decode(GitPeerEnvelope.self, from: data)
        print("decoded envelope: \(decoded)")
        XCTAssertEqual(.refs, decoded.kind)
        XCTAssertEqual("demo", decoded.repository)
        XCTAssertEqual("peer-1", decoded.peer)
        XCTAssertEqual(refs.commit, try JSONDecoder().decode(GitPeerRepositoryRefs.self, from: decoded.payload!.data(using: .utf8)!).commit)
    }

    func testPushRequestCarriesPackBytes() throws {
        let source = makeRepositoryURL()
        let destination = makeRepositoryURL()
        let ready = expectation(description: "source repository committed")
        var baseID = ""
        var headID = ""

        try! Data("hello world\n".utf8).write(to: source.appendingPathComponent("file.txt"))
        Hub.create(source, result: { repository in
            repository.commit([source.appendingPathComponent("file.txt")], message: "First commit\n", done: {
                baseID = try! Hub.head.id(source)
                try! Data("hello world updated\n".utf8).write(to: source.appendingPathComponent("file.txt"))
                repository.commit([source.appendingPathComponent("file.txt")], message: "Second commit\n", done: {
                    headID = try! Hub.head.id(source)
                    ready.fulfill()
                })
            })
        })

        waitForExpectations(timeout: 2)

        let pack = try Hub.pack(source, from: headID)
        let request = GitPeerPushRequest(
            repository: source.lastPathComponent,
            old: baseID,
            new: headID,
            pack: pack,
            peer: "peer-1"
        )
        let response = GitPeerPackResponse(
            repository: source.lastPathComponent,
            want: headID,
            have: baseID,
            pack: pack,
            peer: "peer-1"
        )

        let requestData = try JSONEncoder().encode(request)
        let decodedRequest = try JSONDecoder().decode(GitPeerPushRequest.self, from: requestData)
        let responseData = try JSONEncoder().encode(response)
        let decodedResponse = try JSONDecoder().decode(GitPeerPackResponse.self, from: responseData)

        print("push request decoded repository=\(decodedRequest.repository) old=\(decodedRequest.old) new=\(decodedRequest.new) peer=\(decodedRequest.peer)")
        XCTAssertEqual(pack, Data(base64Encoded: decodedRequest.pack))
        XCTAssertEqual(pack, Data(base64Encoded: decodedResponse.pack))

        let destinationReady = expectation(description: "destination repository created")
        Hub.create(destination, result: { _ in
            destinationReady.fulfill()
        })
        waitForExpectations(timeout: 2)

        try Hub.unpack(Data(base64Encoded: decodedRequest.pack)!, url: destination)
        try Hub.update(destination, id: headID)
        try Hub.origin(destination, id: headID)

        XCTAssertEqual(headID, try Hub.head.id(destination))
    }

    private func makeRepositoryURL() -> URL {
        let url = URL(fileURLWithPath: NSTemporaryDirectory()).appendingPathComponent(UUID().uuidString)
        urlsToRemove.append(url)
        try! FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
        return url
    }
}
