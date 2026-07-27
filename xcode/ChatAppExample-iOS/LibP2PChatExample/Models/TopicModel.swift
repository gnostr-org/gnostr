//
//  TopicModel.swift
//  LibP2PChatExample
//
//  Created by Copilot on 2026-07-27.
//

import Foundation
import Combine

final class Topic: ObservableObject, Identifiable {
    var id: String { name }

    let name: String
    @Published var messages: [Message]

    init(name: String, messages: [Message] = []) {
        self.name = name
        self.messages = messages
    }

    var lastMessage: Message? {
        self.messages.last
    }
}
