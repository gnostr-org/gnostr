//
//  Git.swift
//  gnostr
//
//  Created by git on 12/17/25.
//



import SwiftUI

import GnostrGit



let documentURL = URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)



// Do not do this in a real application, put the credentials somewhere safe

// And possibly encrypt them or keychain them by subclassing CredentialsManager

let credentialManager = CredentialsManager(credentialsFileUrl: documentURL.appendingPathComponent("gitcredentials"))



// For push/fetch to work, you might need to add the credential

var credentialAdded = false



func addCredential() {

    do {

        // TODO Change the info here

        try credentialManager.addOrUpdate(nil, Credential(id: "gnostr-org", kind: .password, targetURL: "https://github.com/gnostr-org/", userName: "gnostr", password: "npub15d9enu3v0yxyud4jk0pvxk3kmvrzymjpc6f0eq4ck44vr32qck7smrxq6k"))

        credentialAdded = true

        print("Credential added.")

    } catch let error {

        print("Fail to add credential:", error)

    }

}



struct GitView: View {



    let repo_url: String



    let repo_name: String



    let commitsToFetch: [String]



    



    @StateObject private var repo: GitRepository



    



    init(repo_url: String, repo_name: String, commitsToFetch: [String] = []) {



        self.repo_url = repo_url



        self.repo_name = repo_name



        self.commitsToFetch = commitsToFetch



        



        



        



                let localRepoLocation = documentURL.appendingPathComponent(repo_name)



        



                self._repo = StateObject(wrappedValue: GitRepository(localRepoLocation, credentialManager))



        



                print("GitView initializing with localRepoLocation: \(localRepoLocation)")



    }







        var body: some View {







    







            var filteredCommits: [GitCommit] {







                if commitsToFetch.isEmpty {







                    return repo.commitGraph.commits







                } else {







                    return repo.commitGraph.commits.filter { commitsToFetch.contains($0.id.description) }







                }







            }







    







                VStack {







    







                    //Text("On Mac Catalyst, you should be able to find the cloned repo in `~/Documents/\(repo_name)`.").italic()







    







                Button(repo.exists() ? "Fetch remote Git repo" : "Clone remote Git repo") {







                    if repo.exists() {







                        let allRemotes = repo.getRemotes()







                        if let remoteOrigin = allRemotes.first {







                            print("Repository exists. Fetching from \(remoteOrigin.url ?? "unknown remote")")







                            repo.fetch(remoteOrigin)







                        }







                    } else {







                        print("Cloning repository from \(repo_url)")







                        repo.clone(repo_url)







                    }







                    // We want to do repo.updateCommitGraph() but this will be invoked







                    // on main thread so likely before clone finishes in background thread.







                    // We don't want to do another callback so maybe await/async.







                }







    







                if repo.remoteProgress.inProgress {







                    ProgressView(repo.remoteProgress.operation)







                }







    







                if repo.hasRepo {







                    // Hide the buttons if there are operations in progress







                    if !repo.remoteProgress.inProgress {







                        HStack {







                            Button("Push to origin") {







                                let allRemotes = repo.getRemotes()     // get the list of remotes







                                let remoteOrigin = allRemotes[0]       // assuming you have only one remote i.e. origin







                                repo.push(remoteOrigin, false)         // push all branches to the corresponding one in origin







                            }







    







                            Button("Fetch from origin") {







                                let allRemotes = repo.getRemotes()







                                let remoteOrigin = allRemotes[0]







                                repo.fetch(remoteOrigin)







                            }







    







                            Button("Merge origin/master into current branch") {







                                repo.updateCommitGraph()







                                for c in repo.commitGraph.commits {







                                    for ref in c.refs {







                                        if ref.name == "refs/remotes/origin/master" {







                                            print("Found", ref.name)







                                             repo.merge([ref]) // merge the changes in the remote repo "origin/master" into the local "master"







                                        }







                                    }







                                }







                            }







                        }







                    }







    







                    // At the moment, clone will update hasRepo after completion. So this







                    // has the effect of automatically update the UI if the clone is successful.







                    List(filteredCommits) { commit in







                        VStack(alignment: .leading) {







                            Text(commit.message).bold()







                            Text(commit.author.name)







                        }







                    }







                    .listStyle(.plain)







                }







            }







            .padding(5)







                    .onAppear {







                        if !credentialAdded {







                            addCredential()







                        }







                        repo.open()







                        if repo.exists() {







                            repo.updateCommitGraph()







                            if !commitsToFetch.isEmpty {







                                let allRemotes = repo.getRemotes()







                                if let remoteOrigin = allRemotes.first {







                                    print("Fetching commits: \(commitsToFetch.joined(separator: ", "))")







                                    repo.fetch(remoteOrigin)







                                }







                            }







                        }







                    }







        }



}



struct GitView_Previews: PreviewProvider {



    static var previews: some View {



        GitView(repo_url: "https://github.com/gnostr-org/gnostr.git", repo_name: "gnostr", commitsToFetch: [])



    }



}
