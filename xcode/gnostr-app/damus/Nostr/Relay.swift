//
//  Relay.swift
//  damus
//
//  Created by William Casarin on 2022-04-11.
//

import Foundation
import SwiftUI

struct RelayInfo: Codable {
    let read: Bool
    let write: Bool

    static let rw = RelayInfo(read: true, write: true)
}

struct RelayDescriptor: Codable {
    let url: URL
    let info: RelayInfo
}

func normalized_relay_url(_ value: String) -> URL? {
    guard !value.contains(where: { $0.isWhitespace }),
          var components = URLComponents(string: value),
          let scheme = components.scheme?.lowercased(),
          scheme == "ws" || scheme == "wss",
          let host = components.host, !host.isEmpty,
          components.user == nil, components.password == nil,
          components.fragment == nil else {
        return nil
    }
    components.scheme = scheme
    components.host = host.lowercased()
    if components.path == "/" {
        components.path = ""
    }
    if components.port == (scheme == "wss" ? 443 : 80) {
        components.port = nil
    }
    return components.url
}

func relay_recommendations(_ event: NostrEvent) -> [RelayDescriptor] {
    return event.tags.compactMap { tag in
        guard tag.count >= 2, tag[0] == "r",
              let url = normalized_relay_url(tag[1]) else {
            return nil
        }
        let info: RelayInfo
        if event.known_kind == .relay_list {
            switch tag.count > 2 ? tag[2] : "" {
            case "": info = .rw
            case "read": info = RelayInfo(read: true, write: false)
            case "write": info = RelayInfo(read: false, write: true)
            default: return nil
            }
        } else {
            info = .rw
        }
        return RelayDescriptor(url: url, info: info)
    }
}

enum RelayFlags: Int {
    case none = 0
    case broken = 1
}

class Relay: Identifiable, ObservableObject {
    let descriptor: RelayDescriptor
    @ObservedObject var connection: RelayConnection
    
    var last_pong: UInt32
    var flags: Int
    
    init(descriptor: RelayDescriptor, connection: RelayConnection) {
        self.flags = 0
        self.descriptor = descriptor
        self.connection = connection
        self.last_pong = 0
    }
    
    func mark_broken() {
        flags |= RelayFlags.broken.rawValue
    }
    
    var is_broken: Bool {
        return (flags & RelayFlags.broken.rawValue) == RelayFlags.broken.rawValue
    }
    
    var id: String {
        return get_relay_id(descriptor.url)
    }

}

enum RelayError: Error {
    case RelayAlreadyExists
    case RelayNotFound
}

func get_relay_id(_ url: URL) -> String {
    return url.absoluteString
}
