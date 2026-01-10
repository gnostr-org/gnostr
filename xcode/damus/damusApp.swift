//
//  damusApp.swift
//  damus
//
//  Created by William Casarin on 2022-04-01.
//

import SwiftUI
import GnostrGit
import Combine // Import Combine for ObservableObject

class GitOperationTracker: ObservableObject {
    @Published var activeClones: [String: String] = [:] // repo_name: status_message
    private var operationCancellables: [String: Set<AnyCancellable>] = [:]

    func startGitOperation(repo_url: String, repo_name: String, commitsToFetch: [String]) {
        if activeClones[repo_name] != nil {
            print("Operation for \(repo_name) already in progress.")
            return
        }

        activeClones[repo_name] = "Initiating..."

        let localRepoLocation = documentURL.appendingPathComponent(repo_name)
        let repo = GitRepository(localRepoLocation, credentialManager)

        if !credentialAdded {
            addCredential()
        }

        var cancellables = Set<AnyCancellable>()

        repo.remoteProgress.$inProgress
            .dropFirst()
            .sink { [weak self] inProgress in
                guard let self = self else { return }
                if !inProgress {
                    if repo.remoteProgress.errorReceiver.hasError {
                        let errorMessage = repo.remoteProgress.errorReceiver.extraMessage ?? "Unknown error"
                        print("Git operation for \(repo_name) failed: \(errorMessage)")
                        DispatchQueue.main.async {
                            self.activeClones[repo_name] = "Failed: \(errorMessage)"
                            self.operationCancellables[repo_name]?.removeAll()
                            self.operationCancellables[repo_name] = nil
                        }
                    } else {
                        print("Git operation for \(repo_name) completed successfully.")
                        DispatchQueue.main.async {
                            self.activeClones[repo_name] = "Completed"
                            // Optionally remove after a delay or keep for user to see status
                            DispatchQueue.main.asyncAfter(deadline: .now() + 5) {
                                self.activeClones[repo_name] = nil
                            }
                            self.operationCancellables[repo_name]?.removeAll()
                            self.operationCancellables[repo_name] = nil
                        }
                    }
                } else {
                    DispatchQueue.main.async {
                        self.activeClones[repo_name] = repo.remoteProgress.operation
                    }
                }
            }
            .store(in: &cancellables)

        self.operationCancellables[repo_name] = cancellables // Retain cancellables

        repo.open()

        if repo.exists() {
            let allRemotes = repo.getRemotes()
            if let remoteOrigin = allRemotes.first {
                print("Repository exists. Fetching from \(remoteOrigin.url ?? "unknown remote")")
                repo.fetch(remoteOrigin)
            } else {
                let errorMessage = "Repository exists but no remote 'origin' found for fetching."
                print("Error: \(errorMessage)")
                DispatchQueue.main.async {
                    self.activeClones[repo_name] = "Failed: \(errorMessage)"
                    self.operationCancellables[repo_name]?.removeAll()
                    self.operationCancellables[repo_name] = nil
                }
            }
        } else {
            print("Cloning repository from \(repo_url)")
            repo.clone(repo_url)
        }
    }
}

struct RepoInfo: Identifiable {
    let id = UUID()
    let url: String
    let name: String
    let commitsToFetch: [String]
}

@main
struct damusApp: App {
    let nipService = NipService()
    let gnostrService = GnostrService()
    let timer = Timer.publish(every: 3600, on: .main, in: .common).autoconnect() // Fetch every hour
    @StateObject var webViewURL = WebViewURL()
    @StateObject var webViewModel = WebViewModel()
    @StateObject var gitOperationTracker = GitOperationTracker() // Instantiate the tracker
    @State private var repoToClone: RepoInfo?
    
    var body: some Scene {
        WindowGroup {
            GeometryReader { geometry in
                ZStack {
                    MainView()
                        .environmentObject(webViewURL)
                        .environmentObject(webViewModel)
                        .environmentObject(gitOperationTracker) // Pass as EnvironmentObject
                        .onAppear {
                            nipService.setup()
                            gnostrService.setup()
                            
                            // --- TEMPORARY WEBSOCKET TEST ---
                            let testURL = URL(string: "ws://127.0.0.1:8080")!
                            let testConnection = RelayConnection(url: testURL) { event in
                                print("WebSocket Test Event: \(event)")
                            }
                            testConnection.connect()
                            // --- END TEMPORARY WEBSOCKET TEST ---

                            webViewModel.onCloneTapped = { url, name, commits in
                                gitOperationTracker.startGitOperation(repo_url: url, repo_name: name, commitsToFetch: commits)
                                self.webViewURL.url = nil // Dismiss the WebView after initiating clone
                            }
                        }
                        .onReceive(timer) { _ in
                            nipService.fetch()
                            gnostrService.fetch()
                        }
                    
                    if let url = webViewURL.url {
                        VStack {
                            HStack {
                                Button(action: {
                                    webViewModel.goBack()
                                }) {
                                    Image(systemName: "chevron.left")
                                }
                                .disabled(!webViewModel.canGoBack)
                                
                                Button(action: {
                                    webViewModel.goForward()
                                }) {
                                    Image(systemName: "chevron.right")
                                }
                                .disabled(!webViewModel.canGoForward)
                                
                                Button(action: {
                                    webViewModel.refresh()
                                }) {
                                    Image(systemName: "arrow.clockwise")
                                }
                                
                                Spacer()
                                
                                if webViewModel.repo_url != nil {
                                    Button(action: {
                                        webViewModel.clone()
                                    }) {
                                        Image(systemName: "arrow.down.circle")
                                    }
                                }
                                
                                Button(action: {
                                    webViewURL.url = nil
                                }) {
                                    Image(systemName: "xmark")
                                }
                            }
                            .padding()
                            
                            WebView(url: url, viewModel: webViewModel)
                            
                            Divider()
                            
                            HStack {
                                Text("Bottom Toolbar")
                            }
                            .padding()
                        }
                        .frame(width: geometry.size.width * 0.8, height: geometry.size.height * 0.8)
                        .background(Color(.systemBackground))
                        .cornerRadius(12)
                        .shadow(radius: 20)
                        .position(x: geometry.size.width / 2, y: geometry.size.height / 2)
                    }
                }
            }
            .sheet(item: $repoToClone) { repoInfo in
                GitView(repo_url: repoInfo.url, repo_name: repoInfo.name, commitsToFetch: repoInfo.commitsToFetch)
            }
        }
    }
    
    struct MainView: View {
        @State var needs_setup = false;
        @State var keypair: Keypair? = nil;
        @EnvironmentObject var webViewURL: WebViewURL
        
        var body: some View {
            Group {
                if let kp = keypair, !needs_setup {
                    ContentView(keypair: kp)
                } else {
                    SetupView()
                        .onReceive(handle_notify(.login)) { notif in
                            needs_setup = false
                            keypair = get_saved_keypair()
                        }
                }
            }
            .background(
                KeyPressView {
                    webViewURL.url = nil
                }
            )
            .onReceive(handle_notify(.logout)) { _ in
                try? clear_keypair()
                keypair = nil
            }
            .onAppear {
                keypair = get_saved_keypair()
            }
        }
    }
    
    func needs_setup() -> Keypair? {
        return get_saved_keypair()
    }
    
}
