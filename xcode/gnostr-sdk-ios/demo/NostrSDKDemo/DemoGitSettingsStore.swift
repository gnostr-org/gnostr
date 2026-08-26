//
//  DemoGitSettingsStore.swift
//  NostrSDKDemo
//
//  Created by Copilot on 6/9/26.
//

import Foundation
import SwiftUI

final class DemoGitSettingsStore: ObservableObject {
    private enum Keys {
        static let appRepositoriesRootPath = "demo.git.appRepositoriesRootPath"
        static let repositoryRootPaths = "demo.git.repositoryRootPaths"
    }

    @Published var appRepositoriesRootPath: String {
        didSet {
            UserDefaults.standard.set(appRepositoriesRootPath, forKey: Keys.appRepositoriesRootPath)
            registerRepositoryRootPath(appRepositoriesRootPath)
        }
    }

    @Published private(set) var repositoryRootPaths: [String]

    init() {
        let storedActiveRootPath = UserDefaults.standard.string(forKey: Keys.appRepositoriesRootPath)
        let activeRootPath = storedActiveRootPath ?? Self.defaultRepositoriesRootPath
        appRepositoriesRootPath = activeRootPath
        repositoryRootPaths = UserDefaults.standard.stringArray(forKey: Keys.repositoryRootPaths) ?? []
        registerRepositoryRootPath(Self.defaultRepositoriesRootPath)
        registerRepositoryRootPath(activeRootPath)
    }

    var appRepositoriesRootURL: URL {
        let trimmed = appRepositoriesRootPath.trimmingCharacters(in: .whitespacesAndNewlines)
        if trimmed.isEmpty {
            return Self.defaultRepositoriesRootURL
        }
        return URL(fileURLWithPath: (trimmed as NSString).expandingTildeInPath, isDirectory: true)
    }

    func resetAppRepositoriesRootPath() {
        appRepositoriesRootPath = Self.defaultRepositoriesRootPath
    }

#if DEBUG
    static func resetStoredValuesForTesting() {
        UserDefaults.standard.removeObject(forKey: Keys.appRepositoriesRootPath)
        UserDefaults.standard.removeObject(forKey: Keys.repositoryRootPaths)
    }
#endif

    func registerRepositoryRootPath(_ rootPath: String) {
        let normalizedRootPath = Self.normalize(rootPath)
        guard repositoryRootPaths.contains(normalizedRootPath) == false else { return }

        repositoryRootPaths.append(normalizedRootPath)
        UserDefaults.standard.set(repositoryRootPaths, forKey: Keys.repositoryRootPaths)
    }

    var repositoryRootURLs: [URL] {
        repositoryRootPaths.map(Self.makeFileURL(path:))
    }

    static var defaultRepositoriesRootURL: URL {
        let baseURL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
        ?? FileManager.default.temporaryDirectory

        return baseURL
            .appendingPathComponent("NostrSDKDemo", isDirectory: true)
            .appendingPathComponent("HostedRepos", isDirectory: true)
    }

    static var defaultRepositoriesRootPath: String {
        defaultRepositoriesRootURL.path
    }

    private static func makeFileURL(path: String) -> URL {
        URL(fileURLWithPath: path, isDirectory: true)
    }

    private static func normalize(_ path: String) -> String {
        (path.trimmingCharacters(in: .whitespacesAndNewlines) as NSString).expandingTildeInPath
    }
}
