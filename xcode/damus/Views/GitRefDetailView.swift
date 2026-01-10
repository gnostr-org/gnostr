import SwiftUI
import GnostrGit

struct GitRefDetailView: View {
    let repo_name: String
    let refName: String
    let commitHash: String
    @Binding var isPresented: Bool // New binding for dismissal
    
    @StateObject private var repo: GitRepository
    @State private var commitDetails: CommitDetails? = nil

    private static let itemFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .short
        return formatter
    }()

    struct CommitDetails {
        let message: String
        let author: String
        let date: Date
    }

    init(repo_name: String, refName: String, commitHash: String, isPresented: Binding<Bool>) {
        self.repo_name = repo_name
        self.refName = refName
        self.commitHash = commitHash
        self._isPresented = isPresented
        
        let localRepoLocation = documentURL.appendingPathComponent(repo_name)
        self._repo = StateObject(wrappedValue: GitRepository(localRepoLocation, credentialManager))
        print("GitRefDetailView initializing with localRepoLocation: \(localRepoLocation)")
    }

    var body: some View {
        ZStack(alignment: .topTrailing) { // Use ZStack to position the close button
            VStack(alignment: .leading, spacing: 10) {
                Text(refName)
                    .font(.title)
                    .bold()
                
                Text(commitHash)
                    .font(.body.monospaced())
                    .foregroundColor(.gray)
                
                Divider()
                
                if let details = commitDetails {
                    Text(details.message)
                        .font(.body)
                    Text("Author: \(details.author)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Text("Date: \(details.date, formatter: Self.itemFormatter)")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    VStack {
                        ProgressView()
                            .progressViewStyle(CircularProgressViewStyle())
                        Text("Loading commit details...")
                            .font(.body)
                            .foregroundColor(.gray)
                    }
                }
                
                Spacer()
            }
            .padding()
            .onAppear(perform: findCommit)
            .onChange(of: isPresented) { newValue in
                print("GitRefDetailView - isPresented changed to \(newValue)")
            }
            .onChange(of: repo.remoteProgress.inProgress) { inProgress in
                if !inProgress {
                    self.repo.updateCommitGraph()
                    if let commit = self.repo.commitGraph.commits.first(where: { $0.id.description == self.commitHash }) {
                        self.commitDetails = CommitDetails(
                            message: commit.message,
                            author: commit.author.name,
                            date: commit.time
                        )
                    } else {
                        self.commitDetails = CommitDetails(message: "Commit not found after fetching.", author: "N/A", date: Date())
                    }
                }
            }
            #if os(macOS) || targetEnvironment(macCatalyst)
            .keyboardShortcut(.escape)
            #endif
            .overlay(
                // Close button
                Button(action: {
                    print("Close button tapped.")
                    isPresented = false
                }) {
                    Image(systemName: "xmark.circle.fill")
                        .font(.title2)
                        .foregroundColor(.gray)
                        .padding(10)
                }, alignment: .topTrailing
            )
        }
    }

    private func findCommit() {
        repo.open()
        
        if !repo.exists() {
            self.commitDetails = CommitDetails(message: "Repository not found locally. Please clone it first.", author: "N/A", date: Date())
            return
        }

        print("Updating commit graph...")
        repo.updateCommitGraph()
        print("Commit graph updated. Commits found: \(repo.commitGraph.commits.count)")
        
        if let commit = repo.commitGraph.commits.first(where: { $0.id.description == commitHash }) {
            print("Commit found locally: \(commit.id.description)")
            self.commitDetails = CommitDetails(
                message: commit.message,
                author: commit.author.name,
                date: commit.time
            )
        } else {
            print("Commit with hash \(commitHash) not found locally. Attempting to fetch from remote...")
            let allRemotes = repo.getRemotes()
            if let remoteOrigin = allRemotes.first {
                // Trigger fetch, then rely on onChange(of: repo.remoteProgress.inProgress) for update
                repo.fetch(remoteOrigin)
                self.commitDetails = CommitDetails(message: "Fetching commit from remote...", author: "N/A", date: Date())
            } else {
                self.commitDetails = CommitDetails(message: "Commit not found and no remote to fetch from.", author: "N/A", date: Date())
            }
        }
    }
}
