import Foundation
@testable import GnostrSDK
@testable import NostrSDKDemo
import XCTest

final class NostrSDKDemoSmokeTests: XCTestCase {
    func testPrimeStoreRecordsCloneTagEvents() throws {
        let store = DemoAppPrimeStore()
        let cloneURL = RepoURLs.cloneTag
        let expectedURL = RepoURLs.ssh
        let event = NostrEvent(
            kind: .unknown(30617),
            content: "",
            tags: [
                Tag(name: .identifier, value: RepoURLs.identifier),
                Tag(name: "clone", value: cloneURL)
            ],
            createdAt: 1,
            pubkey: RepoURLs.pubkey
        )

        store.record(event: event)

        XCTAssertEqual(store.seenRepositoryURLs, [expectedURL])
        XCTAssertEqual(store.repositoryEventByRepoIDAndKind[RepoURLs.identifier]?[30617]?.id, event.id)
    }

    func testNormalizesRepositoryCloneURLs() {
        for form in RepoURLs.expectedForms {
            XCTAssertEqual(
                NostrEvent.repositoryCloneURL(from: form.rawValue),
                form.expected
            )
        }
    }

    func testGitSettingsStoreTracksDefaultAndCustomRepoRoots() {
        DemoGitSettingsStore.resetStoredValuesForTesting()
        let store = DemoGitSettingsStore()
        let customRootPath = tempDirectory(name: "custom-root").path

        store.appRepositoriesRootPath = customRootPath
        XCTAssertTrue(store.repositoryRootPaths.contains(DemoGitSettingsStore.defaultRepositoriesRootPath))
        XCTAssertTrue(store.repositoryRootPaths.contains(customRootPath))

        store.resetAppRepositoriesRootPath()

        XCTAssertEqual(store.appRepositoriesRootPath, DemoGitSettingsStore.defaultRepositoriesRootPath)
        XCTAssertTrue(store.repositoryRootPaths.contains(customRootPath))
    }

    @MainActor
    func testRepositoryHostStoreRestoresPreviouslyClonedReposFromDisk() async throws {
        DemoGitSettingsStore.resetStoredValuesForTesting()
        let settings = DemoGitSettingsStore()
        let customRoot = tempDirectory(name: "repo-root")
        let cloneRoot = customRoot.appendingPathComponent("seen-repo", isDirectory: true)
        let remoteURL = URL(string: "https://github.com/\(RepoURLs.owner)/\(RepoURLs.repo).git")!

        try FileManager.default.createDirectory(at: customRoot, withIntermediateDirectories: true)
        try DemoRepositoryHostStore.createRepositoryFixture(at: cloneRoot, remoteURL: remoteURL)

        settings.appRepositoriesRootPath = customRoot.path

        let store = DemoRepositoryHostStore()
        store.attach(gitSettingsStore: settings)

        let restored = await waitForCondition(timeout: 10) {
            store.repositories.contains(where: { $0.remoteURL == remoteURL && $0.localURL.path == cloneRoot.path })
        }

        let discoveredRepositories = store.repositories.map { "\($0.remoteURL.absoluteString) => \($0.localURL.path)" }
        XCTAssertTrue(restored, "roots: \(settings.repositoryRootURLs.map(\.path)) repos: \(discoveredRepositories)")
    }

    private func tempDirectory(name: String) -> URL {
        FileManager.default.temporaryDirectory
            .appendingPathComponent("NostrSDKDemoSmokeTests-\(UUID().uuidString)-\(name)", isDirectory: true)
    }

    @MainActor
    private func waitForCondition(timeout: TimeInterval = 5, pollInterval: UInt64 = 50_000_000, condition: @escaping () -> Bool) async -> Bool {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if condition() {
                return true
            }
            try? await Task.sleep(nanoseconds: pollInterval)
        }

        return condition()
    }

}

private enum RepoURLs {
    static let owner = "gnostr-org"
    static let repo = "gnostr"
    static let identifier = "repo-1"
    static let pubkey = String(repeating: "2", count: 64)
    static let cloneTag = scp

    static let ssh = URL(string: "ssh://git@github.com/\(owner)/\(repo).git")!
    static let nostr = URL(string: "nostr://npub15qydau2hjma6ngxkl2cyar74wzyjshvl65za5k5rl69264ar2exs5cyejr/relay.ngit.dev/ngit")!

    static let expectedForms: [ExpectedForm] = [
        .init(rawValue: https.absoluteString, expected: https),
        .init(rawValue: ssh.absoluteString, expected: ssh),
        .init(rawValue: scp, expected: ssh),
        .init(rawValue: nostr.absoluteString, expected: nostr)
    ]

    private static let https = URL(string: "https://github.com/\(owner)/\(repo).git")!
    private static let scp = "git@github.com:\(owner)/\(repo).git"

    struct ExpectedForm {
        let rawValue: String
        let expected: URL
    }
}
