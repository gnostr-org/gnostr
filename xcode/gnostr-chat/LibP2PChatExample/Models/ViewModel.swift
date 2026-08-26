//
//  ViewModel.swift
//  LibP2PChatExample
//
//  Created by Brandon Toms on 5/31/22.
//

import Foundation
import LibP2P
import SwiftUI

#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

class ViewModel: ObservableObject, ChatDelegate {
    @Published var isReady:Bool = false
    @Published var groups: [String]
    @Published var chats: [Chat]

    @Published var nickname: String? = nil {
        didSet {
            /// Save the nickname in user defaults
            UserDefaults.standard.set(self.nickname, forKey: "Nickname")
            if let nickname = self.nickname {
                self.p2pService.updateChatDisplayName(nickname)
            }
        }
    }

    public private(set) var p2pService: LibP2PService!
    private var lifecycleObserverTokens: [NSObjectProtocol] = []

    private func ensureChat(for peer: PeerID, nickname: String? = nil, active: Bool = true) -> Int {
        if let index = self.chats.firstIndex(where: { $0.peer.peer == peer }) {
            return index
        }

        let chat = Chat(
            peer: Person(
                id: peer.b58String,
                peer: peer,
                nickname: nickname ?? peer.shortDescription,
                isActive: active
            ),
            messages: []
        )
        self.chats.append(chat)
        return self.chats.count - 1
    }

    init() {
        // Dummy data
        self.groups = ["No Groups Yet"]
        self.chats = []

        // Restore the chats if possible...
        print("Attempting to restore chats")
        self.restoreChats()

        // Create the shared libp2p service before any nickname restoration can trigger it.
        self.p2pService = LibP2PService.shared

        // Restore our Nickname if we have one saved
        if let nickname = UserDefaults.standard.string(forKey: "Nickname") {
            self.nickname = nickname
        }

        // Instantiate our LibP2PService on a background thread to prevent QOS inversion warnings
        Task(priority: .medium) {
            // Create a Topology subscription for the `/chat/1.0.0` protocol
            // As libp2p discovers peers, and their supported protocols, it will notify us
            self.p2pService.topology(
                TopologyRegistration(
                    protocol: "/chat/1.0.0",
                    handler: TopologyHandler(
                        onConnect: onChatBuddyJoined,
                        onDisconnect: onChatBuddyLeft
                    )
                )
            )

            // Register ourselves as the ChatDelegate
            // `p2pService` will call our `on(message:)` and `on(nickname:)` methods
            self.p2pService.delegate = self

            self.installLifecycleObservers()
            
            // Let our UI know that the libp2p service has initialized (this enables the settings icon and start/stop toggle)
            DispatchQueue.main.async {
                self.isReady = true
            }
        }
    }

    /// This method gets called by our `Topology` Registration when a libp2p peer that supports the `/chat/1.0.0` protocol becomes active / comes online.
    private func onChatBuddyJoined(peer: PeerID, conn: Connection) {
        DispatchQueue.main.async {
            let index = self.ensureChat(for: peer, active: true)
            self.chats[index].peer.isActive = true
            self.p2pService.markPeerConnected(peer)
            if let nickname = self.nickname {
                self.p2pService.updateChatDisplayName(nickname)
            }
        }
    }

    /// This method gets called by our `Topology` Registration when a libp2p peer that supports the `/chat/1.0.0` protocol disconnects / goes offline.
    private func onChatBuddyLeft(peer: PeerID) {
        Task {
            DispatchQueue.main.async {
                print("Chat Buddy Left")
                if let index = self.chats.firstIndex(where: { $0.peer.peer == peer }) {
                    self.chats[index].peer.isActive = false
                }
            }
        }
    }

    /// Part of our `ChatDelegate` protocol
    /// This method is called (by our `LibP2PService`) everytime we receive a message from a peer using the `/chat/1.0.0` protocol
    internal func on(message: String, from: PeerID) {
        DispatchQueue.main.async {
            print("We got a message from libP2P!")
            let index = self.ensureChat(for: from, active: true)
            self.chats[index].peer.isActive = true
            self.p2pService.markPeerConnected(from)
            self.chats[index].messages.append(
                Message(message, type: .received)
            )
        }
    }

    /// Part of our `ChatDelegate` protocol
    /// This method is called (by our `LibP2PService`) everytime we receive a nickname update from a peer using the `/chat/1.0.0` protocol
    internal func on(nickname: String, from: PeerID) {
        DispatchQueue.main.async {
            print("We got a nickname from libP2P!")
            let index = self.ensureChat(for: from, nickname: nickname, active: true)
            self.chats[index].peer.nickname = nickname
            self.p2pService.markPeerConnected(from)
        }
    }

    /// Attempts to start the libp2p service
    /// This includes...
    /// - Starting a TCP Server and listening for inbound TCP requests
    /// - Starting the mDNS Discovery service (searching for libp2p peers on the same network LAN)
    public func startP2PService() async {
        do {
            try await self.p2pService.start()
        } catch {
            print("Failed to start libp2p: \(error)")
        }
    }

