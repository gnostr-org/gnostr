//
//  NipService.swift
//  damus
//
//  Created by William Casarin on 2022-05-04.
//

import Foundation
import GnostrGit

class NipService {
    let repo_url = "https://github.com/nostr-protocol/nips.git"
    let repo_name = "nips"
    
    let documentURL = URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)
    lazy var localRepoLocation = documentURL.appendingPathComponent(repo_name)
    lazy var repository = GitRepository(localRepoLocation, credentialManager)
    
    func setup() {
        print("NipService setting up with localRepoLocation: \(localRepoLocation)")
        if repository.exists() {
            fetch()
        } else {
            clone()
        }
    }
    
    func clone() {
        print("Cloning NIPs repository from \(repo_url)")
        repository.clone(repo_url)
    }
    
    func fetch() {
        let allRemotes = repository.getRemotes()
        if let remoteOrigin = allRemotes.first {
            print("Fetching NIPs repository from \(remoteOrigin.url ?? "unknown remote")")
            repository.fetch(remoteOrigin)
        }
    }
}
