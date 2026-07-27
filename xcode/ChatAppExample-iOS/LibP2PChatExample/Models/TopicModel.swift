//
//  TopicModel.swift
//  LibP2PChatExample
//
//  Created by Copilot on 2026-07-27.
//

import Foundation
import Combine
import PeerID

final class Topic: ObservableObject, Identifiable {
    var id: String { name }

    let name: String
    @Published var messages: [Message]
    @Published var members: [Person]

    init(name: String, messages: [Message] = [], members: [Person] = []) {
        self.name = name
        self.messages = messages
        self.members = members
    }

    var lastMessage: Message? {
        self.messages.last
    }

    func upsertMember(_ peer: PeerID, isActive: Bool = true) {
        let memberID = peer.b58String
        if let index = self.members.firstIndex(where: { $0.id == memberID }) {
            self.members[index].isActive = isActive
            self.members[index].nickname = peer.shortDescription
            return
        }

        self.members.append(
            Person(
                id: memberID,
                peer: peer,
                nickname: peer.shortDescription,
                isActive: isActive
            )
        )
    }

    func markMemberInactive(_ peer: PeerID) {
        self.upsertMember(peer, isActive: false)
    }
}