    /// Attempts to shutdown the libp2p service
    /// This includes...
    /// - Stopping the TCP Server
    /// - Stopping the mDNS Discovery service
    @MainActor
    public func stopP2PService() {
        // Stop the service
        self.p2pService.stop()
        // Mark all peers inactive
        for i in 0..<self.chats.count {
            self.chats[i].peer.isActive = false
        }
        // Save Chats
        self.saveChats()
    }

    private func installLifecycleObservers() {
        let center = NotificationCenter.default
        let willResignObserver = center.addObserver(
            forName: Self.willResignActiveNotification,
            object: nil,
            queue: .main
        ) { _ in
            print("App dismissed, saving chats and shutting down libp2p")
            self.saveChats()
            self.p2pService.stop()
        }

        let didBecomeActiveObserver = center.addObserver(
            forName: Self.didBecomeActiveNotification,
            object: nil,
            queue: .main
        ) { _ in
            print("App became active, starting libp2p")
            Task.detached(priority: .background) {
                await self.startP2PService()
            }
        }

        self.lifecycleObserverTokens = [willResignObserver, didBecomeActiveObserver]
    }

    private static var willResignActiveNotification: Notification.Name {
        #if canImport(UIKit)
        return UIApplication.willResignActiveNotification
        #elseif canImport(AppKit)
        return NSApplication.willResignActiveNotification
        #else
        return Notification.Name("AppWillResignActiveNotification")
        #endif
    }

    private static var didBecomeActiveNotification: Notification.Name {
        #if canImport(UIKit)
        return UIApplication.didBecomeActiveNotification
        #elseif canImport(AppKit)
        return NSApplication.didBecomeActiveNotification
        #else
        return Notification.Name("AppDidBecomeActiveNotification")
        #endif
    }

    /// Sends a single message to a Chat buddy
    public func send(message: String, to chat: Chat) -> Message? {
        if let index = self.chats.firstIndex(where: { $0.id == chat.id }) {
            let msg = Message(message, type: .sent)
            self.chats[index].messages.append(msg)
            self.p2pService.send(message: message, to: chat.peer.peer)
            return msg
        }
        return nil
    }

    /// Sends  a Nickname update message to a Chat buddy
    private func send(nickname: String, to peer: PeerID) {
        _ = peer
        self.p2pService.updateChatDisplayName(nickname)
    }

    /// Checks whether a Chat buddy is currently active (whether or not we have an open connection established with them)
    public func isActive(peer: Person) -> Bool {
        if let index = self.chats.firstIndex(where: { $0.peer.peer == peer.peer }) {
            return self.chats[index].peer.isActive
        } else {
            return false
        }
    }

    /// Save the chats out to UserDefaults
    private func saveChats() {
        if let chatData = try? JSONEncoder().encode(self.chats) {
            UserDefaults.standard.set(chatData, forKey: "MyChats")
            print("Saved chats")
        } else {
            print("Failed to save chats")
        }

//        let fileManager = FileManager.default
//        do {
//            let documentDirectory = try fileManager.url(for: ., in: .userDomainMask, appropriateFor:nil, create:false)
//            let fileURL = documentDirectory.appendingPathComponent("chats.json")
//
//            let chatsData = try JSONEncoder().encode(self.chats)
//            if !fileManager.fileExists(atPath: fileURL.path) {
//                print("Creating a chats.json file")
//                fileManager.createFile(atPath: fileURL.path, contents: chatsData)
//            } else {
//                print("overwriting chats.json file")
//                try chatsData.write(to: fileURL)
//            }
//
//        } catch {
//            print(error)
//        }
    }

    /// Restore the chats from UserDefaults if they exist
    private func restoreChats() {
        if let chatData = UserDefaults.standard.data(forKey: "MyChats") {
            if let chats = try? JSONDecoder().decode([Chat].self, from: chatData) {
                self.chats = chats
                print("Restored chats!")
            }
        } else {
            print("Failed to restore chats")
        }

//        let fileManager = FileManager.default
//        do {
//            let documentDirectory = try fileManager.url(for: .applicationDirectory, in: .userDomainMask, appropriateFor:nil, create:false)
//            let fileURL = documentDirectory.appendingPathComponent("chats.json")
//            if let chatsData = fileManager.contents(atPath: fileURL.path) {
//                print("Got contents from our chats.json")
//                self.chats = try JSONDecoder().decode([Chat].self, from: chatsData)
//                print(self.chats)
//            } else {
//                print("No data at chats.json")
//            }
//        } catch {
//            print(error)
//        }
    }

    /// Deletes all stored chats
    public func deleteChats() {
        // Delete all chats...
        self.chats = []
        // Save the new empty chat list...
        self.saveChats()
    }
}
